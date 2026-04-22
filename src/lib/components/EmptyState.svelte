<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { setRootFolder, getWorkspaces } from "../ipc";
  import { config, workspaces } from "../stores";

  async function pickFolder() {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected !== "string") return;
    const newCfg = await setRootFolder(selected);
    config.set(newCfg);
    workspaces.set(await getWorkspaces());
  }
</script>

<div class="empty">
  <h1>VSCode Launcher</h1>
  <p>Pick a folder containing <code>.code-workspace</code> files.</p>
  <button type="button" on:click={pickFolder}>Pick your workspaces folder</button>
</div>

<style>
  .empty {
    display: grid;
    place-items: center;
    gap: 1rem;
    min-height: 70vh;
    text-align: center;
  }
  button {
    background: #0e639c;
    color: white;
    border: none;
    padding: 0.75rem 1.5rem;
    font-size: 1rem;
    border-radius: 4px;
    cursor: pointer;
  }
  button:hover { background: #1177bb; }
  code { background: #2d2d2d; padding: 0.1rem 0.3rem; border-radius: 3px; }
</style>
