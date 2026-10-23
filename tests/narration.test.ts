import { describe, expect, it, beforeEach } from "vitest";
import { JSDOM } from "jsdom";
import {
  getCanonicalOffset,
  getCanonicalText,
  makeRange,
  splitSentences,
  splitWords,
} from "../src/components/SpeakAloud/narration";
import {
  getAlignmentUnits,
  getParagraphIndex,
  getTimelineUnitIndex,
} from "../src/components/SpeakAloud/utils";
import type { NarrationAlignment } from "../src/components/SpeakAloud/types";

beforeEach(() => {
  const { window } = new JSDOM("<html><body></body></html>");
  Object.assign(globalThis, { document: window.document, Text: window.Text });
});

describe("narration text mapping", () => {
  it("normalizes rendered whitespace without losing inline text", () => {
    const block = document.createElement("p");
    block.innerHTML =
      'Hello <em>there</em>\n<a href="/x">friend</a><sup data-footnote-ref>1</sup>.';
    const canonical = getCanonicalText(block);
    expect(canonical.text).toBe("Hello there friend.");
    expect(canonical.map).toHaveLength(canonical.text.length);
  });

  it("creates ranges over multiple inline text nodes", () => {
    const block = document.createElement("p");
    block.innerHTML = "Hello <strong>world</strong>!";
    const canonical = getCanonicalText(block);
    const range = makeRange(canonical.map, 6, 11);
    expect(range?.toString()).toBe("world");
    expect(getCanonicalOffset(canonical.map, canonical.map[7].node, 2)).toBe(8);
  });

  it("splits sentences and words with reconstructable offsets", () => {
    const text = "First sentence. Is this the second? Yes!";
    const sentences = splitSentences(text);
    expect(sentences.map((sentence) => text.slice(sentence.start, sentence.end)).join("")).toBe(
      text,
    );
    expect(splitWords(text).map((word) => word.text)).toEqual([
      "First",
      "sentence.",
      "Is",
      "this",
      "the",
      "second?",
      "Yes!",
    ]);
  });
});

describe("narration timeline lookup", () => {
  it("selects the last unit after the final timestamp", () => {
    expect(getParagraphIndex(0, [0, 6, 12])).toBe(0);
    expect(getParagraphIndex(6, [0, 6, 12])).toBe(1);
    expect(getParagraphIndex(99, [0, 6, 12])).toBe(2);
  });

  it("coalesces words that share an indistinguishable timestamp", () => {
    const alignment = {
      version: 1,
      recording: "fixture",
      generatedAt: "",
      article: { source: "", textHash: "", blockCount: 1 },
      audio: { file: "", sha256: "", duration: 2 },
      aligner: { name: "stable-ts", version: "", model: "" },
      blocks: [
        {
          index: 0,
          spoken: true,
          hash: "",
          text: "one two three",
          start: 0,
          end: 1,
          sentences: [
            {
              charStart: 0,
              charEnd: 13,
              start: 0,
              end: 1,
              words: [
                [0, 3, 0, 0, 0.1],
                [4, 7, 0, 0.5, 0.8],
                [8, 13, 0.5, 1, 0.9],
              ],
            },
          ],
        },
      ],
    } satisfies NarrationAlignment;

    expect(getAlignmentUnits(alignment, "word")).toEqual([
      {
        blockIndex: 0,
        charStart: 0,
        charEnd: 7,
        start: 0,
        end: 0.5,
        sentenceIndex: 0,
        wordIndex: 0,
      },
      {
        blockIndex: 0,
        charStart: 8,
        charEnd: 13,
        start: 0.5,
        end: 1,
        sentenceIndex: 0,
        wordIndex: 2,
      },
    ]);
  });

  it("handles silence before and between fine-grained units", () => {
    const units = [
      { blockIndex: 0, charStart: 0, charEnd: 2, start: 2, end: 4 },
      { blockIndex: 0, charStart: 3, charEnd: 7, start: 8, end: 10 },
    ];
    expect(getTimelineUnitIndex(0, units)).toBe(0);
    expect(getTimelineUnitIndex(5, units)).toBe(0);
    expect(getTimelineUnitIndex(9, units)).toBe(1);
  });
});
