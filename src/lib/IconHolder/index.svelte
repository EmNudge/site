<script>
  export let icons = [
    ['/icons/email.svg', 'email', 'mailto:calvin@kipperman.co'],
    ['/icons/github.svg', 'GitHub', 'https://github.com/EmNudge'],
    ['/icons/youtube.svg', 'YouTube', 'http://youtube.com/EmNudge'],
    ['/icons/twitter.svg', 'Twitter', 'http://twitter.com/emnudge'],
    ['/icons/devto.svg', 'Dev.to', 'https://dev.to/emnudge'],
  ];
  export let small = false;
  export let opacity = .8;

  export let staggered = false;
</script>

<div style="--opacity: {opacity}" class:staggered>
  {#each icons as [iconUrl, name, link], i}
    <a href={link} style="--index: {i}">
      <img 
        class="dark-mode" 
        class:small 
        src={iconUrl} 
        alt={name} />
    </a>
  {/each}
</div>

<style>
  div {
    display: flex;
  }
  a {
    text-decoration: none;
    display: flex;
    align-items: center;
  }
  a img {
    height: 20px;
    padding: 10px;
    opacity: var(--opacity);
    transition: .2s;
  }
  a:hover img {
    opacity: 1;
  }
  img.small {
    height: 15px;
    padding: 8px;
  }
</style>

