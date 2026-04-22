<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import type { UnlistenFn } from "@tauri-apps/api/event";
  import { getConfig, getRunning, getWorkspaces, onRunningUpdated } from "./lib/ipc";
  import { config, running, workspaces } from "./lib/stores";
  import EmptyState from "./lib/components/EmptyState.svelte";

  let loading = true;
  let unlisten: UnlistenFn | null = null;

  onMount(async () => {
    try {
      const cfg = await getConfig();
      config.set(cfg);
      if (cfg.root_folder) {
        try {
          workspaces.set(await getWorkspaces());
        } catch (e) {
          console.error("scan failed", e);
        }
      }
      running.set(new Set(await getRunning()));
      unlisten = await onRunningUpdated((paths) => running.set(new Set(paths)));
    } catch (e) {
      console.error("init failed", e);
    } finally {
      loading = false;
    }
  });

  onDestroy(() => {
    unlisten?.();
  });
</script>

<main class="app">
  {#if loading}
    <div class="loading">Loading…</div>
  {:else if !$config.root_folder}
    <EmptyState />
  {:else}
    <div class="placeholder">Main view — sections coming in next tasks.</div>
  {/if}
</main>

<style>
  .app {
    min-height: 100vh;
    background: #1e1e1e;
    color: #d4d4d4;
    font-family: system-ui, -apple-system, sans-serif;
    padding: 1rem;
  }
  .loading, .placeholder {
    display: grid;
    place-items: center;
    min-height: 50vh;
    color: #888;
  }
</style>
