<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import {
    getDataForRecording,
    getTimelineUnitIndex,
    setAudio,
    getParagraphHighlighter,
  } from "./utils";
  import {
    getCanonicalText,
    getNarrationBlocks,
    makeRange,
    getCanonicalOffset,
  } from "./narration";
  import type { NarrationGranularity, NarrationTimeline, TimelineUnit } from "./types";
  import Playback from "./Playback.svelte";
  import Volume from "./Volume.svelte";
  import Container from "./Container.svelte";
  import Granularity from "./Granularity.svelte";
  import TimestampEditor from "./TimestampEditor.svelte";

  export let recording: string;
  export let isDev: boolean = false;

  let audio: HTMLAudioElement;
  let timeline: NarrationTimeline;
  let paragraphs: HTMLElement[] = [];
  let textMaps: ReturnType<typeof getCanonicalText>[] = [];
  let paragraphHighlighter: ReturnType<typeof getParagraphHighlighter>;
  let granularity: NarrationGranularity = "paragraph";
  let isAligned = false;
  let availableGranularities: NarrationGranularity[] = ["paragraph"];
  let activeUnitIndex = 0;
  let isPlaying = false;
  let totalProgress = 0;
  let volume = 0.5;
  let showControls = false;
  let animationFrame: number | null = null;
  let articleElement: HTMLElement | null = null;

  const highlightNames = ["speak-sentence", "speak-word"] as const;
  const hasCustomHighlights = () =>
    typeof CSS !== "undefined" && "highlights" in CSS && typeof Highlight !== "undefined";

  function clearCustomHighlights() {
    if (!hasCustomHighlights()) return;
    for (const name of highlightNames) CSS.highlights.delete(name);
  }

  function addHighlight(name: string, ranges: Range[]) {
    if (!hasCustomHighlights() || !ranges.length) return;
    CSS.highlights.set(name, new Highlight(...ranges));
  }

  function rangesFor(unit: TimelineUnit): Range[] {
    const map = textMaps[unit.blockIndex];
    const range = map && makeRange(map.map, unit.charStart, unit.charEnd);
    return range ? [range] : [];
  }

  function sentenceForWord(unit: TimelineUnit): TimelineUnit | undefined {
    if (!timeline.alignment || unit.sentenceIndex === undefined) return undefined;
    const block = timeline.alignment.blocks[unit.blockIndex];
    const sentence = block?.sentences[unit.sentenceIndex];
    if (!sentence) return undefined;
    return {
      blockIndex: unit.blockIndex,
      charStart: sentence.charStart,
      charEnd: sentence.charEnd,
      start: sentence.start,
      end: sentence.end,
      sentenceIndex: unit.sentenceIndex,
    };
  }

  function highlightUnit(index: number, shouldScroll = true) {
    const units = timeline?.units[granularity] ?? [];
    if (!units.length) return;
    const unit = units[Math.max(0, Math.min(index, units.length - 1))];
    const previousUnit = units[activeUnitIndex];
    const blockChanged = !previousUnit || unit.blockIndex !== previousUnit.blockIndex;
    activeUnitIndex = Math.max(0, Math.min(index, units.length - 1));
    paragraphHighlighter?.highlight(unit.blockIndex, shouldScroll && blockChanged);

    clearCustomHighlights();
    if (granularity === "sentence") {
      addHighlight("speak-sentence", rangesFor(unit));
    } else if (granularity === "word") {
      const sentence = sentenceForWord(unit);
      if (sentence) addHighlight("speak-sentence", rangesFor(sentence));
      addHighlight("speak-word", rangesFor(unit));
    }
  }

  function currentUnits() {
    return timeline?.units[granularity] ?? [];
  }

  function updateProgress() {
    totalProgress = audio.duration ? audio.currentTime / audio.duration : 0;
  }

  function stopPlaybackLoop() {
    if (animationFrame !== null) cancelAnimationFrame(animationFrame);
    animationFrame = null;
  }

  function updatePlaybackHighlight() {
    if (!isPlaying || !timeline) {
      animationFrame = null;
      return;
    }
    const units = currentUnits();
    const index = getTimelineUnitIndex(audio.currentTime, units);
    if (index >= 0) highlightUnit(index);
    updateProgress();
    animationFrame = requestAnimationFrame(updatePlaybackHighlight);
  }

  function handleTogglePlay() {
    isPlaying = !isPlaying;
    void setAudio(audio, isPlaying).catch((error) => {
      isPlaying = false;
      stopPlaybackLoop();
      console.error("Could not play narration", error);
    });
    if (isPlaying && animationFrame === null) updatePlaybackHighlight();
    else if (!isPlaying) stopPlaybackLoop();
  }

  function updateToIndex(index: number) {
    const units = currentUnits();
    if (!units[index]) return;
    audio.currentTime = units[index].start;
    highlightUnit(index, false);
    updateProgress();
  }

  function handleSkip(e: Event & { detail: number }) {
    const units = currentUnits();
    if (!units.length) return;
    const offset = e.detail;
    const current = units[activeUnitIndex] ?? units[0];
    if (offset === -1 && audio.currentTime - current.start > 3) {
      updateToIndex(activeUnitIndex);
      return;
    }
    const nextIndex = ((activeUnitIndex + offset) % units.length + units.length) % units.length;
    updateToIndex(nextIndex);
  }

  function setArticleMode() {
    document.querySelector("article")?.setAttribute("data-narration-mode", granularity);
  }

  function changeGranularity(event: CustomEvent<NarrationGranularity>) {
    granularity = event.detail;
    setArticleMode();
    localStorage.setItem("speakAloud.granularity", granularity);
    const index = getTimelineUnitIndex(audio.currentTime, currentUnits());
    highlightUnit(index < 0 ? 0 : index, false);
  }

  function changePlaybackSpeed(e: Event & { currentTarget: HTMLSelectElement }) {
    audio.playbackRate = Number(e.currentTarget.value.replace("x", ""));
  }

  function handleVolumeAdjust(e: Event & { detail: number }) {
    volume = e.detail;
    audio.volume = volume;
  }

  function closeAudioControls() {
    showControls = false;
    isPlaying = false;
    stopPlaybackLoop();
    audio.pause();
    audio.currentTime = 0;
    totalProgress = 0;
    activeUnitIndex = 0;
    clearCustomHighlights();
    paragraphHighlighter?.clear();
  }

  function getPointFromEvent(event: MouseEvent): { node: Node; offset: number } | null {
    const position = document.caretPositionFromPoint?.(event.clientX, event.clientY);
    if (position) return { node: position.offsetNode, offset: position.offset };
    const range = document.caretRangeFromPoint?.(event.clientX, event.clientY);
    return range ? { node: range.startContainer, offset: range.startOffset } : null;
  }

  function handleArticleClick(event: MouseEvent) {
    if (!showControls || !timeline) return;
    if (window.getSelection()?.isCollapsed === false) return;
    const target = event.target as HTMLElement;
    if (target.closest("a")) return;
    const block = target.closest("article > p, article > blockquote") as HTMLElement | null;
    if (!block) return;
    const blockIndex = paragraphs.indexOf(block);
    if (blockIndex < 0) return;

    if (granularity === "paragraph") {
      const index = currentUnits().findIndex((unit) => unit.blockIndex === blockIndex);
      if (index >= 0) updateToIndex(index);
      return;
    }

    const point = getPointFromEvent(event);
    const map = textMaps[blockIndex];
    const offset = point && map ? getCanonicalOffset(map.map, point.node, point.offset) : null;
    const units = currentUnits();
    const index = units.findIndex(
      (unit) =>
        unit.blockIndex === blockIndex &&
        (offset === null || (unit.charStart <= offset && offset < unit.charEnd)),
    );
    const fallback = units.findIndex((unit) => unit.blockIndex === blockIndex);
    if (index >= 0) updateToIndex(index);
    else if (fallback >= 0) updateToIndex(fallback);
  }

  function handleKeydown(event: KeyboardEvent) {
    if (!showControls) return;
    if (
      event.target instanceof HTMLElement &&
      event.target.closest("button, input, textarea, select, [role='radio']")
    )
      return;
    if (event.code === "Space") {
      event.preventDefault();
      handleTogglePlay();
    } else if (event.code === "ArrowLeft") {
      event.preventDefault();
      handleSkip({ detail: -1 } as CustomEvent<number>);
    } else if (event.code === "ArrowRight") {
      event.preventDefault();
      handleSkip({ detail: 1 } as CustomEvent<number>);
    }
  }

  function handleTimestampUpdate(event: CustomEvent<number[]>) {
    if (!timeline.legacyTimestamps) return;
    const timestamps = event.detail;
    timeline = {
      ...timeline,
      legacyTimestamps: timestamps,
      units: {
        ...timeline.units,
        paragraph: timestamps.map((start, index) => ({
          blockIndex: index,
          charStart: 0,
          charEnd: textMaps[index]?.text.length ?? 0,
          start,
          end: timestamps[index + 1] ?? audio.duration,
        })),
      },
    };
  }

  function handleTimestampSeek(event: CustomEvent<number>) {
    audio.currentTime = event.detail;
    updateProgress();
  }

  function handleTimestampDelete(event: CustomEvent<number>) {
    if (!timeline.legacyTimestamps || timeline.legacyTimestamps.length <= 1) return;
    const index = event.detail;
    const timestamps = timeline.legacyTimestamps.filter((_, i) => i !== index);
    const removed = paragraphs[index];
    removed?.removeAttribute("data-index");
    paragraphs = paragraphs.filter((_, i) => i !== index);
    textMaps = textMaps.filter((_, i) => i !== index);
    paragraphs.forEach((paragraph, paragraphIndex) => {
      paragraph.setAttribute("data-index", paragraphIndex.toString());
    });
    paragraphHighlighter.clear();
    paragraphHighlighter = getParagraphHighlighter(paragraphs);
    activeUnitIndex = Math.min(activeUnitIndex, paragraphs.length - 1);
    handleTimestampUpdate({ detail: timestamps } as CustomEvent<number[]>);
  }

  function handlePlay() {
    isPlaying = true;
    if (animationFrame === null) updatePlaybackHighlight();
  }

  function handlePause() {
    isPlaying = false;
    stopPlaybackLoop();
  }

  function handleEnded() {
    isPlaying = false;
    stopPlaybackLoop();
    clearCustomHighlights();
    paragraphHighlighter?.clear();
  }

  function removeListeners() {
    articleElement?.removeEventListener("click", handleArticleClick);
    window.removeEventListener("keydown", handleKeydown);
    audio?.removeEventListener("play", handlePlay);
    audio?.removeEventListener("pause", handlePause);
    audio?.removeEventListener("ended", handleEnded);
  }

  onMount(() => {
    let cancelled = false;
    paragraphs = getNarrationBlocks();
    textMaps = paragraphs.map(getCanonicalText);
    paragraphHighlighter = getParagraphHighlighter(paragraphs);
    paragraphs.forEach((paragraph, index) => {
      paragraph.setAttribute("data-index", index.toString());
    });

    void getDataForRecording(recording, paragraphs)
      .then((data) => {
        if (cancelled) {
          data.audio.pause();
          return;
        }
        audio = data.audio;
        timeline = data.timeline;
        availableGranularities = hasCustomHighlights()
          ? timeline.availableGranularities
          : ["paragraph"];
        isAligned = Boolean(timeline.alignment);
        const saved = localStorage.getItem(
          "speakAloud.granularity",
        ) as NarrationGranularity | null;
        granularity =
          saved && availableGranularities.includes(saved)
            ? saved
            : isAligned && availableGranularities.includes("sentence")
              ? "sentence"
              : "paragraph";
        setArticleMode();
        audio.volume = volume;

        articleElement = document.querySelector("article");
        articleElement?.addEventListener("click", handleArticleClick);
        window.addEventListener("keydown", handleKeydown);
        audio.addEventListener("play", handlePlay);
        audio.addEventListener("pause", handlePause);
        audio.addEventListener("ended", handleEnded);
      })
      .catch((error) => console.error("Could not initialize narration", error));

    return () => {
      cancelled = true;
      removeListeners();
    };
  });

  onDestroy(() => {
    showControls = false;
    isPlaying = false;
    stopPlaybackLoop();
    removeListeners();
    clearCustomHighlights();
    articleElement?.removeAttribute("data-narration-mode");
    if (audio) {
      audio.pause();
      audio.currentTime = 0;
      paragraphHighlighter?.clear();
    }
  });
</script>

{#if audio}
  <div class="button-container">
    {#if !showControls}
      <button class="pill" on:click={() => (showControls = true)}>
        <span>Read This Article Aloud</span>
        <img src="/icons/volume.svg" alt="volume" />
      </button>
    {:else}
      <button class="pill" on:click={closeAudioControls}>
        <span>Close Audio Controls</span>
        <img src="/icons/volume.svg" alt="volume" />
      </button>
    {/if}
  </div>

  <Container {showControls} {totalProgress} on:close={closeAudioControls}>
    <Playback {isPlaying} on:toggleplay={handleTogglePlay} on:skip={handleSkip} />
    <div class="secondary-controls">
      <Volume {volume} on:adjust={handleVolumeAdjust} />
      {#if availableGranularities.length > 1}
        <Granularity
          options={availableGranularities}
          value={granularity}
          on:change={changeGranularity}
        />
      {/if}
      <select class="speed-select" on:change={changePlaybackSpeed} aria-label="Playback speed">
        <option value="x0.75">0.75x</option>
        <option value="x1" selected>1x</option>
        <option value="x1.5">1.5x</option>
        <option value="x2">2x</option>
      </select>
    </div>
  </Container>

  {#if isDev && showControls && timeline && timeline.legacyTimestamps}
    <TimestampEditor
      timestamps={timeline.legacyTimestamps}
      {paragraphs}
      currentIndex={activeUnitIndex}
      on:update={handleTimestampUpdate}
      on:seek={handleTimestampSeek}
      on:delete={handleTimestampDelete}
    />
  {/if}
{/if}

<style>
  .button-container { margin-top: 15px; }
  @media (min-width: 600px) {
    .button-container { float: right; margin-top: -3.5rem; }
  }
  .pill {
    background: none;
    color: var(--muted);
    border: 1px solid #fff4;
    border-radius: 1rem;
    padding: 0.35rem 0.75rem;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 0.85rem;
    transition: 0.15s;
  }
  .pill:hover { border-color: var(--foreground); }
  .pill img { height: 14px; opacity: 0.6; }
  .secondary-controls { display: flex; align-items: center; gap: 12px; }
  .speed-select {
    background: var(--light-bg);
    color: var(--foreground);
    border: 1px solid var(--light-bg);
    border-radius: 6px;
    padding: 6px 8px;
    font-size: var(--font-small);
    cursor: pointer;
    transition: border-color 0.15s ease;
  }
  .speed-select:hover, .speed-select:focus { border-color: var(--active); }
  .speed-select:focus { outline: none; }
  @media (min-width: 900px) {
    .secondary-controls { flex-direction: column; gap: 12px; width: 100%; }
    .speed-select { width: 100%; text-align: center; font-size: 12px; padding: 6px 8px; }
  }
  @media (max-width: 899px) {
    .secondary-controls { gap: 8px; }
    .speed-select { padding: 4px 6px; font-size: 12px; }
  }
</style>
