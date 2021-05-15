<script>
  import { page } from '$app/stores';

  const routes = [
    ['/', 'Home'],
    ['/blog', 'Blog'],
    ['/reads', 'Reads']
  ];
</script>

<header>
  <a href="/" class="logo">
    <img src="/icons/logo.svg" alt="logo" class="dark-mode">
  </a>
  <nav>
    {#each routes as [route, name]}
      <a 
        sveltekit:prefetch 
        aria-current={$page.path == route ? 'page' : undefined}
        href={route}
        class:active={$page.path == route}
      >{name}</a>
    {/each}
  </nav>    
</header>

<style>
  header {
    padding: 20px;
    max-width: 800px;
    margin: 0 auto;  
    display: grid;
    grid-template-columns: 1fr auto;
    align-items: center;
  }
  img {
    height: 40px;
  }
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
  .logo {
    display: flex;
    align-items: center;
  }
</style>