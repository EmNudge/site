<script lang="ts">
import { onMount, onDestroy, tick } from "svelte";
import { fade, scale } from "svelte/transition";
import type { Command } from "./types";

export let postItems: readonly (readonly [string, string])[] = [];

let isOpen = false;
let searchQuery = "";
let selectedIndex = 0;
let currentParent: string | null = null;
let inputEl: HTMLInputElement;
let listEl: HTMLDivElement;
let isEnabled = false;

$: commands = buildCommands(postItems);
$: visibleCommands = getVisibleCommands(commands, searchQuery, currentParent);
$: selectedIndex = Math.min(selectedIndex, Math.max(0, visibleCommands.length - 1));

function buildCommands(posts: readonly (readonly [string, string])[]): Command[] {
  const blogPosts = posts.map(([title, url]) => ({
    id: url.slice(6),
    title,
    parent: "Post",
    handler: () => {
      window.location.pathname = url;
    },
  }));

  const nav: Command[] = [
    { id: "Home", title: "Home", parent: "Page", handler: () => { window.location.pathname = "/"; } },
    { id: "Blog", title: "Blog", parent: "Page", handler: () => { window.location.pathname = "/blog"; } },
    { id: "articles", title: "Article Bookmarks", parent: "Page", handler: () => { window.location.pathname = "/articles"; } },
    { id: "books", title: "Book Bookmarks", parent: "Page", handler: () => { window.location.pathname = "/books"; } },
    { id: "videos", title: "Video Bookmarks", parent: "Page", handler: () => { window.location.pathname = "/videos"; } },
  ];

  return [
    {
      id: "Theme",
      title: "Toggle Theme",
      handler: () => {
        document.documentElement.classList.toggle("light");
        const isLight = document.documentElement.classList.contains("light");
        localStorage.setItem("theme", isLight ? "light" : "dark");
      },
    },
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
}

function getVisibleCommands(
  allCommands: Command[],
  query: string,
  parent: string | null
): Command[] {
  if (parent) {
    return allCommands.filter((cmd) => cmd.parent === parent);
  }

  if (!query.trim()) {
    return allCommands.filter((cmd) => !cmd.parent);
  }

  const lowerQuery = query.toLowerCase();
  return allCommands.filter((cmd) => {
    if (cmd.children) return false;
    return cmd.title.toLowerCase().includes(lowerQuery);
  });
}

function handleGlobalKeydown(e: KeyboardEvent) {
  if ((e.metaKey || e.ctrlKey) && e.key === "k") {
    e.preventDefault();
    if (isOpen) {
      close();
    } else {
      open();
    }
  }
}

async function open() {
  isOpen = true;
  searchQuery = "";
  selectedIndex = 0;
  currentParent = null;
  await tick();
  inputEl?.focus();
}

function close() {
  isOpen = false;
}

function handleModalKeydown(e: KeyboardEvent) {
  switch (e.key) {
    case "Escape":
      if (currentParent) {
        currentParent = null;
        searchQuery = "";
        selectedIndex = 0;
      } else {
        close();
      }
      break;

    case "ArrowDown":
      e.preventDefault();
      selectedIndex = (selectedIndex + 1) % visibleCommands.length;
      scrollToSelected();
      break;

    case "ArrowUp":
      e.preventDefault();
      selectedIndex = (selectedIndex - 1 + visibleCommands.length) % visibleCommands.length;
      scrollToSelected();
      break;

    case "Enter":
      e.preventDefault();
      if (visibleCommands[selectedIndex]) {
        executeCommand(visibleCommands[selectedIndex]);
      }
      break;

    case "Backspace":
      if (!searchQuery && currentParent) {
        currentParent = null;
        selectedIndex = 0;
      }
      break;
  }
}

function scrollToSelected() {
  tick().then(() => {
    const selectedEl = listEl?.querySelector(".command-item.selected");
    selectedEl?.scrollIntoView({ block: "nearest" });
  });
}

function executeCommand(cmd: Command) {
  if (cmd.children) {
    currentParent = cmd.id;
    searchQuery = "";
    selectedIndex = 0;
  } else if (cmd.handler) {
    cmd.handler();
    close();
  }
}

function handleOverlayClick(e: MouseEvent) {
  if (e.target === e.currentTarget) {
    close();
  }
}

function handleItemClick(cmd: Command) {
  executeCommand(cmd);
}

function handleItemMouseEnter(index: number) {
  selectedIndex = index;
}

function checkWidth() {
  isEnabled = window.innerWidth > 800;
}

onMount(() => {
  checkWidth();
  window.addEventListener("keydown", handleGlobalKeydown);
  window.addEventListener("resize", checkWidth);
});

onDestroy(() => {
  window.removeEventListener("keydown", handleGlobalKeydown);
  window.removeEventListener("resize", checkWidth);
});
</script>

{#if isEnabled && isOpen}
  <div
    class="overlay"
    transition:fade={{ duration: 150 }}
    on:click={handleOverlayClick}
    on:keydown={handleModalKeydown}
    role="dialog"
    aria-modal="true"
    aria-label="Command palette"
  >
    <div class="palette" transition:scale={{ duration: 150, start: 0.95 }}>
      <input
        bind:this={inputEl}
        bind:value={searchQuery}
        type="text"
        class="search-input"
        placeholder={currentParent ? "Search..." : "Type a command or search..."}
        autocomplete="off"
        spellcheck="false"
      />

      {#if currentParent}
        <button class="back-button" on:click={() => { currentParent = null; searchQuery = ""; selectedIndex = 0; }}>
          <span class="back-arrow">&#8592;</span>
          Back
        </button>
      {/if}

      <div class="command-list" bind:this={listEl}>
        {#if visibleCommands.length === 0}
          <div class="no-results">No results found</div>
        {:else}
          {#each visibleCommands as cmd, index}
            <button
              class="command-item"
              class:selected={index === selectedIndex}
              on:click={() => handleItemClick(cmd)}
              on:mouseenter={() => handleItemMouseEnter(index)}
            >
              <span class="title">{cmd.title}</span>
              {#if cmd.children}
                <span class="chevron">&#8250;</span>
              {/if}
            </button>
          {/each}
        {/if}
      </div>

      <div class="footer">
        <span class="hint"><kbd>&#8593;</kbd><kbd>&#8595;</kbd> navigate</span>
        <span class="hint"><kbd>&#8629;</kbd> select</span>
        <span class="hint"><kbd>esc</kbd> close</span>
      </div>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background: rgba(0, 0, 0, 0.5);
    backdrop-filter: blur(4px);
    z-index: 100;
    display: flex;
    justify-content: center;
    align-items: flex-start;
    padding-top: 20vh;
  }

  .palette {
    width: 100%;
    max-width: 500px;
    margin: 0 1rem;
    background: var(--bg-light-transparent);
    backdrop-filter: blur(12px);
    border: 1px solid var(--light-bg);
    border-radius: 12px;
    overflow: hidden;
    box-shadow: 0 4px 30px rgba(0, 0, 0, 0.3);
    display: flex;
    flex-direction: column;
  }

  .search-input {
    width: 100%;
    padding: 16px 20px;
    background: transparent;
    border: none;
    border-bottom: 1px solid var(--light-bg);
    color: var(--foreground);
    font-size: var(--font-medium);
    outline: none;
    box-sizing: border-box;
  }

  .search-input::placeholder {
    color: var(--muted);
  }

  .back-button {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 20px;
    background: transparent;
    border: none;
    border-bottom: 1px solid var(--light-bg);
    color: var(--muted);
    font-size: var(--font-small);
    cursor: pointer;
    transition: color 0.15s ease;
  }

  .back-button:hover {
    color: var(--foreground);
  }

  .back-arrow {
    font-size: 14px;
  }

  .command-list {
    max-height: 300px;
    overflow-y: auto;
    padding: 8px;
  }

  .command-item {
    display: flex;
    align-items: center;
    width: 100%;
    padding: 10px 12px;
    border-radius: 6px;
    border: none;
    background: transparent;
    cursor: pointer;
    transition: background 0.15s ease;
    color: var(--foreground);
    font-size: var(--font-small);
    text-align: left;
  }

  .command-item:hover,
  .command-item.selected {
    background: var(--light-bg);
  }

  .command-item.selected {
    border-left: 2px solid var(--active);
    padding-left: 10px;
  }

  .command-item .title {
    flex: 1;
  }

  .command-item .chevron {
    opacity: 0.5;
    font-size: 16px;
  }

  .no-results {
    padding: 16px 12px;
    color: var(--muted);
    font-size: var(--font-small);
    text-align: center;
  }

  .footer {
    display: flex;
    gap: 16px;
    padding: 10px 20px;
    border-top: 1px solid var(--light-bg);
    justify-content: center;
  }

  .hint {
    display: flex;
    align-items: center;
    gap: 4px;
    color: var(--muted);
    font-size: 12px;
  }

  kbd {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 20px;
    padding: 2px 5px;
    background: var(--light-bg);
    border-radius: 4px;
    font-family: inherit;
    font-size: 11px;
  }

  /* Light mode styles */
  @media (prefers-color-scheme: light) {
    .palette {
      background: rgba(255, 255, 255, 0.95);
      border: 1px solid #ddd;
      box-shadow: 0 4px 30px rgba(0, 0, 0, 0.15);
    }
    .search-input {
      border-bottom-color: #eee;
    }
    .back-button {
      border-bottom-color: #eee;
    }
    .command-item:hover,
    .command-item.selected {
      background: #f5f5f5;
    }
    .footer {
      border-top-color: #eee;
    }
    kbd {
      background: #f0f0f0;
      border: 1px solid #ddd;
    }
  }

  :global(:root.light) .palette {
    background: rgba(255, 255, 255, 0.95);
    border: 1px solid #ddd;
    box-shadow: 0 4px 30px rgba(0, 0, 0, 0.15);
  }
  :global(:root.light) .search-input {
    border-bottom-color: #eee;
  }
  :global(:root.light) .back-button {
    border-bottom-color: #eee;
  }
  :global(:root.light) .command-item:hover,
  :global(:root.light) .command-item.selected {
    background: #f5f5f5;
  }
  :global(:root.light) .footer {
    border-top-color: #eee;
  }
  :global(:root.light) kbd {
    background: #f0f0f0;
    border: 1px solid #ddd;
  }
</style>
