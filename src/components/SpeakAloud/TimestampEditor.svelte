<script lang="ts">
    import { createEventDispatcher, onMount, tick } from "svelte";

    export let timestamps: number[];
    export let paragraphs: HTMLParagraphElement[];
    export let currentIndex: number;

    const dispatch = createEventDispatcher<{ update: number[]; seek: number }>();

    $: editableTimestamps = timestamps.map(formatTime);
    let copied = false;
    let positions: { top: number; left: number }[] = [];

    function formatTime(seconds: number): string {
        const mins = Math.floor(seconds / 60);
        const secs = seconds % 60;
        return `${mins.toString().padStart(2, "0")}:${secs.toFixed(2).padStart(5, "0")}`;
    }

    function parseTime(str: string): number | null {
        if (!str || !str.includes(":")) return null;
        const [mins, secs] = str.split(":");
        const minsNum = parseInt(mins, 10);
        const secsNum = parseFloat(secs);
        if (isNaN(minsNum) || isNaN(secsNum)) return null;
        return minsNum * 60 + secsNum;
    }

    function checkInvalid(index: number, timestamps: string[]): boolean {
        const currentValue = parseTime(timestamps[index]);
        if (currentValue === null) return true;

        const prevValue = index > 0 ? parseTime(timestamps[index - 1]) : -1;
        const nextValue = index < timestamps.length - 1
            ? parseTime(timestamps[index + 1])
            : Infinity;

        if (prevValue !== null && currentValue < prevValue) return true;
        if (nextValue !== null && currentValue > nextValue) return true;
        return false;
    }

    let invalidStates: boolean[] = [];
    $: invalidStates = editableTimestamps.map((_, i) => checkInvalid(i, editableTimestamps));

    function handleInput(index: number) {
        const filtered = editableTimestamps[index].replace(/[^\d.:]/g, "");
        editableTimestamps[index] = filtered;
        editableTimestamps = editableTimestamps;
        dispatchUpdate();

        if (index === currentIndex) {
            const time = parseTime(filtered);
            if (time !== null) {
                dispatch("seek", time);
            }
        }
    }

    function adjustTime(index: number, delta: number) {
        const current = parseTime(editableTimestamps[index]) ?? 0;
        const newValue = Math.max(0, current + delta);
        editableTimestamps[index] = formatTime(newValue);
        editableTimestamps = editableTimestamps;
        dispatchUpdate();

        if (index === currentIndex) {
            dispatch("seek", newValue);
        }
    }

    function dispatchUpdate() {
        const newTimestamps = editableTimestamps.map((t) => parseTime(t) ?? 0);
        dispatch("update", newTimestamps);
    }

    function handleCopy() {
        const text = editableTimestamps.join("\n") + "\n";
        navigator.clipboard.writeText(text);
        copied = true;
        setTimeout(() => (copied = false), 2000);
    }

    function updatePositions() {
        positions = paragraphs.map((p) => {
            const rect = p.getBoundingClientRect();
            return {
                top: rect.top + window.scrollY,
                left: rect.left + window.scrollX,
            };
        });
    }

    onMount(() => {
        updatePositions();
        window.addEventListener("resize", updatePositions);
        window.addEventListener("scroll", updatePositions);
        return () => {
            window.removeEventListener("resize", updatePositions);
            window.removeEventListener("scroll", updatePositions);
        };
    });

    $: if (typeof window !== "undefined" && paragraphs) {
        tick().then(updatePositions);
    }
</script>

<div class="timestamp-inputs">
    {#each editableTimestamps as ts, i}
        {#if positions[i]}
            <div
                class="input-wrapper"
                class:active={i === currentIndex}
                class:invalid={invalidStates[i]}
                style="top: {positions[i].top - 20}px; left: {positions[i].left - 100}px;"
            >
                <button class="adj-btn" on:click={() => adjustTime(i, -0.5)}>-</button>
                <input
                    type="text"
                    bind:value={editableTimestamps[i]}
                    on:input={() => handleInput(i)}
                    on:blur={() => editableTimestamps = editableTimestamps}
                />
                <button class="adj-btn" on:click={() => adjustTime(i, 0.5)}>+</button>
            </div>
        {/if}
    {/each}
</div>

<button class="copy-btn" on:click={handleCopy}>
    {copied ? "Copied!" : "Copy"}
</button>

<style>
    .timestamp-inputs {
        position: absolute;
        top: 0;
        left: 0;
        pointer-events: none;
        z-index: 1000;
    }

    .input-wrapper {
        position: absolute;
        pointer-events: auto;
        display: flex;
        align-items: center;
        gap: 2px;
    }

    .adj-btn {
        background: #1a1a2e;
        border: 1px solid #4a4a6a;
        color: #888;
        padding: 4px 5px;
        border-radius: 3px;
        font-family: monospace;
        font-size: 9px;
        cursor: pointer;
        line-height: 1;
    }

    .adj-btn:hover {
        background: #252542;
        color: #e0e0e0;
    }

    input {
        width: 68px;
        background: #1a1a2e;
        border: 1px solid #4a4a6a;
        color: #e0e0e0;
        padding: 4px 6px;
        border-radius: 4px;
        font-family: monospace;
        font-size: 11px;
        text-align: center;
    }

    input:focus {
        outline: none;
        border-color: #4a9eff;
    }

    .input-wrapper.active input {
        border-color: #4a9eff;
        background: #252542;
    }

    .input-wrapper.invalid input {
        border-color: #ff4a4a;
        background: #2e1a1a;
    }

    .copy-btn {
        position: fixed;
        bottom: 20px;
        left: 20px;
        background: #4a9eff;
        color: white;
        border: none;
        padding: 8px 16px;
        border-radius: 6px;
        cursor: pointer;
        font-size: 12px;
        font-family: monospace;
        z-index: 10000;
    }

    .copy-btn:hover {
        background: #3a8eef;
    }
</style>
