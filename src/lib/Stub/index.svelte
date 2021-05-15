<script lang="ts">
  export let link: string;
  export let meta: string;
  export let title: string;
  export let description: string | undefined = undefined;

	export let index: number = 0;

	export let prefetch = false;
</script>

{#if prefetch}
	<a href={link} class="post-stub" style="--index: {index}">
		<h5>{meta}</h5>
		<h2>{title}</h2>
		{#if description}
			<p>{description}</p>
		{/if}
	</a>
{:else}
	<a sveltekit:prefetch href={link} class="post-stub" style="--index: {index}">
		<h5>{meta}</h5>
		<h2>{title}</h2>
		{#if description}
			<p>{description}</p>
		{/if}
	</a>
{/if}

<style>
  a {
		padding: 20px;
		display: block;
		color: white;
    border-bottom: 1px solid #fff2;
	}
	a:hover {
		background: #fff1;
		cursor: pointer;
	}
	h5 {
		opacity: .5;
		font-weight: 100;
		margin-bottom: 5px;
	}
	p {
		margin: 0;
		padding-top: 10px;
		opacity: .8;
	}
</style>