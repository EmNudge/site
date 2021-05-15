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

	export let post: BlogPost;
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

<article>
	{@html post.html}
</article>