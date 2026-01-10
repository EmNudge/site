<script lang="ts">
import { onDestroy, onMount } from "svelte";
import {
	getDataForRecording,
	getParagraphIndex,
	setAudio,
	getParagraphHighlighter,
} from "./utils";
import Playback from "./Playback.svelte";
import Volume from "./Volume.svelte";
import Container from "./Container.svelte";
import TimestampEditor from "./TimestampEditor.svelte";

export let recording: string;
export let isDev: boolean = false;

let audio: HTMLAudioElement;
let timestamps: number[];
let paragraphs: HTMLParagraphElement[];
let paragraphHighlighter: {
	clear(): void;
	highlight(newIndex: number): void;
};

let isPlaying = false;
onMount(async () => {
	const data = await getDataForRecording(recording);
	audio = data.audio;
	timestamps = data.timestamps;
	paragraphs = Array.from(document.querySelectorAll("article > p"));
	paragraphHighlighter = getParagraphHighlighter(paragraphs);

	for (const [index, paragraph] of paragraphs.entries()) {
		paragraph.setAttribute("data-index", index.toString());
	}
	const articleClickListener = (event: MouseEvent) => {
		const audioControls = document.querySelector(".speak-aloud-controls");
		if (!audioControls || audioControls.classList.contains("hide")) return;

		if (!(event.target instanceof HTMLParagraphElement)) return;

		const index = Number(event.target.dataset.index);
		if (Number.isNaN(index)) {
			console.error("Cannot find data index");
			return;
		}
		updateToIndex(index);
	};
	const article = document.querySelector("article");
	article.addEventListener("click", articleClickListener);

	audio.addEventListener("ended", () => {
		paragraphHighlighter.clear();
		isPlaying = false;
	});

	return () => {
		article.removeEventListener("click", articleClickListener);
	};
});

onDestroy(() => {
	showControls = false;
	if (audio) {
		paragraphHighlighter.clear();
		audio.pause();
		audio.currentTime = 0;
	}
});

function handleTogglePlay() {
	isPlaying = !isPlaying;
	setAudio(audio, isPlaying);
	updateParagraphs();
}

function updateToIndex(index: number) {
	paragraphHighlighter.highlight(index);
	paragraphIndex = index;
	audio.currentTime = timestamps[index];
	totalProgress = audio.currentTime / audio.duration;
}

function updateParagraphs() {
	if (!isPlaying) return;
	const index = getParagraphIndex(audio.currentTime, timestamps);
	paragraphHighlighter.highlight(index);
	totalProgress = audio.currentTime / audio.duration;
	paragraphIndex = index;
	requestAnimationFrame(updateParagraphs);
}

function changePlaybackSpeed(e: Event & { currentTarget: HTMLSelectElement }) {
	const speed = Number(e.currentTarget.value.replace("x", ""));
	audio.playbackRate = speed;
}

let paragraphIndex = 0;
function handleSkip(e: Event & { detail: number }) {
	const offset = e.detail;
	// if back, just go to beginning of paragraph unless too close to it
	if (offset === -1) {
		const beginning = timestamps[paragraphIndex];
		const delta = audio.currentTime - beginning;
		if (delta > 3) {
			audio.currentTime = beginning;
			totalProgress = audio.currentTime / audio.duration;
			return;
		}
	}
	paragraphIndex += offset;

	const mod = (value: number, mod: number) => ((value % mod) + mod) % mod;

	const index = mod(paragraphIndex, paragraphs.length);
	audio.currentTime = timestamps[index];
	totalProgress = audio.currentTime / audio.duration;
	paragraphHighlighter.highlight(index);
}

let volume = 0.5;
function handleVolumeAdjust(e: Event & { detail: number }) {
	volume = e.detail;
	audio.volume = volume;
}

let showControls = false;
function closeAudioControls() {
	showControls = false;
	audio.pause();
	audio.currentTime = 0;
	paragraphHighlighter.clear();
}

let totalProgress = 0;

function handleTimestampUpdate(e: CustomEvent<number[]>) {
	timestamps = e.detail;
}
</script>

{#if audio}
    <div class="button-container">
        {#if !showControls}
            <button on:click={() => (showControls = true)}>
                <span>Read This Article Aloud</span>
                <img src="/icons/volume.svg" alt="volume" />
            </button>
        {:else}
            <button on:click={closeAudioControls}>
                <span>Close Audio Controls</span>
                <img src="/icons/volume.svg" alt="volume" />
            </button>
        {/if}
    </div>

    <Container {showControls} {totalProgress} on:close={closeAudioControls}>
        <Playback
            {isPlaying}
            on:toggleplay={handleTogglePlay}
            on:skip={handleSkip}
        />

        <div class="secondary-controls">
            <Volume {volume} on:adjust={handleVolumeAdjust} />
            <select class="speed-select" on:change={changePlaybackSpeed}>
                <option value="x0.75">0.75x</option>
                <option value="x1" selected>1x</option>
                <option value="x1.5">1.5x</option>
                <option value="x2">2x</option>
            </select>
        </div>
    </Container>

    {#if isDev && showControls}
        <TimestampEditor
            {timestamps}
            {paragraphs}
            currentIndex={paragraphIndex}
            on:update={handleTimestampUpdate}
        />
    {/if}
{/if}

<style>
    .button-container {
        margin-top: 15px;
    }
    .button-container button {
        background: var(--light-active);
        color: var(--foreground);
        border: none;
        border-radius: 6px;
        padding: 8px 12px;
        cursor: pointer;
        display: flex;
        align-items: center;
        gap: 8px;
        font-size: var(--font-small);
        transition: opacity 0.15s ease;
    }
    .button-container button:hover {
        opacity: 0.9;
    }
    .button-container button img {
        height: 14px;
        opacity: 0.8;
    }
    .secondary-controls {
        display: flex;
        align-items: center;
        gap: 12px;
    }
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
    .speed-select:hover {
        border-color: var(--active);
    }
    .speed-select:focus {
        outline: none;
        border-color: var(--active);
    }

    /* Desktop: vertical layout */
    @media (min-width: 900px) {
        .secondary-controls {
            flex-direction: column;
            gap: 12px;
            width: 100%;
        }
        .speed-select {
            width: 100%;
            text-align: center;
            font-size: 12px;
            padding: 6px 8px;
        }
    }

    /* Mobile: horizontal layout */
    @media (max-width: 899px) {
        .secondary-controls {
            gap: 8px;
        }
        .speed-select {
            padding: 4px 6px;
            font-size: 12px;
        }
    }
</style>
