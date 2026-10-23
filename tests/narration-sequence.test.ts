import { describe, expect, it } from "vitest";
import { normalizeVariants } from "../scripts/narration/lib/normalize";
import { alignSequences } from "../scripts/narration/lib/sequence";

const words = (values: string[]) =>
  values.map((text) => ({ text, variants: normalizeVariants(text) }));

describe("canonical/ASR sequence alignment", () => {
  it("uses phrase context to disambiguate repeated words", () => {
    const canonical = words([
      "you’re",
      "kind",
      "Kind",
      "doesn’t",
      "mean",
      "naive",
      "it",
      "means",
      "kind",
      "Do",
      "unto",
    ]);
    const asr = words([
      "you’re",
      "kind",
      "Kind",
      "doesn’t",
      "mean",
      "naive",
      "it",
      "means",
      "kind",
      "Do",
      "unto",
    ]);
    const matches = alignSequences(canonical, asr);
    expect(matches.find((match) => match.canonicalIndex === 2)?.asrIndex).toBe(2);
    expect(matches.find((match) => match.canonicalIndex === 8)?.asrIndex).toBe(8);
    expect(matches.filter((match) => match.reliable)).toHaveLength(canonical.length);
  });

  it("stays monotonic through ASR insertions and deletions", () => {
    const canonical = words(["one", "two", "three", "four", "five", "six"]);
    const asr = words(["one", "extra", "two", "four", "five", "six"]);
    const matches = alignSequences(canonical, asr);
    for (let index = 1; index < matches.length; index += 1) {
      expect(matches[index].canonicalIndex).toBeGreaterThan(matches[index - 1].canonicalIndex);
      expect(matches[index].asrIndex).toBeGreaterThan(matches[index - 1].asrIndex);
    }
    expect(matches.some((match) => match.canonicalIndex === 3 && match.asrIndex === 3)).toBe(true);
  });

  it("selects the earliest complete take when audio repeats a passage", () => {
    const canonical = words(["alpha", "beta", "gamma", "delta", "next"]);
    const asr = words([
      "alpha",
      "beta",
      "gamma",
      "delta",
      "alpha",
      "beta",
      "gamma",
      "delta",
      "next",
    ]);
    expect(alignSequences(canonical, asr).map((match) => match.asrIndex)).toEqual([0, 1, 2, 3, 8]);
  });

  it("matches numeric and spoken forms", () => {
    const matches = alignSequences(
      words(["I", "got", "2", "votes"]),
      words(["I", "got", "two", "votes"]),
    );
    expect(matches.some((match) => match.canonicalIndex === 2 && match.asrIndex === 2)).toBe(true);
  });
});
