import { createHash } from "node:crypto";
import { splitSentences, splitWords } from "../../../src/components/SpeakAloud/narration";
import type { AlignmentBlock, AlignmentSentence } from "../../../src/components/SpeakAloud/types";
import { normalizeToken, normalizeVariants } from "./normalize";
import { scoreSentence, validateArtifactTimings } from "./quality";
import { alignSequences } from "./sequence";
import type {
  AlignmentJob,
  Anchor,
  AsrWord,
  CanonicalSentence,
  CanonicalWord,
  SentenceAlignmentResult,
  SentenceWindow,
} from "./types";
import { deriveSentenceWindows, widenWindow } from "./windows";

export interface SourceBlock {
  index: number;
  tag: string;
  text: string;
}

export interface PipelineDiagnostics {
  wordsPerSecond: number;
  canonicalWords: number;
  asrWords: number;
  matches: number;
  reliableAnchors: number;
  unspokenBlocks: number[];
  unspokenWords: number;
  anchorMedianErrors: number[];
  retries: Record<string, number>;
  failures: { id: string; text: string; reasons: string[]; window: SentenceWindow }[];
}

export interface PipelineResult {
  blocks: AlignmentBlock[];
  diagnostics: PipelineDiagnostics;
}

export class NarrationPipelineError extends Error {
  constructor(
    message: string,
    readonly diagnostics: PipelineDiagnostics,
  ) {
    super(message);
  }
}

const hash = (value: string) => `sha256:${createHash("sha256").update(value).digest("hex")}`;

export const buildCanonicalSentences = (blocks: SourceBlock[]) => {
  const sentences: CanonicalSentence[] = [];
  const words: CanonicalWord[] = [];
  for (const block of blocks) {
    const sentenceParts = splitSentences(block.text);
    for (const [sentenceIndex, sentencePart] of sentenceParts.entries()) {
      const sentenceWords: CanonicalWord[] = [];
      for (const [wordIndex, part] of splitWords(sentencePart.text).entries()) {
        const norm = normalizeToken(part.text);
        if (!norm) continue;
        const word: CanonicalWord = {
          text: part.text,
          norm,
          charStart: sentencePart.start + part.start,
          charEnd: sentencePart.start + part.end,
          blockIndex: block.index,
          sentenceIndex,
          wordIndex,
          globalIndex: words.length,
        };
        words.push(word);
        sentenceWords.push(word);
      }
      if (!sentenceWords.length) continue;
      sentences.push({
        id: `b${block.index}s${sentenceIndex}`,
        text: sentencePart.text,
        charStart: sentencePart.start,
        charEnd: sentencePart.end,
        blockIndex: block.index,
        sentenceIndex,
        words: sentenceWords,
      });
    }
  }
  return { sentences, words };
};

const jobsFor = (sentences: Map<string, CanonicalSentence>, windows: SentenceWindow[]) =>
  windows.map(
    (window): AlignmentJob => ({
      id: window.id,
      words: sentences.get(window.id)!.words.map((word) => word.text),
      windowStart: window.start,
      windowEnd: window.end,
    }),
  );

export const runTwoPassPipeline = async (
  blocks: SourceBlock[],
  asrWords: AsrWord[],
  audioDuration: number,
  alignBatch: (jobs: AlignmentJob[]) => Promise<SentenceAlignmentResult[]>,
): Promise<PipelineResult> => {
  const canonical = buildCanonicalSentences(blocks);
  const usableAsr = asrWords.filter((word) => normalizeToken(word.text));
  const matches = alignSequences(
    canonical.words.map((word) => ({ text: word.text, variants: normalizeVariants(word.text) })),
    usableAsr.map((word) => ({ text: word.text, variants: normalizeVariants(word.text) })),
  );
  const anchors: Anchor[] = matches.map((match) => ({
    canonicalIndex: match.canonicalIndex,
    asrIndex: match.asrIndex,
    start: usableAsr[match.asrIndex].start,
    end: usableAsr[match.asrIndex].end,
    probability: usableAsr[match.asrIndex].probability,
    reliable: match.reliable,
    kind: match.kind,
  }));
  const windowData = deriveSentenceWindows(canonical.sentences, anchors, audioDuration);
  const effectiveSentences = canonical.sentences
    .map((sentence) => {
      const words = sentence.words.filter(
        (word) => !windowData.unspokenWords.has(word.globalIndex),
      );
      if (!words.length) return null;
      return {
        ...sentence,
        charStart: words[0].charStart,
        charEnd: words.at(-1)!.charEnd,
        words,
      };
    })
    .filter((sentence): sentence is CanonicalSentence => Boolean(sentence));
  const sentenceById = new Map(effectiveSentences.map((sentence) => [sentence.id, sentence]));
  const diagnostics: PipelineDiagnostics = {
    wordsPerSecond: windowData.wordsPerSecond,
    canonicalWords: canonical.words.length,
    asrWords: usableAsr.length,
    matches: matches.length,
    reliableAnchors: matches.filter((match) => match.reliable).length,
    unspokenBlocks: [...windowData.unspokenBlocks],
    unspokenWords: windowData.unspokenWords.size,
    anchorMedianErrors: [],
    retries: {},
    failures: [],
  };
  if (windowData.errors.length) {
    throw new NarrationPipelineError(windowData.errors.join("\n"), diagnostics);
  }

  const accepted = new Map<string, AlignmentSentence>();
  let pending = windowData.windows.filter((window) => sentenceById.has(window.id));
  for (let level = 1; level <= 4 && pending.length; level += 1) {
    const results = await alignBatch(jobsFor(sentenceById, pending));
    const resultById = new Map(results.map((result) => [result.id, result]));
    const next: SentenceWindow[] = [];
    diagnostics.failures = [];
    for (const window of pending) {
      const sentence = sentenceById.get(window.id)!;
      const result = resultById.get(window.id);
      const sentenceAnchors = anchors.filter((anchor) =>
        sentence.words.some((word) => word.globalIndex === anchor.canonicalIndex),
      );
      if (!result?.ok) {
        const reasons = [result?.error ?? "aligner returned no result"];
        diagnostics.failures.push({ id: window.id, text: sentence.text, reasons, window });
        if (level < 4) next.push(widenWindow(window, audioDuration, level + 1));
        continue;
      }
      const scored = scoreSentence(sentence, window, result.words, sentenceAnchors);
      if (!scored.ok) {
        diagnostics.failures.push({
          id: window.id,
          text: sentence.text,
          reasons: scored.reasons,
          window,
        });
        if (level < 4) next.push(widenWindow(window, audioDuration, level + 1));
        continue;
      }
      accepted.set(window.id, {
        charStart: sentence.charStart,
        charEnd: sentence.charEnd,
        start: scored.words[0][2],
        end: scored.words.at(-1)![3],
        words: scored.words,
      });
      diagnostics.retries[window.id] = level - 1;
      if (scored.anchorMedianError !== null) {
        diagnostics.anchorMedianErrors.push(scored.anchorMedianError);
      }
    }
    pending = next;
  }
  if (pending.length || diagnostics.failures.length) {
    throw new NarrationPipelineError(
      `${diagnostics.failures.length} sentences failed automatic alignment`,
      diagnostics,
    );
  }

  const outputBlocks: AlignmentBlock[] = blocks.map((block) => {
    if (windowData.unspokenBlocks.has(block.index)) {
      return {
        index: block.index,
        spoken: false,
        hash: hash(block.text),
        text: block.text,
        start: 0,
        end: 0,
        sentences: [],
      };
    }
    const blockSentences = effectiveSentences
      .filter((sentence) => sentence.blockIndex === block.index)
      .map((sentence) => accepted.get(sentence.id))
      .filter((sentence): sentence is AlignmentSentence => Boolean(sentence));
    if (!blockSentences.length) {
      throw new NarrationPipelineError(
        `No aligned sentences for spoken block ${block.index}`,
        diagnostics,
      );
    }
    return {
      index: block.index,
      spoken: true,
      hash: hash(block.text),
      text: block.text,
      start: blockSentences[0].start,
      end: blockSentences.at(-1)!.end,
      sentences: blockSentences,
    };
  });

  const orderedSentences = outputBlocks.flatMap((block) =>
    block.sentences.map((sentence, sentenceIndex) => ({
      sentence,
      id: `b${block.index}s${sentenceIndex}`,
    })),
  );
  for (let index = 1; index < orderedSentences.length; index += 1) {
    const previousEntry = orderedSentences[index - 1];
    const currentEntry = orderedSentences[index];
    const previous = previousEntry.sentence;
    const current = currentEntry.sentence;
    const overlap = previous.end - current.start;
    if (overlap <= 0) continue;
    if (overlap > 0.5) {
      throw new NarrationPipelineError(
        `${previousEntry.id} and ${currentEntry.id} overlap by ${overlap.toFixed(2)}s after local alignment`,
        diagnostics,
      );
    }
    const previousWord = previous.words.at(-1)!;
    const currentWord = current.words[0];
    const span = currentWord[3] - previousWord[2];
    if (span < 0.02) {
      throw new NarrationPipelineError(
        `Could not reconcile a ${overlap.toFixed(2)}s overlap between ${previousEntry.id} and ${currentEntry.id}`,
        diagnostics,
      );
    }
    const previousWeight = Math.max(1, previousWord[1] - previousWord[0]);
    const currentWeight = Math.max(1, currentWord[1] - currentWord[0]);
    const boundary = previousWord[2] + (span * previousWeight) / (previousWeight + currentWeight);
    previousWord[3] = boundary;
    previous.end = boundary;
    currentWord[2] = boundary;
    current.start = boundary;
  }
  for (const block of outputBlocks) {
    if (!block.spoken) continue;
    block.start = block.sentences[0].start;
    block.end = block.sentences.at(-1)!.end;
  }

  const timingErrors = validateArtifactTimings({
    version: 1,
    recording: "validation",
    generatedAt: "",
    article: { source: "", textHash: "", blockCount: outputBlocks.length },
    audio: { file: "", sha256: "", duration: audioDuration },
    aligner: { name: "stable-ts", version: "2.19.1", model: "two-pass" },
    blocks: outputBlocks,
  });
  if (timingErrors.length) {
    throw new NarrationPipelineError(timingErrors.join("\n"), diagnostics);
  }

  return { blocks: outputBlocks, diagnostics };
};
