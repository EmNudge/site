<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import type { NarrationGranularity } from "./types";

  export let options: NarrationGranularity[] = [];
  export let value: NarrationGranularity;
  const dispatch = createEventDispatcher<{ change: NarrationGranularity }>();
  const labels: Record<NarrationGranularity, string> = {
    paragraph: "Paragraph",
    sentence: "Sentence",
    word: "Word",
  };
</script>

<div class="granularity" role="radiogroup" aria-label="Highlight detail">
  {#each options as option}
    <button
      type="button"
      role="radio"
      aria-checked={value === option}
      class:active={value === option}
      on:click={() => dispatch("change", option)}
    >{labels[option]}</button>
  {/each}
</div>

<style>
  .granularity {
    display: flex;
    width: 100%;
    border: 1px solid var(--light-bg);
    border-radius: 6px;
    overflow: hidden;
  }
  button {
    flex: 1;
    border: 0;
    border-right: 1px solid var(--light-bg);
    padding: 6px 4px;
    background: transparent;
    color: var(--muted);
    font: inherit;
    font-size: 11px;
    cursor: pointer;
  }
  button:last-child { border-right: 0; }
  button:hover, button:focus-visible { color: var(--foreground); background: var(--light-bg); }
  button:focus-visible { outline: 2px solid var(--active); outline-offset: -2px; }
  button.active { color: var(--background); background: var(--active); }
  @media (min-width: 900px) {
    .granularity { flex-direction: column; }
    button { border-right: 0; border-bottom: 1px solid var(--light-bg); }
    button:last-child { border-bottom: 0; }
  }
  @media (max-width: 899px) {
    .granularity { width: auto; }
    button { padding: 4px 7px; }
  }
</style>
