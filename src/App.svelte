<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import type { UnlistenFn } from "@tauri-apps/api/event";
  import { getConfig, getWorkspaces, onRunningUpdated } from "./lib/ipc";
  import { applyStatuses, config, workspaces } from "./lib/stores";
  import EmptyState from "./lib/components/EmptyState.svelte";
  import PinnedSection from "./lib/components/PinnedSection.svelte";
  import AllSection from "./lib/components/AllSection.svelte";
  import RunningSection from "./lib/components/RunningSection.svelte";
  import ContextMenu from "./lib/components/ContextMenu.svelte";
  import SettingsDialog from "./lib/components/SettingsDialog.svelte";

  let loading = true;
  let settingsOpen = false;
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
      unlisten = await onRunningUpdated((statuses) => applyStatuses(statuses));
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
  <button
    class="gear"
    type="button"
    on:click={() => (settingsOpen = true)}
    aria-label="Settings"
  >⚙</button>

  {#if loading}
    <div class="loading">Loading…</div>
  {:else if !$config.root_folder}
    <EmptyState />
  {:else}
    <RunningSection />
    <PinnedSection />
    <AllSection />
  {/if}
  <ContextMenu />
  <SettingsDialog isOpen={settingsOpen} onClose={() => (settingsOpen = false)} />
</main>

<style>
  .app {
    min-height: 100vh;
    background: #1e1e1e;
    color: #d4d4d4;
    font-family: system-ui, -apple-system, sans-serif;
    padding: 1rem;
  }
  .loading {
    display: grid;
    place-items: center;
    min-height: 50vh;
    color: #888;
  }
  .gear {
    position: fixed;
    top: 0.5rem;
    right: 0.5rem;
    background: transparent;
    color: #888;
    border: none;
    font-size: 1.25rem;
    cursor: pointer;
    z-index: 10;
    padding: 0.2rem 0.4rem;
    line-height: 1;
  }
  .gear:hover { color: #d4d4d4; }
</style>
