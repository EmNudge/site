const SMALL_NUMBERS = [
  "zero",
  "one",
  "two",
  "three",
  "four",
  "five",
  "six",
  "seven",
  "eight",
  "nine",
  "ten",
  "eleven",
  "twelve",
  "thirteen",
  "fourteen",
  "fifteen",
  "sixteen",
  "seventeen",
  "eighteen",
  "nineteen",
  "twenty",
] as const;

const baseNormalize = (value: string) =>
  value
    .normalize("NFKD")
    .replace(/\p{M}/gu, "")
    .toLocaleLowerCase()
    .replace(/[’']/g, "")
    .replace(/[^\p{L}\p{N}]+/gu, "");

const numberWords = (value: number): string | null => {
  if (value >= 0 && value <= 20) return SMALL_NUMBERS[value];
  if (value > 20 && value < 100) {
    const tens = [
      "",
      "",
      "twenty",
      "thirty",
      "forty",
      "fifty",
      "sixty",
      "seventy",
      "eighty",
      "ninety",
    ];
    return `${tens[Math.floor(value / 10)]}${value % 10 ? SMALL_NUMBERS[value % 10] : ""}`;
  }
  if (value >= 2000 && value <= 2099) {
    const tail = value - 2000;
    return `twenty${tail ? numberWords(tail) : ""}`;
  }
  return null;
};

export const normalizeVariants = (value: string): string[] => {
  const normalized = baseNormalize(value);
  if (!normalized) return [];
  const variants = new Set([normalized]);
  if (/^\d+$/.test(normalized)) {
    const words = numberWords(Number(normalized));
    if (words) variants.add(words);
  }
  return [...variants];
};

export const normalizeToken = (value: string) => normalizeVariants(value)[0] ?? "";

export const levenshteinDistance = (a: string, b: string) => {
  if (a === b) return 0;
  if (!a.length) return b.length;
  if (!b.length) return a.length;
  let previous = new Uint16Array(b.length + 1);
  let current = new Uint16Array(b.length + 1);
  for (let j = 0; j <= b.length; j += 1) previous[j] = j;
  for (let i = 1; i <= a.length; i += 1) {
    current[0] = i;
    for (let j = 1; j <= b.length; j += 1) {
      current[j] = Math.min(
        current[j - 1] + 1,
        previous[j] + 1,
        previous[j - 1] + (a[i - 1] === b[j - 1] ? 0 : 1),
      );
    }
    [previous, current] = [current, previous];
  }
  return previous[b.length];
};

export const similarity = (a: string, b: string) =>
  1 - levenshteinDistance(a, b) / Math.max(a.length, b.length, 1);

export const bestSimilarity = (left: string[], right: string[]) => {
  let best = 0;
  for (const a of left) {
    for (const b of right) best = Math.max(best, similarity(a, b));
  }
  return best;
};

export const variantsMatch = (left: string[], right: string[]) =>
  left.some((value) => right.includes(value));
