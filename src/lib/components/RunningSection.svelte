<script lang="ts">
  import { runningTiles } from "../stores";
  import WorkspaceButton from "./WorkspaceButton.svelte";
</script>

{#if $runningTiles.length > 0}
  <section class="running">
    <h2>Running</h2>
    <div class="grid">
      {#each $runningTiles as tile (tile.path)}
        <WorkspaceButton {tile} size="large" />
      {/each}
    </div>
  </section>
{/if}

<style>
  h2 { font-size: 1rem; color: #888; margin: 0.5rem 0; }
  .grid { display: flex; flex-wrap: wrap; gap: 0.75rem; }
  section.running {
    border-bottom: 1px solid #333;
    padding-bottom: 1rem;
    margin-bottom: 0.5rem;
  }
</style>
