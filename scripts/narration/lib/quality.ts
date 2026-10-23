import { normalizeToken } from "./normalize";
import type { AlignedWordResult, Anchor, CanonicalSentence, SentenceWindow } from "./types";
import type { AlignmentWord, NarrationAlignment } from "../../../src/components/SpeakAloud/types";

export interface ScoredSentence {
  ok: boolean;
  words: AlignmentWord[];
  reasons: string[];
  meanConfidence: number;
  lowConfidence: number;
  anchorMedianError: number | null;
}

const median = (values: number[]) => {
  const sorted = [...values].sort((a, b) => a - b);
  if (!sorted.length) return 0;
  const middle = Math.floor(sorted.length / 2);
  return sorted.length % 2 ? sorted[middle] : (sorted[middle - 1] + sorted[middle]) / 2;
};

const mapAlignedWords = (
  sentence: CanonicalSentence,
  aligned: AlignedWordResult[],
): AlignmentWord[] | null => {
  const usable = aligned.filter((word) => normalizeToken(word.text));
  if (usable.length !== sentence.words.length) return null;
  for (let index = 0; index < usable.length; index += 1) {
    if (normalizeToken(usable[index].text) !== sentence.words[index].norm) return null;
  }
  return usable.map((word, index) => [
    sentence.words[index].charStart,
    sentence.words[index].charEnd,
    word.start,
    word.end,
    word.probability,
  ]);
};

const constrainUnanchoredWords = (
  words: AlignmentWord[],
  sentence: CanonicalSentence,
  anchors: Map<number, Anchor>,
  window: SentenceWindow,
) => {
  let cursor = 0;
  while (cursor < words.length) {
    if (anchors.get(sentence.words[cursor].globalIndex)?.kind === "exact") {
      cursor += 1;
      continue;
    }
    const start = cursor;
    while (
      cursor < words.length &&
      anchors.get(sentence.words[cursor].globalIndex)?.kind !== "exact"
    ) {
      cursor += 1;
    }
    const end = cursor;
    const lower = start > 0 ? words[start - 1][3] : window.start;
    const upper = end < words.length ? words[end][2] : window.end;
    const invalid = words
      .slice(start, end)
      .some(
        (word, index, run) =>
          word[3] <= word[2] ||
          word[2] < window.start ||
          word[3] > window.end ||
          (index > 0 && word[2] < run[index - 1][3]),
      );
    if (!invalid) continue;
    let repairStart = start;
    let repairEnd = end;
    let repairLower = lower;
    let repairUpper = upper;
    if (repairUpper - repairLower < (repairEnd - repairStart) * 0.01) {
      repairStart = Math.max(0, start - 1);
      repairEnd = Math.min(words.length, end + 1);
      repairLower = Math.max(window.start, words[repairStart][2]);
      repairUpper = Math.min(window.end, words[repairEnd - 1][3]);
    }
    if (repairUpper - repairLower < (repairEnd - repairStart) * 0.01) return false;
    const weights = words
      .slice(repairStart, repairEnd)
      .map((word) => Math.max(1, word[1] - word[0]));
    const totalWeight = weights.reduce((sum, weight) => sum + weight, 0);
    let time = repairLower;
    for (let index = repairStart; index < repairEnd; index += 1) {
      const duration = ((repairUpper - repairLower) * weights[index - repairStart]) / totalWeight;
      words[index][2] = time;
      words[index][3] = time + duration;
      time += duration;
    }
  }
  return true;
};

const repairDurations = (words: AlignmentWord[], windowStart: number, windowEnd: number) => {
  let cursor = 0;
  while (cursor < words.length) {
    const sharedStart = words[cursor][2];
    let end = cursor + 1;
    while (end < words.length && words[end][2] <= sharedStart + 0.03) end += 1;
    const needsRepair =
      end - cursor > 1 || words.slice(cursor, end).some((word) => word[3] <= word[2]);
    if (needsRepair) {
      const lower = cursor > 0 ? words[cursor - 1][3] : Math.max(windowStart, sharedStart);
      const groupEnd = Math.max(...words.slice(cursor, end).map((word) => word[3]));
      const naturalEnd =
        groupEnd > lower
          ? groupEnd
          : Math.min(windowEnd, lower + Math.max(0.2, (end - cursor) * 0.08));
      const upper = Math.max(groupEnd, end < words.length ? words[end][2] : naturalEnd);
      const available = upper - lower;
      if (available < (end - cursor) * 0.01) return false;
      const weights = words.slice(cursor, end).map((word) => Math.max(1, word[1] - word[0]));
      const totalWeight = weights.reduce((sum, weight) => sum + weight, 0);
      let time = lower;
      for (let index = cursor; index < end; index += 1) {
        const duration = (available * weights[index - cursor]) / totalWeight;
        words[index][2] = time;
        words[index][3] = time + duration;
        time += duration;
      }
    }
    cursor = end;
  }

  for (let index = 0; index < words.length - 1; index += 1) {
    if (words[index][3] > words[index + 1][2]) {
      const boundary = (words[index][3] + words[index + 1][2]) / 2;
      if (boundary <= words[index][2] || boundary >= words[index + 1][3]) return false;
      words[index][3] = boundary;
      words[index + 1][2] = boundary;
    }
  }
  return words.every((word) => word[3] > word[2]);
};

export const scoreSentence = (
  sentence: CanonicalSentence,
  window: SentenceWindow,
  aligned: AlignedWordResult[],
  anchors: Anchor[],
): ScoredSentence => {
  const reasons: string[] = [];
  const words = mapAlignedWords(sentence, aligned);
  if (!words) {
    return {
      ok: false,
      words: [],
      reasons: ["forced aligner did not reproduce the canonical word sequence"],
      meanConfidence: 0,
      lowConfidence: 1,
      anchorMedianError: null,
    };
  }

  const usableAnchors = anchors.filter(
    (anchor) =>
      anchor.kind === "exact" &&
      (anchor.reliable || (anchor.start >= window.start && anchor.end <= window.end)),
  );
  const anchorByCanonical = new Map(usableAnchors.map((anchor) => [anchor.canonicalIndex, anchor]));
  const errors: number[] = [];
  for (let index = 0; index < sentence.words.length; index += 1) {
    const anchor = anchorByCanonical.get(sentence.words[index].globalIndex);
    if (!anchor || anchor.kind !== "exact") continue;
    errors.push(Math.abs(words[index][2] - anchor.start));
    // ASR timestamps are independent evidence and are more reliable for words
    // it recognized exactly than a forced result pinned to a window edge.
    words[index][2] = anchor.start;
    words[index][3] = anchor.end;
    words[index][4] = Math.max(words[index][4], anchor.probability);
  }

  if (!constrainUnanchoredWords(words, sentence, anchorByCanonical, window))
    reasons.push("unanchored words could not be constrained");
  if (!repairDurations(words, window.start, window.end))
    reasons.push("zero-duration words could not be repaired");
  if (words.some((word) => !word.slice(2, 5).every(Number.isFinite))) {
    reasons.push("non-finite word timing");
  }
  if (words.some((word) => word[3] <= word[2])) reasons.push("zero-duration word");
  for (let index = 1; index < words.length; index += 1) {
    if (words[index][2] < words[index - 1][3] - 0.05) {
      reasons.push("non-monotonic words");
      break;
    }
  }

  let collapsed = 0;
  for (const word of words) {
    collapsed = word[3] - word[2] < 0.06 ? collapsed + 1 : 0;
    if (collapsed >= 3) {
      reasons.push("collapsed word run");
      break;
    }
  }

  const duration = words.at(-1)![3] - words[0][2];
  const rate = words.length / Math.max(duration, 0.001);
  const maximumRate = words.length <= 10 ? 10 : 8.5;
  if (rate < 0.7 || rate > maximumRate)
    reasons.push(`implausible speech rate ${rate.toFixed(2)} words/s`);

  const meanConfidence = words.reduce((sum, word) => sum + word[4], 0) / words.length;
  const lowConfidenceCount = words.filter((word) => word[4] < 0.3).length;
  const lowConfidence = lowConfidenceCount / words.length;
  if (meanConfidence < (window.interpolated ? 0.4 : 0.3)) {
    reasons.push(`low mean confidence ${meanConfidence.toFixed(2)}`);
  }
  if (lowConfidenceCount >= 3 && lowConfidence > 0.45)
    reasons.push(`too many low-confidence words ${lowConfidence.toFixed(2)}`);

  const anchorMedianError = errors.length ? median(errors) : null;
  const exactAnchorCoverage = usableAnchors.length / sentence.words.length;
  if (errors.length >= 2 && exactAnchorCoverage < 0.6 && anchorMedianError! > 0.75) {
    reasons.push(`ASR disagreement ${anchorMedianError!.toFixed(2)}s`);
  }
  if (words[0][2] < window.start - 0.01 || words.at(-1)![3] > window.end + 0.01) {
    reasons.push("timing escaped sentence window");
  }

  return {
    ok: reasons.length === 0,
    words,
    reasons,
    meanConfidence,
    lowConfidence,
    anchorMedianError,
  };
};

export const validateArtifactTimings = (alignment: NarrationAlignment) => {
  const errors: string[] = [];
  let previousSentenceEnd = -Infinity;
  for (const block of alignment.blocks) {
    if (!block.spoken) {
      if (block.sentences.length) errors.push(`unspoken block ${block.index} contains sentences`);
      continue;
    }
    if (!block.sentences.length) errors.push(`spoken block ${block.index} has no sentences`);
    for (const [sentenceIndex, sentence] of block.sentences.entries()) {
      if (!(sentence.start < sentence.end)) {
        errors.push(`block ${block.index} sentence ${sentenceIndex} has invalid bounds`);
      }
      if (sentence.start < previousSentenceEnd - 0.05) {
        errors.push(`block ${block.index} sentence ${sentenceIndex} overlaps its predecessor`);
      }
      previousSentenceEnd = sentence.end;
      let previousWordEnd = -Infinity;
      let collapsedRun = 0;
      for (const [wordIndex, word] of sentence.words.entries()) {
        if (!word.slice(2, 5).every(Number.isFinite) || word[3] <= word[2]) {
          errors.push(
            `block ${block.index} sentence ${sentenceIndex} word ${wordIndex} is invalid`,
          );
        }
        if (word[2] < previousWordEnd - 0.05) {
          errors.push(
            `block ${block.index} sentence ${sentenceIndex} word ${wordIndex} overlaps its predecessor`,
          );
        }
        previousWordEnd = word[3];
        collapsedRun = word[3] - word[2] < 0.06 ? collapsedRun + 1 : 0;
        if (collapsedRun === 3) {
          errors.push(`block ${block.index} sentence ${sentenceIndex} has a collapsed word run`);
        }
        if (word[2] < -0.01 || word[3] > alignment.audio.duration + 0.5) {
          errors.push(
            `block ${block.index} sentence ${sentenceIndex} word ${wordIndex} is outside audio`,
          );
        }
      }
    }
  }
  return errors;
};
