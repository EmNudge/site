<script context="module" lang="ts">
	import type { Load } from '@sveltejs/kit';
  import type { Read } from '$lib/types';

	export const load: Load = async ({ fetch }) => {
		const res = await fetch('/reads.json');

		if (res.ok) {
			const reads = await res.json();
			return {
				props: { reads }
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

  export let reads: Read[] = [];
</script>

<svelte:head>
  <title>What I'm Reading</title>
</svelte:head>

<h1>What I'm Reading</h1>

<div class="staggered">
  {#each reads as read, i}
    <post-container style="--index: {i}">
      <Stub 
        link={read.link} 
        meta="{read.author} • {read.createdAt}" 
        title={read.title}
        hint={read.description}
      />
    </post-container>
  {/each}
</div>

<style>
  h1 {
    margin: 80px 0;
    text-align: center;
  }
</style>