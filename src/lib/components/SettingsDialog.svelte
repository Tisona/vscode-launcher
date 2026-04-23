<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { getWorkspaces, resolvedCodeBinary, setRootFolder } from "../ipc";
  import { config, workspaces } from "../stores";

  export let isOpen = false;
  export let onClose: () => void;

  let codeBinary = "";
  $: if (isOpen) {
    resolvedCodeBinary().then((p) => (codeBinary = p ?? "(not found)"));
  }

  async function pick() {
    try {
      const selected = await open({ directory: true, multiple: false });
      if (typeof selected !== "string") return;
      const newCfg = await setRootFolder(selected);
      config.set(newCfg);
      workspaces.set(await getWorkspaces());
    } catch (e) {
      console.error("pick failed", e);
    }
  }

  async function rescan() {
    if (!$config.root_folder) return;
    try {
      workspaces.set(await getWorkspaces());
    } catch (e) {
      console.error("rescan failed", e);
    }
  }
</script>

{#if isOpen}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div class="backdrop" on:click={onClose} role="presentation">
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_interactive_supports_focus -->
    <div
      class="dialog"
      on:click|stopPropagation
      role="dialog"
      aria-modal="true"
    >
      <header>
        <h2>Settings</h2>
        <button type="button" class="close" on:click={onClose} aria-label="Close">✕</button>
      </header>
      <dl>
        <dt>Workspaces folder</dt>
        <dd>
          <code>{$config.root_folder ?? "(not set)"}</code>
          <button type="button" on:click={pick}>Change…</button>
          <button type="button" on:click={rescan} disabled={!$config.root_folder}>Rescan</button>
        </dd>
        <dt>VSCode CLI</dt>
        <dd><code>{codeBinary}</code></dd>
      </dl>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed; inset: 0;
    background: rgba(0,0,0,0.5);
    display: grid; place-items: center;
    z-index: 200;
  }
  .dialog {
    background: #252526;
    border: 1px solid #454545;
    border-radius: 6px;
    min-width: 28rem;
    max-width: 90vw;
    padding: 1rem;
  }
  header { display: flex; justify-content: space-between; align-items: center; }
  header h2 { margin: 0; font-size: 1.1rem; }
  header .close {
    background: transparent; border: none;
    color: #d4d4d4; cursor: pointer; font-size: 1rem;
    padding: 0.2rem 0.4rem;
  }
  header .close:hover { color: #fff; }
  dl { margin: 1rem 0 0; }
  dt { color: #888; font-size: 0.85rem; margin-top: 0.75rem; }
  dd {
    margin: 0.25rem 0 0;
    display: flex; gap: 0.5rem; align-items: center; flex-wrap: wrap;
  }
  code {
    background: #1e1e1e;
    padding: 0.2rem 0.4rem; border-radius: 3px;
    word-break: break-all;
    font-size: 0.85rem;
  }
  dd button {
    background: #0e639c; color: white; border: none;
    padding: 0.3rem 0.6rem; border-radius: 3px; cursor: pointer;
    font: inherit;
  }
  dd button:hover { background: #1177bb; }
  dd button:disabled { background: #555; cursor: not-allowed; }
</style>
