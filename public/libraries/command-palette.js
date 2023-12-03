/*
  Honestly, I didn't want to include this file here.
  Astro has been giving me a ton of problems defining it as a local TS module.
  Now I'm using JSDoc and putting it in the global scope. Oh well.
*/

let commandPaletteCreated = false;

/** @param {[string, string][]} postItems */
export function addCommandPaletteConditionally(postItems) {
  if (commandPaletteCreated) return;

  if (window.innerWidth < 800) {
    window.addEventListener('resize', function listener() {
      if (window.innerWidth > 800) {
        createCommandPalette(postItems);
        window.removeEventListener('resize', listener);
      }
    });

    return;
  }

  createCommandPalette(postItems);
}

/** @param {[string, string][]} postItems */
async function createCommandPalette(postItems) {
  if (commandPaletteCreated) return;

  // @ts-ignore - importing ninja keys file from public folder
  await import("/libraries/ninja-keys.min.js");

  const ninjaKeys = document.querySelector("ninja-keys");
  if (!ninjaKeys) {
    console.error('cannot find ninja keys');
    return;
  }

  if (window.matchMedia("(prefers-color-scheme: dark)").matches) {
    ninjaKeys.classList.add("dark");
  }

  const blogPosts = postItems.map(([title, url]) => ({
    id: url.slice(6),
    title,
    parent: "Post",
    handler: () => {
      window.location.pathname = url;
    },
  }));

  const nav = [
    ["Home", "/"],
    ["Blog", "/blog"],
    ["Article Bookmarks", "articles"],
    ["Book Bookmarks", "books"],
    ["Video Bookmarks", "videos"],
  ].map(([title, path]) => ({
    id: title,
    title,
    parent: "Page",
    handler: () => {
      window.location.pathname = path;
    },
  }));

  ninjaKeys.data = [
    {
      id: "Page",
      title: "Navigate To",
      children: nav.map((item) => item.id),
    },
    ...nav,
    {
      id: "Post",
      title: "Open Blog Post",
      children: blogPosts.map((item) => item.id),
    },
    ...blogPosts,
  ];

  commandPaletteCreated = true;
}