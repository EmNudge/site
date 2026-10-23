export type NarrationGranularity = "paragraph" | "sentence" | "word";

export type AlignmentWord = [
  charStart: number,
  charEnd: number,
  start: number,
  end: number,
  confidence: number,
];

export interface AlignmentSentence {
  charStart: number;
  charEnd: number;
  start: number;
  end: number;
  words: AlignmentWord[];
}

export interface AlignmentBlock {
  index: number;
  spoken: boolean;
  hash: string;
  text: string;
  start: number;
  end: number;
  sentences: AlignmentSentence[];
}

export interface NarrationAlignment {
  version: 1;
  recording: string;
  generatedAt: string;
  article: {
    source: string;
    textHash: string;
    blockCount: number;
  };
  audio: {
    file: string;
    sha256: string;
    duration: number;
  };
  aligner: {
    name: "stable-ts";
    version: string;
    model: string;
  };
  blocks: AlignmentBlock[];
}

export interface TimelineUnit {
  blockIndex: number;
  charStart: number;
  charEnd: number;
  start: number;
  end: number;
  sentenceIndex?: number;
  wordIndex?: number;
}

export interface NarrationTimeline {
  alignment: NarrationAlignment | null;
  units: Record<NarrationGranularity, TimelineUnit[]>;
  availableGranularities: NarrationGranularity[];
  legacyTimestamps: number[] | null;
}
