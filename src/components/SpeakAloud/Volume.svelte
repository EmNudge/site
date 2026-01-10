<script lang="ts">
import { createEventDispatcher } from "svelte";
const dispatch = createEventDispatcher();

export let volume: number;

function handleChange(e: Event & { currentTarget: HTMLInputElement }) {
	dispatch("adjust", Number(e.currentTarget.value));
}
</script>

<div class="volume-control">
    <svg class="volume-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M11 5L6 9H2v6h4l5 4V5z" fill="currentColor"/>
        {#if volume > 0}
            <path d="M15.54 8.46a5 5 0 010 7.07" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
        {/if}
        {#if volume > 0.5}
            <path d="M19.07 4.93a10 10 0 010 14.14" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
        {/if}
    </svg>
    <input
        type="range"
        min="0"
        max="1"
        step=".05"
        value={volume}
        on:input={handleChange}
        aria-label="Volume"
    />
</div>

<style>
    .volume-control {
        display: flex;
        align-items: center;
        gap: 8px;
    }
    .volume-icon {
        color: var(--foreground);
        opacity: 0.7;
        flex-shrink: 0;
    }
    input[type="range"] {
        -webkit-appearance: none;
        appearance: none;
        height: 6px;
        background: var(--light-bg);
        border-radius: 3px;
        cursor: pointer;
    }
    input[type="range"]::-webkit-slider-thumb {
        -webkit-appearance: none;
        appearance: none;
        width: 16px;
        height: 16px;
        background: var(--active);
        border-radius: 50%;
        cursor: pointer;
    }
    input[type="range"]::-moz-range-thumb {
        width: 16px;
        height: 16px;
        background: var(--active);
        border-radius: 50%;
        border: none;
        cursor: pointer;
    }

    /* Desktop */
    @media (min-width: 900px) {
        .volume-icon {
            width: 20px;
            height: 20px;
        }
        input[type="range"] {
            width: 100%;
            min-width: 70px;
        }
    }

    /* Mobile */
    @media (max-width: 899px) {
        .volume-control {
            gap: 4px;
        }
        .volume-icon {
            width: 14px;
            height: 14px;
        }
        input[type="range"] {
            width: 50px;
        }
    }
</style>
