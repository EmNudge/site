import { describe, expect, it } from "vitest";
import { buildCanonicalSentences } from "../scripts/narration/lib/pipeline";
import { scoreSentence, validateArtifactTimings } from "../scripts/narration/lib/quality";
import type { Anchor, SentenceWindow } from "../scripts/narration/lib/types";
import type { NarrationAlignment } from "../src/components/SpeakAloud/types";

const fixture = () => {
  const { sentences } = buildCanonicalSentences([
    { index: 0, tag: "p", text: "Kind does not mean naive." },
  ]);
  const sentence = sentences[0];
  const window: SentenceWindow = {
    id: sentence.id,
    blockIndex: 0,
    sentenceIndex: 0,
    start: 1,
    end: 4,
    anchors: [],
    interpolated: false,
    level: 1,
  };
  return { sentence, window };
};

describe("sentence alignment quality", () => {
  it("uses exact ASR anchors to repair collapsed forced words", () => {
    const { sentence, window } = fixture();
    const anchors: Anchor[] = sentence.words.map((word, index) => ({
      canonicalIndex: word.globalIndex,
      asrIndex: index,
      start: 1 + index * 0.4,
      end: 1.3 + index * 0.4,
      probability: 0.9,
      reliable: true,
      kind: "exact",
    }));
    const scored = scoreSentence(
      sentence,
      window,
      sentence.words.map((word) => ({
        text: word.text,
        start: 2,
        end: 2,
        probability: 0.1,
      })),
      anchors,
    );
    expect(scored.ok).toBe(true);
    expect(scored.words.every((word) => word[3] > word[2])).toBe(true);
    expect(scored.words.map((word) => word[2])).toEqual(anchors.map((anchor) => anchor.start));
  });

  it("gives a lone zero-duration ASR word a bounded duration", () => {
    const { sentences } = buildCanonicalSentences([{ index: 0, tag: "p", text: "Consciousness?" }]);
    const sentence = sentences[0];
    const window: SentenceWindow = {
      id: sentence.id,
      blockIndex: 0,
      sentenceIndex: 0,
      start: 4,
      end: 5,
      anchors: [],
      interpolated: false,
      level: 1,
    };
    const scored = scoreSentence(
      sentence,
      window,
      [{ text: "Consciousness", start: 4.5, end: 4.5, probability: 0.9 }],
      [
        {
          canonicalIndex: sentence.words[0].globalIndex,
          asrIndex: 0,
          start: 4.5,
          end: 4.5,
          probability: 0.9,
          reliable: true,
          kind: "exact",
        },
      ],
    );
    expect(scored.ok).toBe(true);
    expect(scored.words[0][3]).toBeGreaterThan(scored.words[0][2]);
    expect(scored.words[0][3]).toBeLessThanOrEqual(5);
  });

  it("ignores an isolated repeated-word anchor outside the sentence window", () => {
    const { sentences } = buildCanonicalSentences([{ index: 0, tag: "p", text: "And it." }]);
    const sentence = sentences[0];
    const window: SentenceWindow = {
      id: sentence.id,
      blockIndex: 0,
      sentenceIndex: 0,
      start: 1,
      end: 2,
      anchors: [],
      interpolated: false,
      level: 1,
    };
    const scored = scoreSentence(
      sentence,
      window,
      [
        { text: "And", start: 1.2, end: 1.4, probability: 0.9 },
        { text: "it", start: 1.4, end: 1.6, probability: 0.9 },
      ],
      [
        {
          canonicalIndex: sentence.words[1].globalIndex,
          asrIndex: 20,
          start: 5,
          end: 5.1,
          probability: 0.9,
          reliable: false,
          kind: "exact",
        },
      ],
    );
    expect(scored.ok).toBe(true);
    expect(scored.words[1].slice(2, 4)).toEqual([1.4, 1.6]);
  });

  it("rejects sentence overlap in committed artifacts", () => {
    const alignment = {
      version: 1,
      recording: "fixture",
      generatedAt: "",
      article: { source: "", textHash: "", blockCount: 1 },
      audio: { file: "", sha256: "", duration: 5 },
      aligner: { name: "stable-ts", version: "", model: "" },
      blocks: [
        {
          index: 0,
          spoken: true,
          hash: "",
          text: "One. Two.",
          start: 1,
          end: 3,
          sentences: [
            { charStart: 0, charEnd: 4, start: 1, end: 2.2, words: [[0, 4, 1, 2.2, 1]] },
            { charStart: 5, charEnd: 9, start: 2, end: 3, words: [[5, 9, 2, 3, 1]] },
          ],
        },
      ],
    } satisfies NarrationAlignment;
    expect(validateArtifactTimings(alignment)).toContain(
      "block 0 sentence 1 overlaps its predecessor",
    );
  });
});
