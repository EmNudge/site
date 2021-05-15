<script>
  import Header from '$lib/Header/index.svelte';
  import Footer from '$lib/Footer/index.svelte';
  import '../app.css';
</script>

<Header />

<main>
	<slot />
</main>

<Footer />

<style>
  main {
    max-width: 800px;
    margin: 0 auto;
    padding: 5px;
  }
</style>