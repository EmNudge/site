<script lang="ts">
import { createEventDispatcher } from "svelte";
const dispatch = createEventDispatcher();

export let showControls = false;
export let totalProgress = 0;

let containerEl: HTMLDivElement;
let isDragging = false;
let offsetY = 0;
let positionY = 50; // percentage from top

function isInteractiveElement(el: EventTarget | null): boolean {
    if (!el || !(el instanceof HTMLElement)) return false;
    const tag = el.tagName.toLowerCase();
    return tag === 'button' || tag === 'input' || tag === 'select' || el.closest('button') !== null;
}

function handleMouseDown(e: MouseEvent) {
    if (window.innerWidth < 900) return;
    if (isInteractiveElement(e.target)) return;

    isDragging = true;
    offsetY = e.clientY - containerEl.getBoundingClientRect().top;
    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
}

function handleMouseMove(e: MouseEvent) {
    if (!isDragging) return;
    e.preventDefault();

    const containerHeight = containerEl.offsetHeight;
    const windowHeight = window.innerHeight;
    const minY = containerHeight / 2;
    const maxY = windowHeight - containerHeight / 2;

    let newY = e.clientY - offsetY + containerHeight / 2;
    newY = Math.max(minY, Math.min(maxY, newY));

    positionY = (newY / windowHeight) * 100;
}

function handleMouseUp() {
    isDragging = false;
    document.removeEventListener('mousemove', handleMouseMove);
    document.removeEventListener('mouseup', handleMouseUp);
}

function handleTouchStart(e: TouchEvent) {
    if (window.innerWidth < 900) return;
    if (isInteractiveElement(e.target)) return;

    isDragging = true;
    offsetY = e.touches[0].clientY - containerEl.getBoundingClientRect().top;
}

function handleTouchMove(e: TouchEvent) {
    if (!isDragging || window.innerWidth < 900) return;

    const containerHeight = containerEl.offsetHeight;
    const windowHeight = window.innerHeight;
    const minY = containerHeight / 2;
    const maxY = windowHeight - containerHeight / 2;

    let newY = e.touches[0].clientY - offsetY + containerHeight / 2;
    newY = Math.max(minY, Math.min(maxY, newY));

    positionY = (newY / windowHeight) * 100;
}

function handleTouchEnd() {
    isDragging = false;
}
</script>

<div
    class="controls speak-aloud-controls"
    class:hide={!showControls}
    class:dragging={isDragging}
    bind:this={containerEl}
    style="--position-y: {positionY}%"
    on:mousedown={handleMouseDown}
    on:touchstart={handleTouchStart}
    on:touchmove={handleTouchMove}
    on:touchend={handleTouchEnd}
    role="region"
    aria-label="Audio controls">
    <button class="close-btn" on:click={() => dispatch("close")} aria-label="Close audio player">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path d="M18 6L6 18M6 6l12 12" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
    </button>
    <div class="progress-bar">
        <div class="progress-fill" style="--percentage: {totalProgress * 100}%;" />
    </div>
    <div class="controls-content">
        <slot />
    </div>
</div>

<style>
    .controls {
        display: flex;
        flex-direction: column;
        gap: 12px;
        box-sizing: border-box;
        padding: 16px;
        position: fixed;
        background: var(--bg-light-transparent);
        backdrop-filter: blur(12px);
        border: 1px solid var(--light-bg);
        z-index: 10;
        transition: transform 0.25s ease, opacity 0.25s ease;
    }

    .close-btn {
        position: absolute;
        display: flex;
        align-items: center;
        justify-content: center;
        background: var(--light-bg);
        border: none;
        border-radius: 50%;
        width: 24px;
        height: 24px;
        cursor: pointer;
        color: var(--foreground);
        opacity: 0.7;
        transition: opacity 0.15s ease, background 0.15s ease;
    }

    .close-btn:hover {
        opacity: 1;
        background: var(--light-active);
    }

    /* Desktop: sidebar on the right */
    @media (min-width: 900px) {
        .controls {
            top: var(--position-y);
            right: 16px;
            transform: translateY(-50%);
            width: 160px;
            padding: 16px;
            gap: 12px;
            border-radius: 12px;
            overflow: visible;
            cursor: grab;
            user-select: none;
        }
        .controls.dragging {
            cursor: grabbing;
            transition: none;
        }
        .controls.hide {
            transform: translateY(-50%) translateX(200px);
            opacity: 0;
            pointer-events: none;
        }
        .close-btn {
            top: -8px;
            right: -8px;
            cursor: pointer;
        }
        .controls-content {
            display: flex;
            flex-direction: column;
            align-items: center;
            gap: 12px;
            width: 100%;
        }
        .progress-bar {
            width: 100%;
            height: 3px;
            background: var(--light-bg);
            border-radius: 2px;
            overflow: hidden;
        }
        .progress-fill {
            height: 100%;
            width: var(--percentage);
            background: var(--active);
            border-radius: 2px;
            transition: width 0.1s linear;
        }
    }

    /* Mobile: bottom bar */
    @media (max-width: 899px) {
        .controls {
            bottom: 0;
            left: 0;
            right: 0;
            padding: 12px 16px;
            border-radius: 12px 12px 0 0;
            border-bottom: none;
        }
        .controls.hide {
            transform: translateY(100%);
            opacity: 0;
            pointer-events: none;
        }
        .close-btn {
            top: -12px;
            right: 12px;
        }
        .controls-content {
            display: flex;
            justify-content: center;
            align-items: center;
            gap: 16px;
        }
        .progress-bar {
            width: 100%;
            height: 4px;
            background: var(--light-bg);
            border-radius: 2px;
            overflow: hidden;
        }
        .progress-fill {
            height: 100%;
            width: var(--percentage);
            background: var(--active);
            border-radius: 2px;
            transition: width 0.1s linear;
        }
    }
</style>
