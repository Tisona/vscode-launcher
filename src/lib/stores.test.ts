import { describe, expect, it } from "vitest";
import { get } from "svelte/store";
import { config, running, runningTiles, workspaces, allTiles, pinnedTiles } from "./stores";

describe("stores", () => {
  it("runningTiles includes outsiders with derived display name", () => {
    config.set({ root_folder: "/root", pinned: [], icons: {} });
    workspaces.set([]);
    running.set(new Set(["/elsewhere/orphan.code-workspace"]));
    const tiles = get(runningTiles);
    expect(tiles).toHaveLength(1);
    expect(tiles[0].displayName).toBe("orphan");
    expect(tiles[0].isRunning).toBe(true);
  });

  it("pinnedTiles reflects config.pinned", () => {
    config.set({ root_folder: "/root", pinned: ["/root/a.code-workspace"], icons: {} });
    workspaces.set([
      { path: "/root/a.code-workspace", display_name: "a", auto_icon: null },
      { path: "/root/b.code-workspace", display_name: "b", auto_icon: null },
    ]);
    running.set(new Set());
    const pins = get(pinnedTiles);
    expect(pins.map((t) => t.displayName)).toEqual(["a"]);
  });

  it("icon override takes precedence over auto_icon", () => {
    config.set({
      root_folder: "/root",
      pinned: [],
      icons: { "/root/a.code-workspace": "/custom/icon.png" },
    });
    workspaces.set([
      { path: "/root/a.code-workspace", display_name: "a", auto_icon: "/root/a.png" },
    ]);
    running.set(new Set());
    const tiles = get(allTiles);
    expect(tiles[0].icon).toBe("/custom/icon.png");
  });

  it("isRunning reflects running set", () => {
    config.set({ root_folder: "/root", pinned: [], icons: {} });
    workspaces.set([
      { path: "/root/a.code-workspace", display_name: "a", auto_icon: null },
    ]);
    running.set(new Set(["/root/a.code-workspace"]));
    const tiles = get(allTiles);
    expect(tiles[0].isRunning).toBe(true);
  });
});
