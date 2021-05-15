<script>
  import IconHolder from '$lib/IconHolder/index.svelte';
  const icons = [
    ['/icons/github.svg', 'GitHub', 'https://github.com/EmNudge'],
    ['/icons/youtube.svg', 'YouTube', 'http://youtube.com/EmNudge'],
    ['/icons/twitter.svg', 'Twitter', 'http://twitter.com/emnudge'],
  ];

  let titleIndex = 0;
  const titles = [
    'Local Nobody',
    'Software Engine Ear',
    'Perfektonist',
    'Public Nuisance',
    'Disorderly Conductor',
    'Voice Over Easy'
  ]
  function changeTitle() {
    titleIndex = (titleIndex + 1) % titles.length;
  }
</script>

<section>
  <div class="name">
    <IconHolder {icons} small opacity={.5} staggered />
    <h1>Calvin Kipperman</h1>
    <div class="special-container">
      {#key titleIndex}
        <h2 on:click={changeTitle}>{titles[titleIndex]}</h2>
      {/key}
    </div>
  </div>
</section>

<style>
  section {
    display: flex;
    justify-content: center;
    padding: 20px;
    margin: 40px auto;
  }
  .name {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
  }
  h1 {
    font-size: clamp(2.5em, 5vw, 4em); 
    font-weight: 200;
  }
  .special-container {
    width: 100%;
  }

  @keyframes pop-in {
    from { transform: scale(1.1); }
    to { transform: scale(1); }
  }
  h2 {
    opacity: .8;
    width: 100%;
    cursor: grab;
    user-select: none;
    width: max-content;
    animation: pop-in 250ms;
  }
  h2:active {
    transform: scale(.9);
  }

</style>