import { getCanonicalText, getAlignmentText, hashText } from "./narration";
import type {
  NarrationAlignment,
  NarrationGranularity,
  NarrationTimeline,
  TimelineUnit,
} from "./types";

const getTimestamps = async (baseUrl: string): Promise<number[]> => {
  const response = await fetch(baseUrl + ".timestamps");
  if (!response.ok) return [];
  const timestampsText = await response.text();
  return timestampsText
    .split(/\s+/)
    .filter(Boolean)
    .map((time) => {
      const [minutes, seconds] = time.split(":");
      return parseInt(minutes, 10) * 60 + parseFloat(seconds);
    })
    .filter(Number.isFinite);
};

const isAlignment = (value: unknown, recording: string): value is NarrationAlignment => {
  if (!value || typeof value !== "object") return false;
  const alignment = value as NarrationAlignment;
  return (
    alignment.version === 1 &&
    alignment.recording === recording &&
    typeof alignment.article?.textHash === "string" &&
    Array.isArray(alignment.blocks) &&
    alignment.blocks.every(
      (block) =>
        typeof block.spoken === "boolean" &&
        typeof block.text === "string" &&
        typeof block.hash === "string" &&
        Array.isArray(block.sentences),
    )
  );
};

const getAlignment = async (recording: string): Promise<NarrationAlignment | null> => {
  try {
    const response = await fetch(`/recordings/${recording}.align.json`);
    if (!response.ok) return null;
    const value: unknown = await response.json();
    return isAlignment(value, recording) ? value : null;
  } catch {
    return null;
  }
};

const getAudioSource = (url: string, type: string) => {
  const source = document.createElement("source");
  source.setAttribute("src", url);
  source.setAttribute("type", "audio/" + type);
  return source;
};

const getAudio = (baseUrl: string) => {
  const audio = document.createElement("audio");
  audio.preload = "metadata";
  // Alignments are generated from the MP3, so prefer that exact asset. OGG is
  // retained as a codec fallback for browsers that cannot play MP3.
  audio.appendChild(getAudioSource(baseUrl + ".mp3", "mpeg"));
  audio.appendChild(getAudioSource(baseUrl + ".ogg", "ogg"));
  audio.load();
  return audio;
};

export const getAlignmentUnits = (
  alignment: NarrationAlignment,
  granularity: NarrationGranularity,
) => {
  const units: TimelineUnit[] = [];
  for (const block of alignment.blocks) {
    if (!block.spoken) continue;
    if (granularity === "paragraph") {
      units.push({
        blockIndex: block.index,
        charStart: 0,
        charEnd: block.text.length,
        start: block.start,
        end: block.end,
      });
    } else if (granularity === "sentence") {
      block.sentences.forEach((sentence, sentenceIndex) => {
        units.push({
          blockIndex: block.index,
          charStart: sentence.charStart,
          charEnd: sentence.charEnd,
          start: sentence.start,
          end: sentence.end,
          sentenceIndex,
        });
      });
    } else {
      block.sentences.forEach((sentence, sentenceIndex) => {
        sentence.words.forEach((word, wordIndex) => {
          const previous = units[units.length - 1];
          if (
            previous &&
            previous.blockIndex === block.index &&
            previous.sentenceIndex === sentenceIndex &&
            previous.start === word[2]
          ) {
            // Forced alignment sometimes cannot distinguish several adjacent
            // words and assigns them one instant. Highlight that phrase as a
            // unit rather than making every word but the last unreachable.
            previous.charEnd = word[1];
            previous.end = Math.max(previous.end, word[3]);
            return;
          }
          units.push({
            blockIndex: block.index,
            charStart: word[0],
            charEnd: word[1],
            start: word[2],
            end: word[3],
            sentenceIndex,
            wordIndex,
          });
        });
      });
    }
  }
  return units;
};

const legacyTimeline = (
  timestamps: number[],
  blocks: HTMLElement[],
  duration: number,
): NarrationTimeline => {
  const units = timestamps.slice(0, blocks.length).map((start, index) => ({
    blockIndex: index,
    charStart: 0,
    charEnd: getCanonicalText(blocks[index]).text.length,
    start,
    end: timestamps[index + 1] ?? duration,
  }));

  return {
    alignment: null,
    units: { paragraph: units, sentence: [], word: [] },
    availableGranularities: ["paragraph"],
    legacyTimestamps: timestamps,
  };
};

const alignedTimeline = async (
  alignment: NarrationAlignment,
  blocks: HTMLElement[],
): Promise<NarrationTimeline | null> => {
  if (alignment.blocks.length !== blocks.length) return null;
  const text = blocks.map((block) => getCanonicalText(block).text).join("\n\n");
  if (text !== getAlignmentText(alignment.blocks)) return null;
  if ((await hashText(text)) !== alignment.article.textHash) return null;
  for (const [index, block] of blocks.entries()) {
    if ((await hashText(getCanonicalText(block).text)) !== alignment.blocks[index].hash)
      return null;
  }

  const units = {
    paragraph: getAlignmentUnits(alignment, "paragraph"),
    sentence: getAlignmentUnits(alignment, "sentence"),
    word: getAlignmentUnits(alignment, "word"),
  };
  const availableGranularities: NarrationGranularity[] = ["paragraph"];
  if (units.sentence.length) availableGranularities.push("sentence");
  if (units.word.length) availableGranularities.push("word");
  return { alignment, units, availableGranularities, legacyTimestamps: null };
};

export const getDataForRecording = async (recording: string, blocks: HTMLElement[]) => {
  const [alignment, timestamps, audio] = await Promise.all([
    getAlignment(recording),
    getTimestamps(`/recordings/${recording}`),
    getAudio(`https://assets.emnudge.com/${recording}`),
  ]);

  const timeline = alignment ? await alignedTimeline(alignment, blocks) : null;
  return {
    audio,
    timeline: timeline ?? legacyTimeline(timestamps, blocks, audio.duration),
  };
};

export const getTimelineUnitIndex = (currentTime: number, units: TimelineUnit[]) => {
  if (!units.length) return -1;
  let low = 0;
  let high = units.length - 1;
  while (low <= high) {
    const middle = Math.floor((low + high) / 2);
    if (units[middle].start <= currentTime) low = middle + 1;
    else high = middle - 1;
  }
  return Math.max(0, Math.min(units.length - 1, high));
};

export const getParagraphIndex = (currentTime: number, timestamps: number[]) =>
  getTimelineUnitIndex(
    currentTime,
    timestamps.map((start, index) => ({
      blockIndex: index,
      charStart: 0,
      charEnd: 0,
      start,
      end: timestamps[index + 1] ?? Infinity,
    })),
  );

export const updateParagraphHighlight = (
  paragraphs: HTMLElement[],
  oldIndex: number,
  newIndex: number,
  shouldScroll = true,
) => {
  if (oldIndex >= 0 && paragraphs[oldIndex]) paragraphs[oldIndex].classList.remove("speaking");
  if (newIndex >= 0 && paragraphs[newIndex]) {
    paragraphs[newIndex].classList.add("speaking");
    if (shouldScroll) {
      paragraphs[newIndex].scrollIntoView({
        behavior: window.matchMedia("(prefers-reduced-motion: reduce)").matches ? "auto" : "smooth",
        block: "center",
        inline: "nearest",
      });
    }
  }
};

export const setAudio = async (audio: HTMLAudioElement, isPlaying: boolean) => {
  if (isPlaying) await audio.play();
  else audio.pause();
};

export const getParagraphHighlighter = (paragraphs: HTMLElement[]) => {
  let index = -1;
  return {
    clear() {
      updateParagraphHighlight(paragraphs, index, -1, false);
      index = -1;
    },
    highlight(newIndex: number, shouldScroll = true) {
      if (newIndex === index) return;
      updateParagraphHighlight(paragraphs, index, newIndex, shouldScroll);
      index = newIndex;
    },
  };
};
