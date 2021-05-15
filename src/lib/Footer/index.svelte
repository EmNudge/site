<script>
  import IconHolder from '$lib/IconHolder/index.svelte';
  import { onMount } from 'svelte';

  const icons = [
    ['/icons/email.svg', 'email', 'mailto:calvin@kipperman.co'],
    ['/icons/github.svg', 'GitHub', 'https://github.com/EmNudge'],
    ['/icons/youtube.svg', 'YouTube', 'http://youtube.com/EmNudge'],
    ['/icons/twitter.svg', 'Twitter', 'http://twitter.com/emnudge'],
    ['/icons/devto.svg', 'Dev.to', 'https://dev.to/emnudge'],
  ];

  // if javascript is enabled, use a dynamic solution
  let year = 2021;
  onMount(() => {
    year = new Date().getFullYear();
  });
</script>

<footer>
  <IconHolder {icons} />
	<p>© Copyright 2019 - {year} Calvin Kipperman</p>
</footer>

<style>
  footer {
    border-top: 1px solid var(--light-bg);
    padding: 20px;
    margin: 40px auto;
    margin-top: 80px;

    max-width: 1000px;
    display: grid;
    grid-template-columns: 1fr auto;
  }
  
  @media screen and (max-width: 650px) {
    footer {
      grid-template-columns: 1fr;
      justify-items: center;
    }
  }
</style>