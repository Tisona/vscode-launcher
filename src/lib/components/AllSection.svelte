<script lang="ts">
  import { allTiles } from "../stores";
  import WorkspaceButton from "./WorkspaceButton.svelte";
</script>

<section class="all">
  <h2>All workspaces</h2>
  {#if $allTiles.length === 0}
    <p class="empty">No <code>.code-workspace</code> files in the configured folder.</p>
  {:else}
    <div class="grid">
      {#each $allTiles as tile (tile.path)}
        <WorkspaceButton {tile} size="small" />
      {/each}
    </div>
  {/if}
</section>

<style>
  h2 { font-size: 1rem; color: #888; margin: 1rem 0 0.5rem; }
  .grid { display: flex; flex-wrap: wrap; gap: 0.5rem; }
  .empty { color: #666; font-size: 0.9rem; }
</style>
