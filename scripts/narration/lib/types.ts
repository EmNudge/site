export interface AsrWord {
  text: string;
  start: number;
  end: number;
  probability: number;
}

export interface CanonicalWord {
  text: string;
  norm: string;
  charStart: number;
  charEnd: number;
  blockIndex: number;
  sentenceIndex: number;
  wordIndex: number;
  globalIndex: number;
}

export interface CanonicalSentence {
  id: string;
  text: string;
  charStart: number;
  charEnd: number;
  blockIndex: number;
  sentenceIndex: number;
  words: CanonicalWord[];
}

export interface Anchor {
  canonicalIndex: number;
  asrIndex: number;
  start: number;
  end: number;
  probability: number;
  reliable: boolean;
  kind: "exact" | "fuzzy";
}

export interface SentenceWindow {
  id: string;
  blockIndex: number;
  sentenceIndex: number;
  start: number;
  end: number;
  anchors: Anchor[];
  interpolated: boolean;
  level: number;
}

export interface AlignedWordResult {
  text: string;
  start: number;
  end: number;
  probability: number;
}

export interface SentenceAlignmentResult {
  id: string;
  ok: boolean;
  words: AlignedWordResult[];
  error?: string;
}

export interface AlignmentJob {
  id: string;
  words: string[];
  windowStart: number;
  windowEnd: number;
}
