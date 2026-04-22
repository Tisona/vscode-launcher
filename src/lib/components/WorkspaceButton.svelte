<script lang="ts">
  import { convertFileSrc } from "@tauri-apps/api/core";
  import type { TileModel } from "../stores";
  import { launch } from "../ipc";

  export let tile: TileModel;
  export let size: "large" | "small" = "small";

  $: iconUrl = tile.icon ? convertFileSrc(tile.icon) : null;

  async function handleClick() {
    try {
      await launch(tile.path);
    } catch (e) {
      console.error("launch failed", e);
      // Toast in Task 14.
    }
  }
</script>

<button
  type="button"
  class="tile {size}"
  class:running={tile.isRunning}
  on:click={handleClick}
  title={tile.path}
>
  {#if iconUrl}
    <img class="icon" src={iconUrl} alt="" />
  {:else}
    <span class="icon placeholder">📁</span>
  {/if}
  <span class="label">{tile.displayName}</span>
</button>

<style>
  .tile {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    background: #2d2d2d;
    color: #d4d4d4;
    border: 1px solid #3c3c3c;
    border-radius: 4px;
    cursor: pointer;
    padding: 0.5rem 0.75rem;
    text-align: left;
  }
  .tile:hover { background: #3c3c3c; }
  .tile.running { border-color: #0e639c; }
  .tile.large {
    flex-direction: column;
    align-items: flex-start;
    min-width: 11rem;
    min-height: 6rem;
    padding: 1rem;
    font-size: 1.1rem;
  }
  .tile.large .icon { font-size: 2rem; }
  .tile.large img.icon { width: 2.5rem; height: 2.5rem; }
  .tile.small img.icon { width: 1.25rem; height: 1.25rem; }
  .icon.placeholder { display: inline-block; }
  .label { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
</style>
