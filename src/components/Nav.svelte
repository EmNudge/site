<script lang="ts">
    export let routes: [string, string][];
    export let path;
    import { onMount } from 'svelte';
</script>

<nav>
    {#each routes as [route, name]}
        <a
            sveltekit:prefetch
            aria-current={path == route ? "page" : undefined}
            href={route}
            class:active={path == route}>{name}</a
        >
    {/each}
</nav>

<style>
    nav {
        display: flex;
    }
    nav a {
        text-decoration: none;
        color: var(--foreground);
        font-size: 1.2em;
        padding: 15px;
    }
    nav a.active {
        color: var(--active);
    }
</style>
