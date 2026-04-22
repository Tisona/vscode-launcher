<script lang="ts">
  import { pinnedTiles } from "../stores";
  import WorkspaceButton from "./WorkspaceButton.svelte";
</script>

{#if $pinnedTiles.length > 0}
  <section class="pinned">
    <h2>Pinned</h2>
    <div class="grid">
      {#each $pinnedTiles as tile (tile.path)}
        <WorkspaceButton {tile} size="small" />
      {/each}
    </div>
  </section>
{/if}

<style>
  h2 { font-size: 1rem; color: #888; margin: 1rem 0 0.5rem; }
  .grid { display: flex; flex-wrap: wrap; gap: 0.5rem; }
</style>
