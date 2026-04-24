<script lang="ts">
  import { running, totalCpuHistory, totalRamHistory } from "../stores";

  function formatBytes(n: number): string {
    if (n >= 1_073_741_824) return `${(n / 1_073_741_824).toFixed(1)} GB`;
    if (n >= 1_048_576) return `${(n / 1_048_576).toFixed(0)} MB`;
    if (n >= 1024) return `${(n / 1024).toFixed(0)} KB`;
    return `${n} B`;
  }

  function sparkline(samples: number[]): string {
    if (samples.length === 0) return "";
    const W = 100;
    const H = 22;
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

  $: cpuNow = [...$running.values()].reduce((a, s) => a + s.cpu, 0);
  $: ramNow = [...$running.values()].reduce((a, s) => a + s.ram_bytes, 0);
  $: cpuPoints = sparkline($totalCpuHistory);
  $: ramPoints = sparkline($totalRamHistory);
</script>

<div class="stats" title="Total CPU and RAM across every VSCode process on this machine (5-minute sparkline)">
  <div class="heading">
    <span class="tag">Stats</span>
    <span class="sub">across every VSCode process</span>
  </div>
  <div class="rows">
    <div class="stat">
      <span class="m-label">CPU</span>
      <svg class="sparkline" viewBox="0 0 100 22" preserveAspectRatio="none" aria-hidden="true">
        <polyline class="spark-cpu" points={cpuPoints} />
      </svg>
      <span class="m-value">{Math.round(cpuNow)}%</span>
    </div>
    <div class="stat">
      <span class="m-label">RAM</span>
      <svg class="sparkline" viewBox="0 0 100 22" preserveAspectRatio="none" aria-hidden="true">
        <polyline class="spark-ram" points={ramPoints} />
      </svg>
      <span class="m-value">{formatBytes(ramNow)}</span>
    </div>
  </div>
</div>

<style>
  .stats {
    display: flex;
    align-items: center;
    gap: 1.25rem;
    background: #252525;
    border: 1px solid #3c3c3c;
    border-radius: 4px;
    padding: 0.6rem 0.9rem;
    margin-bottom: 0.75rem;
  }
  .heading {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    flex-shrink: 0;
    min-width: 11rem;
  }
  .tag {
    display: inline-block;
    background: #0e639c;
    color: #fff;
    font-size: 0.7rem;
    font-weight: 600;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    padding: 0.15rem 0.45rem;
    border-radius: 3px;
    align-self: flex-start;
  }
  .sub {
    font-size: 0.7rem;
    color: #888;
    line-height: 1.3;
  }
  .rows {
    display: flex;
    flex: 1;
    gap: 1rem;
  }
  .stat {
    display: grid;
    grid-template-columns: 2.3rem 1fr auto;
    gap: 0.4rem;
    align-items: center;
    flex: 1;
    font-size: 0.8rem;
    color: #888;
    font-variant-numeric: tabular-nums;
  }
  .m-label {
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: #9a9a9a;
    font-weight: 500;
  }
  .m-value {
    color: #d4d4d4;
    text-align: right;
    min-width: 4.5rem;
  }
  .sparkline { display: block; width: 100%; height: 22px; }
  .sparkline .spark-cpu { stroke: #e5c07b; fill: none; stroke-width: 1.3; }
  .sparkline .spark-ram { stroke: #61afef; fill: none; stroke-width: 1.3; }
</style>
