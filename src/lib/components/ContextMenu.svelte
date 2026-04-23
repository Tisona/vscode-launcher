<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { revealItemInDir } from "@tauri-apps/plugin-opener";
  import { contextMenu, closeMenu } from "../contextMenu";
  import { setIcon, setPinned } from "../ipc";
  import { config } from "../stores";
  import { pushToast } from "../toasts";

  async function togglePin() {
    const s = $contextMenu;
    if (!s) return;
    try {
      const newCfg = await setPinned(s.tile.path, !s.tile.isPinned);
      config.set(newCfg);
    } catch (e) {
      pushToast(`Pin failed: ${e}`);
    }
    closeMenu();
  }

  async function pickIcon() {
    const s = $contextMenu;
    if (!s) return;
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: "Images", extensions: ["png", "svg", "jpg", "jpeg"] }],
      });
      if (typeof selected !== "string") { closeMenu(); return; }
      const newCfg = await setIcon(s.tile.path, selected);
      config.set(newCfg);
    } catch (e) {
      pushToast(`Set icon failed: ${e}`);
    }
    closeMenu();
  }

  async function clearIcon() {
    const s = $contextMenu;
    if (!s) return;
    try {
      const newCfg = await setIcon(s.tile.path, null);
      config.set(newCfg);
    } catch (e) {
      pushToast(`Clear icon failed: ${e}`);
    }
    closeMenu();
  }

  async function reveal() {
    const s = $contextMenu;
    if (!s) return;
    try {
      await revealItemInDir(s.tile.path);
    } catch (e) {
      pushToast(`Reveal failed: ${e}`);
    }
    closeMenu();
  }

  function onWindowClick(e: MouseEvent) {
    const target = e.target as HTMLElement;
    if (!target.closest(".context-menu")) closeMenu();
  }

  $: hasOverride = $contextMenu
    ? Object.prototype.hasOwnProperty.call($config.icons, $contextMenu.tile.path)
    : false;
</script>

<svelte:window on:click={onWindowClick} on:contextmenu={onWindowClick} />

{#if $contextMenu}
  <ul
    class="context-menu"
    style="left: {$contextMenu.x}px; top: {$contextMenu.y}px;"
    role="menu"
  >
    <li>
      <button type="button" on:click={togglePin}>
        {$contextMenu.tile.isPinned ? "Unpin" : "Pin"}
      </button>
    </li>
    <li>
      <button type="button" on:click={pickIcon}>Set icon…</button>
    </li>
    {#if hasOverride}
      <li>
        <button type="button" on:click={clearIcon}>Clear icon override</button>
      </li>
    {/if}
    <li>
      <button type="button" on:click={reveal}>Reveal in file manager</button>
    </li>
  </ul>
{/if}

<style>
  .context-menu {
    position: fixed;
    background: #252526;
    border: 1px solid #454545;
    border-radius: 4px;
    padding: 0.25rem 0;
    list-style: none;
    margin: 0;
    min-width: 12rem;
    z-index: 100;
    box-shadow: 0 4px 8px rgba(0,0,0,0.4);
  }
  li { margin: 0; padding: 0; }
  button {
    width: 100%;
    background: transparent;
    color: #d4d4d4;
    border: none;
    padding: 0.4rem 0.75rem;
    text-align: left;
    cursor: pointer;
    font: inherit;
  }
  button:hover { background: #094771; }
</style>
