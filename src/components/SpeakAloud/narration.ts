import type { AlignmentBlock } from "./types";

export const NARRATION_SELECTOR = "article > p, article > blockquote";
const EXCLUDED_TAGS = new Set(["SCRIPT", "STYLE", "SUP"]);

export interface TextMapEntry {
  node: Text;
  start: number;
  end: number;
}

export interface CanonicalText {
  text: string;
  map: TextMapEntry[];
}

function isExcluded(node: Node, block: HTMLElement): boolean {
  let parent = node.parentElement;
  while (parent && parent !== block) {
    if (
      EXCLUDED_TAGS.has(parent.tagName) ||
      parent.classList.contains("header-link") ||
      parent.hasAttribute("data-footnote-ref") ||
      parent.getAttribute("aria-hidden") === "true"
    ) {
      return true;
    }
    parent = parent.parentElement;
  }
  return false;
}

/**
 * Convert rendered article text to the representation used by the aligner.
 * Whitespace is deliberately normalized here and in the build-time extractor;
 * the map retains the original DOM offsets so a Range can be painted later.
 */
export const getCanonicalText = (block: HTMLElement): CanonicalText => {
  const text = [] as string[];
  const map: TextMapEntry[] = [];
  const walker = document.createTreeWalker(block, 4); // NodeFilter.SHOW_TEXT
  let node: Node | null;

  while ((node = walker.nextNode())) {
    if (!(node instanceof Text) || isExcluded(node, block)) continue;
    const value = node.nodeValue ?? "";
    for (let offset = 0; offset < value.length; offset += 1) {
      const character = value[offset];
      if (/\s/.test(character)) {
        if (text.length > 0 && text[text.length - 1] !== " ") {
          text.push(" ");
          map.push({ node, start: offset, end: offset + 1 });
        }
      } else {
        text.push(character);
        map.push({ node, start: offset, end: offset + 1 });
      }
    }
  }

  while (text[text.length - 1] === " ") {
    text.pop();
    map.pop();
  }

  return { text: text.join(""), map };
};

export const getNarrationBlocks = (root: ParentNode = document): HTMLElement[] =>
  Array.from(root.querySelectorAll<HTMLElement>(NARRATION_SELECTOR));

export const splitSentences = (text: string): { start: number; end: number; text: string }[] => {
  if (!text) return [];
  if (typeof Intl.Segmenter === "undefined") {
    const result: { start: number; end: number; text: string }[] = [];
    const expression = /[^.!?]+(?:[.!?]+|$)/g;
    for (const match of text.matchAll(expression)) {
      const start = match.index ?? 0;
      result.push({ start, end: start + match[0].length, text: match[0] });
    }
    return result;
  }

  const segmenter = new Intl.Segmenter("en", { granularity: "sentence" });
  return Array.from(segmenter.segment(text), (segment) => ({
    start: segment.index,
    end: segment.index + segment.segment.length,
    text: segment.segment,
  }));
};

export const splitWords = (text: string): { start: number; end: number; text: string }[] => {
  const words: { start: number; end: number; text: string }[] = [];
  for (const match of text.matchAll(/\S+/g)) {
    const start = match.index ?? 0;
    words.push({ start, end: start + match[0].length, text: match[0] });
  }
  return words;
};

export const makeRange = (map: TextMapEntry[], start: number, end: number): Range | null => {
  if (start >= end || start < 0 || end > map.length) return null;
  const first = map[start];
  const last = map[end - 1];
  const range = document.createRange();
  range.setStart(first.node, first.start);
  range.setEnd(last.node, last.end);
  return range;
};

export const textMatchesBlock = (block: HTMLElement, expected: string): boolean =>
  getCanonicalText(block).text === expected;

export const getCanonicalOffset = (
  map: TextMapEntry[],
  node: Node,
  offset: number,
): number | null => {
  const exact = map.filter((entry) => entry.node === node);
  if (exact.length) {
    const first = map.indexOf(exact[0]);
    const last = first + exact.length;
    for (let index = first; index < last; index += 1) {
      if (offset < map[index].end) return index;
    }
    return last;
  }

  // caretPositionFromPoint can return an element node at a boundary rather
  // than the text node under the pointer. Find the nearest mapped descendant.
  const descendant = map.findIndex((entry) => node.contains(entry.node));
  return descendant >= 0 ? descendant : null;
};

export const hashText = async (text: string): Promise<string> => {
  const bytes = new TextEncoder().encode(text);
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  return `sha256:${Array.from(new Uint8Array(digest), (byte) => byte.toString(16).padStart(2, "0")).join("")}`;
};

export const getAlignmentText = (blocks: AlignmentBlock[]): string =>
  blocks.map((block) => block.text).join("\n\n");
