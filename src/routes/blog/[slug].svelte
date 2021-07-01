<script context="module" lang="ts">
	import type { Load } from '@sveltejs/kit';

	export const load: Load = async ({ page, fetch }) => {
		const res = await fetch(`${page.params.slug}.json`);

		if (res.ok) {
			const post = await res.json();
			return {
				props: { post }
			};
		}

		const { message } = await res.json();

		return {
			error: new Error(message)
		};
	};
</script>

<script lang="ts">
  import type { BlogPost } from '$lib/types';
	import { onMount } from 'svelte';
	import { fade } from 'svelte/transition';

	export let post: BlogPost;

	let articleEl: HTMLElement;
	let showLargeImage = false;
	let clickedImage = '#';
	onMount(() => {
		articleEl.classList.add('js-enabled');
		articleEl.addEventListener('click', e => {
			if (!(e.target instanceof HTMLImageElement)) return;
			if (!['P', 'FIGURE'].includes(e.target.parentElement.tagName)) return;
			// if we clicked an image that is a direct descendant of a paragraph element
			showLargeImage = true;
			clickedImage = e.target.src;
		});
	});

	function maybeCloseImage(e) {
		if (e.target instanceof HTMLImageElement) return;
		showLargeImage = false;
	}
</script>

<svelte:head>
	<title>{post.title}</title>
	<link rel="stylesheet" href="/blog.css">
</svelte:head>

<div class="title-container">
  <h5>{post.date} • {post.minuteLength} minute read</h5>
  <h1>{post.title}</h1>
  <p>{post.summary}</p>
</div>

<article bind:this={articleEl}>
	{@html post.html}
</article>

{#if showLargeImage}
	<div class="overlay-image" transition:fade={{ duration: 100 }} on:click={maybeCloseImage}>
		<img src={clickedImage} alt="large version of thing you clicked">
	</div>
{/if}

<style>
	article:global(.js-enabled p > img), 
	article:global(.js-enabled figure > img) {
		transition: .25s;
	}
	article:global(.js-enabled p > img:hover), 
	article:global(.js-enabled figure > img:hover) {
		transform: scale(1.03);
		cursor: pointer;
	}
	.overlay-image {
		display: flex;
		justify-content: center;
		align-items: center;

		position: fixed;
		top: 0; left: 0;
		width: 100%;
		height: 100%;

		background: #000a;
	}
	.overlay-image img {
		max-height: 80%;
		max-width: 80%;
	}
</style>