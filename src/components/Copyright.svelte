<script>
    import { onMount } from "svelte";

    let year = 2022;
    onMount(() => {
        year = new Date().getFullYear();
    });
</script>

<p>© Copyright 2019 - {year} EmNudge</p>
