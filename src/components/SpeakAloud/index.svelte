<script lang="ts">
import { onDestroy, onMount } from "svelte";
import {
	getDataForRecording,
	getParagraphIndex,
	setAudio,
	getParagraphHighlighter,
	getParagraphPercentage,
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
}

function updateParagraphs() {
	if (!isPlaying) return;
	const index = getParagraphIndex(audio.currentTime, timestamps);
	paragraphHighlighter.highlight(index);
	paragraphPercentage = getParagraphPercentage(audio, index, timestamps);
	paragraphIndex = index;
	requestAnimationFrame(updateParagraphs);
}

function changePlaybackSpeed(e: Event & { currentTarget: HTMLSelectElement }) {
	const speed = Number(e.currentTarget.value.slice(1));
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
			return;
		}
	}
	paragraphIndex += offset;

	const mod = (value: number, mod: number) => ((value % mod) + mod) % mod;

	const index = mod(paragraphIndex, paragraphs.length);
	audio.currentTime = timestamps[index];
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

let paragraphPercentage = 0;

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

    <Container {showControls} {paragraphPercentage}>
        <div class="side-controls">
            <Volume {volume} on:adjust={handleVolumeAdjust} />
            <select on:change={changePlaybackSpeed}>
                <option>x.75</option>
                <option selected>x1</option>
                <option>x1.5</option>
                <option>x2</option>
            </select>
        </div>

        <Playback
            {isPlaying}
            on:toggleplay={handleTogglePlay}
            on:skip={handleSkip}
        />
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
        border: none;
        border-radius: 4px;
        padding: 5px 10px;
        cursor: pointer;
        display: grid;
        align-items: center;
        grid-template-columns: 1fr auto;
        grid-gap: 10px;
    }
    button img {
        height: 12px;
        opacity: 0.7;
    }
    .side-controls {
        display: grid;
        grid-template-columns: 1fr auto;
        grid-gap: 20px;
    }
</style>
