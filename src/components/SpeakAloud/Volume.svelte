<script lang="ts">
    import { createEventDispatcher } from "svelte";
    const dispatch = createEventDispatcher();

    export let volume;

    function handleChange(e: Event & { currentTarget: HTMLInputElement }) {
        dispatch("adjust", e.currentTarget.value);
    }
</script>

<input
    type="range"
    min="0"
    max="1"
    step=".05"
    {volume}
    on:input={handleChange}
/>
