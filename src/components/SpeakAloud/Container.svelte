<script>
    export let showControls = false;
    export let paragraphPercentage = 0;
</script>

<div class="controls speak-aloud-controls" class:hide={!showControls}>
    <slot />
    <div class="progress" style="--percentage: {paragraphPercentage * 100}%;" />
</div>

<style>
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
    .controls.hide {
        bottom: calc(-1 * var(--height));
    }
    .controls > :global(*) {
        margin: 0 20px;
    }
    .progress {
        margin: 0;
        position: absolute;
        bottom: 0;
        left: 0;
        width: var(--percentage);
        height: 2px;
        background: var(--active);
    }
</style>
