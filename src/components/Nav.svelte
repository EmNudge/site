<script lang="ts">
    export let routes: [string, string][];
    export let path;
</script>

<nav>
    {#each routes as [route, name]}
        <a
            rel="prefetch"
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
    a {
        text-decoration: none;
        color: var(--foreground);
        font-size: 1.2em;
        padding: 15px;
    }
    a.active {
        color: var(--active);
    }
</style>
