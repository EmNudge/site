<script context="module" lang="ts">
	import type { Load } from '@sveltejs/kit';
  import type { BlogPost } from '$lib/types';

	export const load: Load = async ({ fetch }) => {
		const res = await fetch('/blog/posts.json');

		if (res.ok) {
			const posts = await res.json();
			return {
				props: { posts }
			};
		}

		const { message } = await res.json();

		return {
			error: new Error(message)
		};
	};
</script>

<script lang="ts">
	import Stub from '$lib/Stub/index.svelte';

  export let posts: BlogPost[] = [];
</script>

<svelte:head>
  <title>Blog</title>
</svelte:head>

<h1>Blog</h1>

<div class="staggered">
  {#each posts as post, i}
    <post-container style="--index: {i}">
      <Stub 
        link="/blog/{post.slug}"
        meta="{post.date} • {post.minuteLength} minute read" 
        title={post.title}
        description={post.summary}
      />
    </post-container>  
  {/each}
</div>

<p class="feed">Subscribe to <a href="/blog/rss.xml">my rss feed <img src="icons/rss-feed-icon.png" alt="rss icon" /></a></p>

<style>
  h1 {
    margin: 80px 0;
    text-align: center;
  }
  .feed {
    margin-top: 50px;
    text-align: center;
  }
  .feed img {
    height: 15px;
  }
</style>