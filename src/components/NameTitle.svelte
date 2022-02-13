<script>
    export let titles;

    let titleIndex = 0;
    function changeTitle() {
        titleIndex = (titleIndex + 1) % titles.length;
    }
</script>

<div class="special-container">
    {#key titleIndex}
        <h2 on:click={changeTitle}>{titles[titleIndex]}</h2>
    {/key}
</div>

<style>
    .special-container {
        width: 100%;
    }
    @keyframes pop-in {
        from {
            transform: scale(1.1);
        }
        to {
            transform: scale(1);
        }
    }
    h2 {
        opacity: 0.8;
        width: 100%;
        cursor: grab;
        user-select: none;
        width: max-content;
        animation: pop-in 250ms;
    }
    h2:active {
        transform: scale(0.9);
    }
</style>