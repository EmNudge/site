<script context="module" lang="ts">
	import type { Load } from '@sveltejs/kit';
  import type { Video, BlogPostMeta, Read } from '$lib/types';

	export const load: Load = async ({ fetch }) => {
		const res = await fetch('/index.json');

		if (res.ok) {
			const { videos, posts, reads } = await res.json();
			return {
				props: { videos, posts, reads }
			};
		}

		const { message } = await res.json();

		return {
			error: new Error(message)
		};
	};
</script>

<script lang="ts">
  import Name from '$lib/sections/Name/index.svelte';
	import TitleSection from '$lib/TitleSection/index.svelte';
	import Stub from '$lib/Stub/index.svelte';

  export let videos: Video[] = [];
	export let posts: BlogPostMeta[] = [];
	export let reads: Read[] = [];
</script>

<svelte:head>
	<title>Home</title>
</svelte:head>

<Name />

<TitleSection title="Blog" link="/blog" staggered>
	{#each posts as post, i}
		<Stub 
			index={i}
			link="/blog/{post.slug}"
			meta="{post.date} • {post.minuteLength} minute read" 
			title={post.title}
			description={post.summary}
		/>
	{/each}
</TitleSection>

<TitleSection title="Videos" link="https://www.youtube.com/emnudge" staggered>
	<div class="videos">
		{#each videos.slice(0, 4) as video, i}
			<a href="https://www.youtube.com/watch?v={video.id}" class="video" style="--index: {i}">
				<img alt={video.title} src={video.thumbnails.medium.url} />
				<h3>{@html video.title}</h3>
			</a>
		{/each}
	</div>
</TitleSection>

<TitleSection title="What I'm Reading" link="/reads" staggered>
	{#each reads as read, i}
		<Stub 
			index={i}
			link={read.link} 
			meta="{read.author} • {read.createdAt}" 
			title={read.title}
		/>
	{/each}
</TitleSection>


<style>
	.videos {
		display: flex;
		justify-content: space-around;
		flex-wrap: wrap;
		margin: 0 auto;
	}
	.video {
		box-sizing: border-box;
		display: block;
		margin: 20px;
		position: relative;
		width: 320px;
		transition: .25s;
	}
	.video a {
		color: var(--forground);
	}
	.video h3 {
		margin-top: 10px;
		font-weight: 100;
	}
	.video:hover {
		transform: scale(1.05);
		opacity: .8;
	}
	.video img {
		max-width: 320px;
		max-height: 180px;
		width: auto;
		height: auto;
	}
</style>
