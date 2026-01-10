<script>
import { createEventDispatcher } from "svelte";
const dispatch = createEventDispatcher();
export let isPlaying;
</script>

<div class="main-controls">
    <button class="skip-btn skip-back" on:click={() => dispatch("skip", -1)} aria-label="Skip back">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path d="M19 20L9 12L19 4V20Z" fill="currentColor"/>
            <rect x="5" y="4" width="2" height="16" fill="currentColor"/>
        </svg>
    </button>
    <button class="play-btn" on:click={() => dispatch("toggleplay")} aria-label={isPlaying ? "Pause" : "Play"}>
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
            {#if isPlaying}
                <rect x="6" y="4" width="4" height="16" rx="1" fill="currentColor"/>
                <rect x="14" y="4" width="4" height="16" rx="1" fill="currentColor"/>
            {:else}
                <path d="M8 5.14v13.72a1 1 0 001.5.86l11-6.86a1 1 0 000-1.72l-11-6.86a1 1 0 00-1.5.86z" fill="currentColor"/>
            {/if}
        </svg>
    </button>
    <button class="skip-btn skip-forward" on:click={() => dispatch("skip", 1)} aria-label="Skip forward">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path d="M5 4L15 12L5 20V4Z" fill="currentColor"/>
            <rect x="17" y="4" width="2" height="16" fill="currentColor"/>
        </svg>
    </button>
</div>

<style>
    .main-controls {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 4px;
        flex-shrink: 0;
    }

    @media (min-width: 900px) {
        .main-controls {
            width: 100%;
        }
    }
    button {
        display: flex;
        align-items: center;
        justify-content: center;
        background: none;
        border: none;
        padding: 6px;
        cursor: pointer;
        transition: transform 0.15s ease, background 0.15s ease;
        border-radius: 50%;
        color: var(--foreground);
        flex-shrink: 0;
    }
    button:hover {
        background: var(--light-bg);
        transform: scale(1.1);
    }
    .play-btn {
        width: 36px;
        height: 36px;
        background: var(--active);
        color: var(--background);
    }
    .play-btn:hover {
        background: var(--active);
        opacity: 0.9;
    }
    .skip-btn {
        width: 28px;
        height: 28px;
    }
    .skip-btn svg {
        width: 16px;
        height: 16px;
    }
    .play-btn svg {
        width: 20px;
        height: 20px;
    }

    /* Mobile: slightly larger controls */
    @media (max-width: 899px) {
        .main-controls {
            gap: 8px;
        }
        .play-btn {
            width: 44px;
            height: 44px;
        }
        .skip-btn {
            width: 32px;
            height: 32px;
        }
        .skip-btn svg {
            width: 20px;
            height: 20px;
        }
        .play-btn svg {
            width: 24px;
            height: 24px;
        }
    }
</style>
