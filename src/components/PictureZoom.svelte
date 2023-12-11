<script lang="ts">
    import { onMount } from "svelte";
    import { fade } from "svelte/transition";

    let clickedImage = '';

    onMount(() => {
        const articleEl = document.querySelector('article');
        articleEl.classList.add("js-enabled");
        articleEl.addEventListener("click", (e) => {
            if (!(e.target instanceof HTMLImageElement)) return;
            if (!["P", "FIGURE", "PICTURE"].includes(e.target.parentElement.tagName))
                return;
            // if we clicked an image that is a direct descendant of a paragraph element
            clickedImage = e.target.src;
        });
    });

    function maybeCloseImage(e) {
        if (e.target instanceof HTMLImageElement) return;
        clickedImage = '';
    }
</script>

{#if clickedImage}
    <div
        class="overlay-image"
        transition:fade={{ duration: 100 }}
        on:click={maybeCloseImage}
        on:keypress={maybeCloseImage}
    >
        <img src={clickedImage} alt="large version of thing you clicked" />
    </div>
{/if}

<style>
    .overlay-image {
        display: flex;
        justify-content: center;
        align-items: center;
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background: #000a;
        z-index: 99;
    }
    .overlay-image img {
        max-height: 80%;
        max-width: 80%;
    }
</style>
