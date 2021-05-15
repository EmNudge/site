<script>
  export let title;
  export let link;
  export let staggered = false;
</script>

<section>
  <h1>{title}</h1>
  
  <div class:staggered>
    <slot />
  </div>
  
  {#if link}
    <a href={link} class="more">More</a>
  {/if}
</section>

<style>
  section {
    margin: 80px 0;
  }
  .more {
    font-size: 1.5em;
    margin-top: 20px;
    padding: 15px;
    text-align: center;
    display: block;
  }
  .more:hover {
    background: var(--light-bg);
  }
  h1 {
		margin: 20px 10px;
    font-size: 2.2em;
	}
</style>

