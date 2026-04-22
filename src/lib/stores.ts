import { derived, writable, type Readable } from "svelte/store";
import type { Config, WorkspaceEntry } from "./types";

export const config = writable<Config>({ root_folder: null, pinned: [], icons: {} });
export const workspaces = writable<WorkspaceEntry[]>([]);
export const running = writable<Set<string>>(new Set());

export interface TileModel {
  path: string;
  displayName: string;
  icon: string | null;
  isRunning: boolean;
  isPinned: boolean;
}

const buildTile = (e: WorkspaceEntry, cfg: Config, runSet: Set<string>): TileModel => {
  const override = cfg.icons[e.path] ?? null;
  return {
    path: e.path,
    displayName: e.display_name,
    icon: override ?? e.auto_icon,
    isRunning: runSet.has(e.path),
    isPinned: cfg.pinned.includes(e.path),
  };
};

export const runningTiles: Readable<TileModel[]> = derived(
  [workspaces, running, config],
  ([$ws, $run, $cfg]) => {
    const known = new Map($ws.map((w) => [w.path, w]));
    const result: TileModel[] = [];
    for (const path of $run) {
      const w = known.get(path);
      if (w) {
        result.push(buildTile(w, $cfg, $run));
      } else {
        // Outsider — running but not in scanned folder.
        const name = path.split(/[\\/]/).pop()?.replace(/\.code-workspace$/, "") ?? path;
        result.push({
          path,
          displayName: name,
          icon: $cfg.icons[path] ?? null,
          isRunning: true,
          isPinned: $cfg.pinned.includes(path),
        });
      }
    }
    return result.sort((a, b) =>
      a.displayName.toLowerCase().localeCompare(b.displayName.toLowerCase())
    );
  }
);

export const pinnedTiles: Readable<TileModel[]> = derived(
  [workspaces, running, config],
  ([$ws, $run, $cfg]) =>
    $ws
      .filter((w) => $cfg.pinned.includes(w.path))
      .map((w) => buildTile(w, $cfg, $run))
);

export const allTiles: Readable<TileModel[]> = derived(
  [workspaces, running, config],
  ([$ws, $run, $cfg]) => $ws.map((w) => buildTile(w, $cfg, $run))
);
