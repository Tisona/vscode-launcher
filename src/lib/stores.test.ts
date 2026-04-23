import { describe, expect, it } from "vitest";
import { get } from "svelte/store";
import {
  allTiles,
  applyStatuses,
  config,
  cpuHistory,
  pinnedTiles,
  ramHistory,
  running,
  runningTiles,
  workspaces,
} from "./stores";

function resetStores() {
  config.set({ root_folder: null, pinned: [], icons: {} });
  workspaces.set([]);
  running.set(new Map());
  cpuHistory.set(new Map());
  ramHistory.set(new Map());
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
    expect(tiles[0].cpu).toBe(12.5);
    expect(tiles[0].ramBytes).toBe(1_500_000_000);
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
    expect(tiles[0].cpu).toBe(5);
  });

  it("applyStatuses accumulates up to 60 samples of history", () => {
    resetStores();
    for (let i = 0; i < 65; i++) {
      applyStatuses([{ path: "/x.code-workspace", cpu: i, ram_bytes: 1000 * i, window_count: 1 }]);
    }
    const hist = get(cpuHistory).get("/x.code-workspace")!;
    expect(hist).toHaveLength(60);
    expect(hist[0]).toBe(5);
    expect(hist[59]).toBe(64);
  });

  it("drops history for workspaces no longer running", () => {
    resetStores();
    // Tick 1: workspace A running.
    applyStatuses([{ path: "/a.code-workspace", cpu: 10, ram_bytes: 1000, window_count: 1 }]);
    expect(get(cpuHistory).has("/a.code-workspace")).toBe(true);
    expect(get(ramHistory).has("/a.code-workspace")).toBe(true);

    // Tick 2: workspace A closed, workspace B running.
    applyStatuses([{ path: "/b.code-workspace", cpu: 20, ram_bytes: 2000, window_count: 1 }]);
    expect(get(cpuHistory).has("/a.code-workspace")).toBe(false);
    expect(get(ramHistory).has("/a.code-workspace")).toBe(false);
    expect(get(cpuHistory).get("/b.code-workspace")).toEqual([20]);

    // Tick 3: nothing running.
    applyStatuses([]);
    expect(get(cpuHistory).size).toBe(0);
    expect(get(ramHistory).size).toBe(0);
    expect(get(running).size).toBe(0);
  });
});
