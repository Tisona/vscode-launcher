import { describe, expect, it } from "vitest";
import { get } from "svelte/store";
import {
  allTiles,
  applyStatuses,
  config,
  pinnedTiles,
  running,
  runningTiles,
  totalCpuHistory,
  totalRamHistory,
  workspaces,
} from "./stores";

function resetStores() {
  config.set({ root_folder: null, pinned: [], icons: {} });
  workspaces.set([]);
  running.set(new Map());
  totalCpuHistory.set([]);
  totalRamHistory.set([]);
}

describe("stores", () => {
  it("runningTiles includes outsiders with derived display name", () => {
    resetStores();
    config.set({ root_folder: "/root", pinned: [], icons: {} });
    applyStatuses([
      { path: "/elsewhere/orphan.code-workspace", cpu: 12.5, ram_bytes: 1_500_000_000, window_count: 1 },
    ]);
    const tiles = get(runningTiles);
    expect(tiles).toHaveLength(1);
    expect(tiles[0].displayName).toBe("orphan");
    expect(tiles[0].isRunning).toBe(true);
    expect(tiles[0].windowCount).toBe(1);
  });

  it("pinnedTiles reflects config.pinned", () => {
    resetStores();
    config.set({ root_folder: "/root", pinned: ["/root/a.code-workspace"], icons: {} });
    workspaces.set([
      { path: "/root/a.code-workspace", display_name: "a", auto_icon: null },
      { path: "/root/b.code-workspace", display_name: "b", auto_icon: null },
    ]);
    const pins = get(pinnedTiles);
    expect(pins.map((t) => t.displayName)).toEqual(["a"]);
  });

  it("icon override takes precedence over auto_icon", () => {
    resetStores();
    config.set({
      root_folder: "/root",
      pinned: [],
      icons: { "/root/a.code-workspace": "/custom/icon.png" },
    });
    workspaces.set([
      { path: "/root/a.code-workspace", display_name: "a", auto_icon: "/root/a.png" },
    ]);
    const tiles = get(allTiles);
    expect(tiles[0].icon).toBe("/custom/icon.png");
  });

  it("isRunning reflects running map", () => {
    resetStores();
    workspaces.set([
      { path: "/root/a.code-workspace", display_name: "a", auto_icon: null },
    ]);
    applyStatuses([
      { path: "/root/a.code-workspace", cpu: 5, ram_bytes: 900_000_000, window_count: 1 },
    ]);
    const tiles = get(allTiles);
    expect(tiles[0].isRunning).toBe(true);
  });

  it("aggregate history keeps up to 60 samples, summing CPU across workspaces per tick", () => {
    resetStores();
    for (let i = 0; i < 65; i++) {
      applyStatuses([
        { path: "/x.code-workspace", cpu: i, ram_bytes: 1000, window_count: 1 },
        { path: "/y.code-workspace", cpu: 1, ram_bytes: 500, window_count: 1 },
      ]);
    }
    const cpuHist = get(totalCpuHistory);
    const ramHist = get(totalRamHistory);
    expect(cpuHist).toHaveLength(60);
    expect(cpuHist[0]).toBe(6); // tick 5: 5 + 1
    expect(cpuHist[59]).toBe(65); // tick 64: 64 + 1
    expect(ramHist).toHaveLength(60);
    expect(ramHist[59]).toBe(1500);
  });

  it("running tile carries the hwnd from the status", () => {
    resetStores();
    workspaces.set([
      { path: "/a.code-workspace", display_name: "a", auto_icon: null },
    ]);
    applyStatuses([
      { path: "/a.code-workspace", cpu: 1, ram_bytes: 100, window_count: 1, hwnd: 0x1234 },
    ]);
    const tiles = get(allTiles);
    expect(tiles[0].hwnd).toBe(0x1234);
  });

  it("running map reflects latest tick; empty tick clears it", () => {
    resetStores();
    applyStatuses([{ path: "/a.code-workspace", cpu: 10, ram_bytes: 1000, window_count: 1 }]);
    expect(get(running).size).toBe(1);

    applyStatuses([]);
    expect(get(running).size).toBe(0);
  });
});
