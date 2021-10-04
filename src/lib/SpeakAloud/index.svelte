<script lang="ts">
    import { onDestroy, onMount } from "svelte";
    import {
        getDataForRecording,
        getParagraphIndex,
        setAudio,
        getParagraphHighlighter,
    } from "./utils";
    import { mod } from "../../utils/mod";

    import Playback from "./Playback.svelte";
    import Volume from "./Volume.svelte";

    export let recording: string;

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

        paragraphs = Array.from(document.querySelectorAll("article p"));
        paragraphHighlighter = getParagraphHighlighter(paragraphs);

        audio.addEventListener("ended", () => {
            paragraphHighlighter.clear();
            isPlaying = false;
        });
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

    function updateParagraphs() {
        if (!isPlaying) return;

        const index = getParagraphIndex(audio.currentTime, timestamps);
        paragraphHighlighter.highlight(index);

        requestAnimationFrame(updateParagraphs);
    }

    function changePlaybackSpeed(
        e: Event & { currentTarget: HTMLSelectElement }
    ) {
        const speed = Number(e.currentTarget.value.slice(1));
        audio.playbackRate = speed;
    }

    let paragraphIndex = 0;
    function handleSkip(e: Event & { detail: number }) {
        const offset = e.detail;
        paragraphIndex += offset;

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
{/if}

<div class="controls" class:hide={!showControls}>
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
</div>

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
    @media (prefers-color-scheme: light) {
        button img {
            height: 12px;
            filter: invert(1);
        }
    }

    .controls {
        display: grid;
        justify-content: center;
        align-items: center;
        --height: 120px;

        height: var(--height);
        box-sizing: border-box;
        padding: 20px;
        padding-bottom: 10px;

        position: fixed;
        bottom: 0;
        left: 0;
        right: 0;
        background: #0005;
        backdrop-filter: blur(5px);
        z-index: 1;

        transition: 0.25s;
    }
    .side-controls {
        display: grid;
        grid-template-columns: 1fr auto;
        grid-gap: 20px;
    }
    .controls.hide {
        bottom: calc(-1 * var(--height));
    }
    .controls > :global(*) {
        margin: 0 20px;
    }
</style>
