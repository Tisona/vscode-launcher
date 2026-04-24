<script lang="ts">
  import { convertFileSrc } from "@tauri-apps/api/core";
  import type { TileModel } from "../stores";
  import { closeWorkspaceWindow, focusWindow, launch } from "../ipc";
  import { openMenu } from "../contextMenu";
  import { pushToast } from "../toasts";

  export let tile: TileModel;
  export let size: "large" | "small" = "small";

  $: iconUrl = tile.icon ? convertFileSrc(tile.icon) : null;

  async function handleClick() {
    try {
      if (tile.isRunning && tile.hwnd) {
        await focusWindow(tile.hwnd);
      } else {
        await launch(tile.path);
      }
    } catch (e) {
      pushToast(`Launch failed: ${e}`);
    }
  }

  async function handleClose() {
    if (!tile.hwnd) return;
    try {
      await closeWorkspaceWindow(tile.hwnd);
    } catch (e) {
      pushToast(`Close failed: ${e}`);
    }
  }
</script>

<div class="tile-wrap {size}" class:running={tile.isRunning}>
  <button
    type="button"
    class="tile"
    on:click={handleClick}
    on:contextmenu|preventDefault={(e) => openMenu(tile, e)}
    title={tile.path}
  >
    {#if iconUrl}
      <img class="icon" src={iconUrl} alt="" />
    {:else}
      <svg class="icon default" viewBox="0 0 24 24" aria-hidden="true">
        <rect x="3.5" y="5.5" width="17" height="13" rx="1.5"
              fill="none" stroke="currentColor" stroke-width="1.6"/>
        <polyline points="9.5,10 7,12 9.5,14"
                  fill="none" stroke="currentColor" stroke-width="1.6"
                  stroke-linecap="round" stroke-linejoin="round"/>
        <polyline points="14.5,10 17,12 14.5,14"
                  fill="none" stroke="currentColor" stroke-width="1.6"
                  stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
    {/if}
    <div class="text">
      <span class="label">{tile.displayName}</span>
      {#if size === "large" && tile.windowCount > 1}
        <span class="windows">×{tile.windowCount} windows</span>
      {/if}
    </div>
  </button>
  {#if tile.isRunning && tile.hwnd}
    <button
      type="button"
      class="close"
      title="Close workspace window"
      aria-label="Close workspace window"
      on:click={handleClose}
    >×</button>
  {/if}
</div>

<style>
  .tile-wrap {
    position: relative;
    display: inline-flex;
  }
  .tile-wrap.large { min-width: 15rem; }

  .tile {
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: 0.6rem;
    width: 100%;
    background: #2d2d2d;
    color: #d4d4d4;
    border: 1px solid #3c3c3c;
    border-radius: 4px;
    cursor: pointer;
    padding: 0.5rem 0.75rem;
    text-align: left;
    font: inherit;
  }
  .tile:hover { background: #3c3c3c; }
  .tile-wrap.running .tile { border-color: #0e639c; }

  .tile-wrap.large .tile {
    padding: 1rem 1.1rem;
    padding-right: 2rem;
    font-size: 1.05rem;
    min-height: 4.5rem;
    gap: 0.9rem;
  }

  .icon { width: 1.25rem; height: 1.25rem; color: #8a8a8a; flex-shrink: 0; }
  .tile-wrap.large .icon { width: 2.2rem; height: 2.2rem; }
  img.icon { object-fit: contain; }

  .text {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    flex: 1;
    min-width: 0;
  }

  .label {
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  }
  .tile-wrap.large .label { font-weight: 500; }

  .windows {
    font-size: 0.75rem; color: #9a9a9a;
  }

  .close {
    position: absolute;
    top: 4px;
    right: 4px;
    background: transparent;
    color: #888;
    border: none;
    font-size: 1.15rem;
    line-height: 1;
    cursor: pointer;
    padding: 0.15rem 0.4rem;
    border-radius: 3px;
  }
  .close:hover {
    background: #c04040;
    color: #fff;
  }
</style>
