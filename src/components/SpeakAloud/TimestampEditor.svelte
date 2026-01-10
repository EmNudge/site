<script lang="ts">
    import { createEventDispatcher } from "svelte";

    export let timestamps: number[];
    export let paragraphs: HTMLParagraphElement[];
    export let currentIndex: number;

    const dispatch = createEventDispatcher<{ update: number[] }>();

    let editableTimestamps = timestamps.map(formatTime);
    let copied = false;
    let collapsed = false;

    function formatTime(seconds: number): string {
        const mins = Math.floor(seconds / 60);
        const secs = Math.floor(seconds % 60);
        return `${mins.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
    }

    function parseTime(str: string): number {
        const [mins, secs] = str.split(":").map(Number);
        if (isNaN(mins) || isNaN(secs)) return 0;
        return mins * 60 + secs;
    }

    function handleInput(index: number, value: string) {
        editableTimestamps[index] = value;
        const newTimestamps = editableTimestamps.map(parseTime);
        dispatch("update", newTimestamps);
    }

    function handleCopy() {
        const text = editableTimestamps.join("\n") + "\n";
        navigator.clipboard.writeText(text);
        copied = true;
        setTimeout(() => (copied = false), 2000);
    }

    function getParagraphPreview(index: number): string {
        if (!paragraphs[index]) return "";
        const text = paragraphs[index].textContent || "";
        return text.slice(0, 40) + (text.length > 40 ? "..." : "");
    }

    $: if (timestamps) {
        editableTimestamps = timestamps.map(formatTime);
    }
</script>

<div class="timestamp-editor" class:collapsed>
    <div class="header">
        <button class="collapse-btn" on:click={() => (collapsed = !collapsed)} aria-label={collapsed ? "Expand" : "Collapse"}>
            <svg width="12" height="12" viewBox="0 0 12 12" fill="currentColor">
                <path d={collapsed ? "M3 5l3 3 3-3" : "M3 7l3-3 3 3"} stroke="currentColor" stroke-width="1.5" fill="none" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
        </button>
        <h3>Timestamp Editor</h3>
        {#if !collapsed}
            <button class="copy-btn" on:click={handleCopy}>
                {copied ? "Copied!" : "Copy All"}
            </button>
        {/if}
    </div>
    {#if !collapsed}
        <div class="list">
            {#each editableTimestamps as ts, i}
                <div class="row" class:active={i === currentIndex}>
                    <span class="index">{i + 1}</span>
                    <input
                        type="text"
                        value={ts}
                        on:input={(e) => handleInput(i, e.currentTarget.value)}
                        pattern="\d{2}:\d{2}"
                    />
                    <span class="preview">{getParagraphPreview(i)}</span>
                </div>
            {/each}
        </div>
    {/if}
</div>

<style>
    .timestamp-editor {
        position: fixed;
        top: 10px;
        left: 10px;
        width: 350px;
        max-height: 80vh;
        background: #1a1a2e;
        border: 1px solid #4a4a6a;
        border-radius: 8px;
        font-family: monospace;
        font-size: 12px;
        z-index: 10000;
        display: flex;
        flex-direction: column;
    }

    .timestamp-editor.collapsed {
        width: auto;
    }

    .timestamp-editor.collapsed .header {
        border-bottom: none;
        border-radius: 8px;
    }

    .header {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 8px 12px;
        border-bottom: 1px solid #4a4a6a;
        background: #252542;
        border-radius: 8px 8px 0 0;
    }

    h3 {
        margin: 0;
        font-size: 14px;
        color: #e0e0e0;
        flex: 1;
    }

    .collapse-btn {
        display: flex;
        align-items: center;
        justify-content: center;
        background: none;
        border: none;
        color: #888;
        cursor: pointer;
        padding: 4px;
        border-radius: 4px;
        transition: color 0.15s ease, background 0.15s ease;
    }

    .collapse-btn:hover {
        color: #e0e0e0;
        background: #3a3a5a;
    }

    .copy-btn {
        background: #4a9eff;
        color: white;
        border: none;
        padding: 4px 12px;
        border-radius: 4px;
        cursor: pointer;
        font-size: 12px;
    }

    .copy-btn:hover {
        background: #3a8eef;
    }

    .list {
        overflow-y: auto;
        padding: 8px;
    }

    .row {
        display: grid;
        grid-template-columns: 24px 60px 1fr;
        gap: 8px;
        align-items: center;
        padding: 4px;
        border-radius: 4px;
    }

    .row.active {
        background: #3a3a5a;
    }

    .index {
        color: #888;
        text-align: right;
    }

    input {
        background: #2a2a4a;
        border: 1px solid #4a4a6a;
        color: #e0e0e0;
        padding: 4px 8px;
        border-radius: 4px;
        font-family: monospace;
        font-size: 12px;
        width: 100%;
    }

    input:focus {
        outline: none;
        border-color: #4a9eff;
    }

    .preview {
        color: #888;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }
</style>
