<script>
import { createEventDispatcher } from "svelte";
const dispatch = createEventDispatcher();
export let isPlaying;
</script>

<div class="main-controls">
    <button class="skip-back-btn" on:click={() => dispatch("skip", -1)}>
        <img src="/icons/skip.svg" alt="skip back" />
    </button>
    <button class="play-btn" on:click={() => dispatch("toggleplay")}>
        <svg
            width="62"
            height="58"
            viewBox="0 0 62 58"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >
            {#if isPlaying}
                <rect
                    x="18.494"
                    y="13.8494"
                    width="6.56024"
                    height="30.6145"
                    fill="white"
                />
                <rect
                    x="34.5302"
                    y="13.8494"
                    width="6.56024"
                    height="30.6145"
                    fill="white"
                />
            {:else}
                <path
                    d="M45.1927 29.1566L22.7786 44.3069L22.7786 14.0064L45.1927 29.1566Z"
                    fill="white"
                />
            {/if}
        </svg>
    </button>
    <button class="skip-ahead-btn" on:click={() => dispatch("skip", 1)}>
        <img src="/icons/skip.svg" alt="skip ahead" />
    </button>
</div>

<style>
    .main-controls {
        display: grid;
        grid-template-columns: 1fr auto 1fr;
        grid-gap: 10px;
    }
    button {
        background: none;
        border: none;
        padding: 0;
        cursor: pointer;
        transition: 0.15s;
    }
    button:hover {
        transform: scale(1.1);
    }
    .skip-back-btn {
        transform: scaleX(-1);
    }
    .skip-back-btn:hover {
        transform: scaleX(-1.1) scaleY(1.1);
    }
</style>
