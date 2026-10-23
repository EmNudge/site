import { describe, expect, it } from "vitest";
import { buildCanonicalSentences } from "../scripts/narration/lib/pipeline";
import { deriveSentenceWindows } from "../scripts/narration/lib/windows";
import type { Anchor } from "../scripts/narration/lib/types";

const anchorsFor = (indices: number[], start: number, step = 0.3): Anchor[] =>
  indices.map((canonicalIndex, index) => ({
    canonicalIndex,
    asrIndex: index,
    start: start + index * step,
    end: start + index * step + 0.2,
    probability: 0.9,
    reliable: true,
    kind: "exact",
  }));

describe("sentence window derivation", () => {
  it("extrapolates before a late first anchor", () => {
    const { sentences } = buildCanonicalSentences([
      { index: 0, tag: "p", text: "You can put your cans neatly in the recycling bin." },
    ]);
    const lastThree = sentences[0].words.slice(-3).map((word) => word.globalIndex);
    const result = deriveSentenceWindows(sentences, anchorsFor(lastThree, 10), 20);
    expect(result.errors).toEqual([]);
    expect(result.windows[0].start).toBeLessThan(9);
    expect(result.windows[0].end).toBeGreaterThan(10.7);
  });

  it("infers an unread leading block when there is no room to speak it", () => {
    const { sentences } = buildCanonicalSentences([
      {
        index: 0,
        tag: "blockquote",
        text: "This long opening quotation is displayed but is never spoken in the recording.",
      },
      { index: 1, tag: "p", text: "The actual narration starts right here with these words." },
    ]);
    const spoken = sentences[1].words.map((word) => word.globalIndex);
    const result = deriveSentenceWindows(sentences, anchorsFor(spoken, 0.2, 0.15), 8);
    expect(result.unspokenBlocks.has(0)).toBe(true);
    expect(result.windows.some((window) => window.blockIndex === 0)).toBe(false);
    expect(result.windows.some((window) => window.blockIndex === 1)).toBe(true);
  });

  it("infers an unspoken attribution suffix from the next anchor", () => {
    const { sentences } = buildCanonicalSentences([
      {
        index: 0,
        tag: "blockquote",
        text: "Then it is correct even if wrong Finneas Tape Notes Podcast",
      },
      { index: 1, tag: "p", text: "When the next spoken paragraph starts here." },
    ]);
    const quoteWords = sentences[0].words;
    const spokenQuote = quoteWords.slice(0, 7).map((word) => word.globalIndex);
    const nextWords = sentences[1].words.map((word) => word.globalIndex);
    const anchors = [
      ...anchorsFor(spokenQuote, 0, 0.2),
      ...anchorsFor(nextWords, 1.7, 0.2).map((anchor, index) => ({
        ...anchor,
        asrIndex: spokenQuote.length + index,
      })),
    ];
    const result = deriveSentenceWindows(sentences, anchors, 5);
    expect(quoteWords.slice(7).every((word) => result.unspokenWords.has(word.globalIndex))).toBe(
      true,
    );
    expect(quoteWords.slice(0, 7).some((word) => result.unspokenWords.has(word.globalIndex))).toBe(
      false,
    );
  });

  it("keeps derived windows inside the recording", () => {
    const { sentences } = buildCanonicalSentences([
      { index: 0, tag: "p", text: "First sentence. Second sentence. Third sentence." },
    ]);
    const indices = sentences.flatMap((sentence) => sentence.words.map((word) => word.globalIndex));
    const result = deriveSentenceWindows(sentences, anchorsFor(indices, 0.1, 0.25), 4);
    expect(result.windows.every((window) => window.start >= 0 && window.end <= 4)).toBe(true);
    expect(result.windows.every((window) => window.end - window.start <= 28)).toBe(true);
  });
});
