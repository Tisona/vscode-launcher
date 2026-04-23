<script lang="ts">
  import { convertFileSrc } from "@tauri-apps/api/core";
  import type { TileModel } from "../stores";
  import { launch } from "../ipc";
  import { openMenu } from "../contextMenu";
  import { pushToast } from "../toasts";

  export let tile: TileModel;
  export let size: "large" | "small" = "small";

  $: iconUrl = tile.icon ? convertFileSrc(tile.icon) : null;

  async function handleClick() {
    try {
      await launch(tile.path);
    } catch (e) {
      pushToast(`Launch failed: ${e}`);
    }
  }

  function formatBytes(n: number): string {
    if (n >= 1_073_741_824) return `${(n / 1_073_741_824).toFixed(1)} GB`;
    if (n >= 1_048_576) return `${(n / 1_048_576).toFixed(0)} MB`;
    if (n >= 1024) return `${(n / 1024).toFixed(0)} KB`;
    return `${n} B`;
  }

  function sparklinePoints(samples: number[]): string {
    if (samples.length === 0) return "";
    const W = 100;
    const H = 18;
    const pad = 1;
    const max = Math.max(...samples, 1);
    const n = samples.length;
    return samples
      .map((v, i) => {
        const x = n === 1 ? W / 2 : (i / (n - 1)) * W;
        const y = H - pad - (v / max) * (H - pad * 2);
        return `${x.toFixed(2)},${y.toFixed(2)}`;
      })
      .join(" ");
  }

  $: cpuPoints = sparklinePoints(tile.cpuHistory);
  $: ramPoints = sparklinePoints(tile.ramHistory);
  $: cpuDisplay = `${Math.round(tile.cpu)}%`;
  $: ramDisplay = formatBytes(tile.ramBytes);
</script>

<button
  type="button"
  class="tile {size}"
  class:running={tile.isRunning}
  on:click={handleClick}
  on:contextmenu|preventDefault={(e) => openMenu(tile, e)}
  title={tile.path}
>
  <div class="header">
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
    <span class="label">{tile.displayName}</span>
    {#if size === "large" && tile.windowCount > 1}
      <span class="windows">×{tile.windowCount}</span>
    {/if}
  </div>

  {#if size === "large" && tile.isRunning}
    <div class="metrics">
      <span class="m-label">CPU</span>
      <svg class="sparkline" viewBox="0 0 100 18" preserveAspectRatio="none" aria-hidden="true">
        <polyline class="spark-cpu" points={cpuPoints} />
      </svg>
      <span class="m-value">{cpuDisplay}</span>

      <span class="m-label">RAM</span>
      <svg class="sparkline" viewBox="0 0 100 18" preserveAspectRatio="none" aria-hidden="true">
        <polyline class="spark-ram" points={ramPoints} />
      </svg>
      <span class="m-value">{ramDisplay}</span>
    </div>
  {/if}
</button>

<style>
  .tile {
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: 0.5rem;
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
  .tile.running { border-color: #0e639c; }

  .tile.large {
    flex-direction: column;
    align-items: stretch;
    min-width: 14rem;
    min-height: 6rem;
    padding: 0.9rem 1rem;
    font-size: 1rem;
    gap: 0.35rem;
  }
  .tile.large .header {
    display: flex; align-items: center; gap: 0.5rem;
  }
  .tile.small .header {
    display: flex; align-items: center; gap: 0.5rem;
  }

  .icon { width: 1.25rem; height: 1.25rem; color: #8a8a8a; flex-shrink: 0; }
  .tile.large .icon { width: 1.8rem; height: 1.8rem; }
  img.icon { object-fit: contain; }

  .label { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .tile.large .label { font-weight: 500; flex: 1; }

  .windows {
    font-size: 0.75rem; color: #888;
    background: #1a1a1a; border: 1px solid #3c3c3c;
    padding: 0.05rem 0.3rem; border-radius: 3px;
  }

  .metrics {
    display: grid;
    grid-template-columns: 2.3rem 1fr auto;
    gap: 0.35rem;
    align-items: center;
    font-size: 0.75rem;
    color: #888;
    font-variant-numeric: tabular-nums;
  }
  .m-label { text-transform: uppercase; letter-spacing: 0.05em; color: #6e6e6e; }
  .m-value { color: #c0c0c0; text-align: right; min-width: 3.5rem; }
  .sparkline { display: block; width: 100%; height: 18px; }
  .sparkline .spark-cpu { stroke: #e5c07b; fill: none; stroke-width: 1.2; }
  .sparkline .spark-ram { stroke: #61afef; fill: none; stroke-width: 1.2; }
</style>
