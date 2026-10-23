import type { Anchor, CanonicalSentence, SentenceWindow } from "./types";

const median = (values: number[]) => {
  const sorted = [...values].sort((a, b) => a - b);
  if (!sorted.length) return 2.6;
  const middle = Math.floor(sorted.length / 2);
  return sorted.length % 2 ? sorted[middle] : (sorted[middle - 1] + sorted[middle]) / 2;
};

const clamp = (value: number, minimum: number, maximum: number) =>
  Math.max(minimum, Math.min(maximum, value));

export interface WindowDerivation {
  windows: SentenceWindow[];
  unspokenBlocks: Set<number>;
  unspokenWords: Set<number>;
  wordsPerSecond: number;
  errors: string[];
}

export const deriveSentenceWindows = (
  sentences: CanonicalSentence[],
  allAnchors: Anchor[],
  audioDuration: number,
): WindowDerivation => {
  const anchorBySentence = new Map<string, Anchor[]>();
  const reliableAnchors = allAnchors.filter((anchor) => anchor.reliable);
  for (const sentence of sentences) {
    const anchors = reliableAnchors.filter((anchor) =>
      sentence.words.some((word) => word.globalIndex === anchor.canonicalIndex),
    );
    // A pair of contextual exact matches is still more useful than treating a
    // sentence as wholly unknown, even if the surrounding run was short.
    if (!anchors.length) {
      const exact = allAnchors.filter(
        (anchor) =>
          anchor.kind === "exact" &&
          sentence.words.some((word) => word.globalIndex === anchor.canonicalIndex),
      );
      if (exact.length >= 2) anchors.push(...exact);
    }
    anchorBySentence.set(sentence.id, anchors);
  }

  const rates: number[] = [];
  for (const sentence of sentences) {
    const anchors = anchorBySentence.get(sentence.id) ?? [];
    if (anchors.length < 2) continue;
    const first = anchors[0];
    const last = anchors[anchors.length - 1];
    const seconds = last.end - first.start;
    if (seconds <= 0) continue;
    const words = last.canonicalIndex - first.canonicalIndex + 1;
    rates.push(words / seconds);
  }
  const wordsPerSecond = clamp(median(rates), 1.8, 3.6);
  const windows = Array.from({ length: sentences.length }, () => null as SentenceWindow | null);
  const unspokenWords = new Set<number>();

  const previousAnchor = (canonicalIndex: number) => {
    for (let index = reliableAnchors.length - 1; index >= 0; index -= 1) {
      if (reliableAnchors[index].canonicalIndex < canonicalIndex) return reliableAnchors[index];
    }
    return undefined;
  };
  const nextAnchor = (canonicalIndex: number) =>
    reliableAnchors.find((anchor) => anchor.canonicalIndex > canonicalIndex);

  for (const [index, sentence] of sentences.entries()) {
    const anchors = anchorBySentence.get(sentence.id) ?? [];
    if (!anchors.length) continue;
    const first = anchors[0];
    const last = anchors[anchors.length - 1];
    const firstPosition = sentence.words.findIndex(
      (word) => word.globalIndex === first.canonicalIndex,
    );
    const lastPosition = sentence.words.findIndex(
      (word) => word.globalIndex === last.canonicalIndex,
    );
    const missingHead = Math.max(0, firstPosition);
    const missingTail = Math.max(0, sentence.words.length - lastPosition - 1);
    const before = previousAnchor(sentence.words[0].globalIndex);
    const after = nextAnchor(sentence.words[sentence.words.length - 1].globalIndex);
    const headGap = first.start - (before?.end ?? 0);
    const tailGap = (after?.start ?? audioDuration) - last.end;
    const omitHead = missingHead > 0 && headGap < (missingHead / wordsPerSecond) * 0.75;
    const omitTail = missingTail > 0 && tailGap < (missingTail / wordsPerSecond) * 0.75;
    if (omitHead) {
      for (const word of sentence.words.slice(0, firstPosition)) {
        unspokenWords.add(word.globalIndex);
      }
    }
    if (omitTail) {
      for (const word of sentence.words.slice(lastPosition + 1)) {
        unspokenWords.add(word.globalIndex);
      }
    }
    const spokenMissingHead = omitHead ? 0 : missingHead;
    const spokenMissingTail = omitTail ? 0 : missingTail;
    let start = first.start - (spokenMissingHead / wordsPerSecond) * 1.35 - 0.4;
    let end = last.end + (spokenMissingTail / wordsPerSecond) * 1.35 + 0.4;
    if (before) start = Math.max(start, before.end - 0.25);
    if (after) end = Math.min(end, after.start + 0.25);
    start = clamp(start, 0, audioDuration);
    end = clamp(end, start + 0.15, audioDuration);
    if (end - start > 28) end = start + 28;
    windows[index] = {
      id: sentence.id,
      blockIndex: sentence.blockIndex,
      sentenceIndex: sentence.sentenceIndex,
      start,
      end,
      anchors,
      interpolated: false,
      level: 1,
    };
  }

  const unspokenBlocks = new Set<number>();
  const errors: string[] = [];
  let cursor = 0;
  while (cursor < sentences.length) {
    if (windows[cursor]) {
      cursor += 1;
      continue;
    }
    const runStart = cursor;
    while (cursor < sentences.length && !windows[cursor]) cursor += 1;
    const runEnd = cursor;
    const run = sentences.slice(runStart, runEnd);
    const firstWord = run[0].words[0]?.globalIndex ?? 0;
    const lastWord = run[run.length - 1].words.at(-1)?.globalIndex ?? firstWord;
    const before = previousAnchor(firstWord);
    const after = nextAnchor(lastWord);
    const gapStart = before?.end ?? 0;
    const gapEnd = after?.start ?? audioDuration;
    const available = Math.max(0, gapEnd - gapStart);
    const totalWords = run.reduce((sum, sentence) => sum + sentence.words.length, 0);
    const expected = totalWords / wordsPerSecond;
    const ratio = expected > 0 ? available / expected : 1;

    if (ratio < 0.3) {
      const blocks = new Set(run.map((sentence) => sentence.blockIndex));
      for (const blockIndex of blocks) {
        const blockSentences = sentences.filter((sentence) => sentence.blockIndex === blockIndex);
        const allInRun = blockSentences.every((sentence) => run.includes(sentence));
        const hasAnyAnchor = blockSentences.some(
          (sentence) => (anchorBySentence.get(sentence.id)?.length ?? 0) > 0,
        );
        if (allInRun && !hasAnyAnchor) unspokenBlocks.add(blockIndex);
        else {
          errors.push(
            `Ambiguous unanchored speech in block ${blockIndex}; ${available.toFixed(2)}s available for ${totalWords} words`,
          );
        }
      }
      continue;
    }

    let time = gapStart;
    for (let index = runStart; index < runEnd; index += 1) {
      const sentence = sentences[index];
      const share = sentence.words.length / Math.max(1, totalWords);
      const duration = available * share;
      const start = clamp(time - 0.25, 0, audioDuration);
      const end = clamp(time + duration + 0.25, start + 0.15, audioDuration);
      windows[index] = {
        id: sentence.id,
        blockIndex: sentence.blockIndex,
        sentenceIndex: sentence.sentenceIndex,
        start,
        end: Math.min(end, start + 28),
        anchors: [],
        interpolated: true,
        level: 1,
      };
      time += duration;
    }
  }

  return {
    windows: windows.filter((window): window is SentenceWindow => Boolean(window)),
    unspokenBlocks,
    unspokenWords,
    wordsPerSecond,
    errors,
  };
};

export const widenWindow = (
  window: SentenceWindow,
  audioDuration: number,
  level: number,
): SentenceWindow => {
  const padding = level === 2 ? 1.5 : level === 3 ? 3 : 5;
  return {
    ...window,
    start: Math.max(0, window.start - padding),
    end: Math.min(audioDuration, window.end + padding),
    level,
  };
};
