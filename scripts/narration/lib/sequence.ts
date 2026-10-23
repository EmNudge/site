import { bestSimilarity, normalizeVariants, variantsMatch } from "./normalize";

export interface SequenceWord {
  text: string;
  variants?: string[];
}

export interface SequenceMatch {
  canonicalIndex: number;
  asrIndex: number;
  kind: "exact" | "fuzzy";
  reliable: boolean;
}

const GAP = -1;
const MISMATCH = -2;
const DIAGONAL = 1;
const UP = 2;
const LEFT = 3;

const variants = (word: SequenceWord) => word.variants ?? normalizeVariants(word.text);

const scorePair = (canonical: SequenceWord, asr: SequenceWord) => {
  const left = variants(canonical);
  const right = variants(asr);
  if (!left.length || !right.length) return { score: MISMATCH, kind: null } as const;
  if (variantsMatch(left, right)) {
    const length = Math.max(...left.map((value) => value.length));
    return { score: length <= 2 ? 2 : 3, kind: "exact" } as const;
  }
  const similarity = bestSimilarity(left, right);
  if (Math.max(left[0].length, right[0].length) >= 4 && similarity >= 0.75) {
    return { score: 1, kind: "fuzzy" } as const;
  }
  return { score: MISMATCH, kind: null } as const;
};

export const alignSequences = (canonical: SequenceWord[], asr: SequenceWord[]): SequenceMatch[] => {
  const width = asr.length + 1;
  const directions = new Uint8Array((canonical.length + 1) * width);
  let previous = new Int32Array(width);
  let current = new Int32Array(width);
  for (let j = 1; j < width; j += 1) {
    previous[j] = previous[j - 1] + GAP;
    directions[j] = LEFT;
  }

  for (let i = 1; i <= canonical.length; i += 1) {
    current[0] = previous[0] + GAP;
    directions[i * width] = UP;
    for (let j = 1; j <= asr.length; j += 1) {
      const pair = scorePair(canonical[i - 1], asr[j - 1]);
      const diagonal = previous[j - 1] + pair.score;
      const up = previous[j] + GAP;
      const left = current[j - 1] + GAP;
      // Prefer skipping an ASR token on score ties. This keeps the earliest
      // complete take when a recording accidentally repeats a passage, while
      // surrounding high-score diagonals still disambiguate repeated words.
      if (left >= diagonal && left >= up) {
        current[j] = left;
        directions[i * width + j] = LEFT;
      } else if (diagonal >= up) {
        current[j] = diagonal;
        directions[i * width + j] = DIAGONAL;
      } else {
        current[j] = up;
        directions[i * width + j] = UP;
      }
    }
    [previous, current] = [current, previous];
  }

  const matches: SequenceMatch[] = [];
  let i = canonical.length;
  let j = asr.length;
  while (i > 0 || j > 0) {
    const direction = directions[i * width + j];
    if (direction === DIAGONAL) {
      const pair = scorePair(canonical[i - 1], asr[j - 1]);
      if (pair.kind) {
        matches.push({
          canonicalIndex: i - 1,
          asrIndex: j - 1,
          kind: pair.kind,
          reliable: false,
        });
      }
      i -= 1;
      j -= 1;
    } else if (direction === UP) {
      i -= 1;
    } else if (direction === LEFT) {
      j -= 1;
    } else {
      // Only reachable at an empty matrix edge.
      if (i > 0) i -= 1;
      else j -= 1;
    }
  }
  matches.reverse();

  let runStart = 0;
  while (runStart < matches.length) {
    let runEnd = runStart + 1;
    while (
      runEnd < matches.length &&
      matches[runEnd].canonicalIndex === matches[runEnd - 1].canonicalIndex + 1 &&
      matches[runEnd].asrIndex === matches[runEnd - 1].asrIndex + 1
    ) {
      runEnd += 1;
    }
    const run = matches.slice(runStart, runEnd);
    const exact = run.filter((match) => match.kind === "exact").length;
    if (run.length >= 3 && exact >= 2) {
      for (let index = runStart; index < runEnd; index += 1) matches[index].reliable = true;
    } else if (
      run.length === 1 &&
      run[0].kind === "exact" &&
      variants(canonical[run[0].canonicalIndex])[0]?.length >= 6
    ) {
      matches[runStart].reliable = true;
    }
    runStart = runEnd;
  }

  return matches;
};
