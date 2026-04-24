import { derived, get, writable, type Readable } from "svelte/store";
import type { Config, WorkspaceEntry, WorkspaceStatus } from "./types";

const HISTORY_SIZE = 60;

export const config = writable<Config>({ root_folder: null, pinned: [], icons: {} });
export const workspaces = writable<WorkspaceEntry[]>([]);
export const running = writable<Map<string, WorkspaceStatus>>(new Map());
export const cpuHistory = writable<Map<string, number[]>>(new Map());
export const ramHistory = writable<Map<string, number[]>>(new Map());
export const scanError = writable<string | null>(null);

/** Call from the onRunningUpdated callback. */
export function applyStatuses(statuses: WorkspaceStatus[]) {
  const currentWorkspaces = get(workspaces);
  const byDisplayName = new Map(currentWorkspaces.map((w) => [w.display_name, w]));
  const m = new Map<string, WorkspaceStatus>();
  for (const s of statuses) {
    let key = s.path;
    if (!key && s.displayNameHint) {
      const match = byDisplayName.get(s.displayNameHint);
      if (!match) continue; // name-only status we can't resolve → drop
      key = match.path;
    } else if (!key) {
      continue;
    }
    m.set(key, { ...s, path: key });
  }
  running.set(m);

  cpuHistory.update((hist) => {
    const next = new Map<string, number[]>();
    for (const [path, s] of m) {
      const prev = hist.get(path) ?? [];
      const arr = [...prev, s.cpu];
      if (arr.length > HISTORY_SIZE) arr.splice(0, arr.length - HISTORY_SIZE);
      next.set(path, arr);
    }
    return next;
  });

  ramHistory.update((hist) => {
    const next = new Map<string, number[]>();
    for (const [path, s] of m) {
      const prev = hist.get(path) ?? [];
      const arr = [...prev, s.ram_bytes];
      if (arr.length > HISTORY_SIZE) arr.splice(0, arr.length - HISTORY_SIZE);
      next.set(path, arr);
    }
    return next;
  });
}

export interface TileModel {
  path: string;
  displayName: string;
  icon: string | null;
  isRunning: boolean;
  isPinned: boolean;
  // Only populated when isRunning === true:
  cpu: number;
  ramBytes: number;
  windowCount: number;
  cpuHistory: number[];
  ramHistory: number[];
  hwnd: number | null;
}

const buildTile = (
  e: WorkspaceEntry,
  cfg: Config,
  runMap: Map<string, WorkspaceStatus>,
  cpuHist: Map<string, number[]>,
  ramHist: Map<string, number[]>
): TileModel => {
  const status = runMap.get(e.path);
  const override = cfg.icons[e.path] ?? null;
  return {
    path: e.path,
    displayName: e.display_name,
    icon: override ?? e.auto_icon,
    isRunning: !!status,
    isPinned: cfg.pinned.includes(e.path),
    cpu: status?.cpu ?? 0,
    ramBytes: status?.ram_bytes ?? 0,
    windowCount: status?.window_count ?? 0,
    cpuHistory: cpuHist.get(e.path) ?? [],
    ramHistory: ramHist.get(e.path) ?? [],
    hwnd: status?.hwnd ?? null,
  };
};

export const runningTiles: Readable<TileModel[]> = derived(
  [workspaces, running, config, cpuHistory, ramHistory],
  ([$ws, $run, $cfg, $cpuH, $ramH]) => {
    const known = new Map($ws.map((w) => [w.path, w]));
    const result: TileModel[] = [];
    for (const [path, status] of $run) {
      const w = known.get(path);
      if (w) {
        result.push(buildTile(w, $cfg, $run, $cpuH, $ramH));
      } else {
        const name = path.split(/[\\/]/).pop()?.replace(/\.code-workspace$/, "") ?? path;
        result.push({
          path,
          displayName: name,
          icon: $cfg.icons[path] ?? null,
          isRunning: true,
          isPinned: $cfg.pinned.includes(path),
          cpu: status.cpu,
          ramBytes: status.ram_bytes,
          windowCount: status.window_count,
          cpuHistory: $cpuH.get(path) ?? [],
          ramHistory: $ramH.get(path) ?? [],
          hwnd: status.hwnd ?? null,
        });
      }
    }
    return result.sort((a, b) =>
      a.displayName.toLowerCase().localeCompare(b.displayName.toLowerCase())
    );
  }
);

export const pinnedTiles: Readable<TileModel[]> = derived(
  [workspaces, running, config, cpuHistory, ramHistory],
  ([$ws, $run, $cfg, $cpuH, $ramH]) =>
    $ws
      .filter((w) => $cfg.pinned.includes(w.path))
      .map((w) => buildTile(w, $cfg, $run, $cpuH, $ramH))
);

export const allTiles: Readable<TileModel[]> = derived(
  [workspaces, running, config, cpuHistory, ramHistory],
  ([$ws, $run, $cfg, $cpuH, $ramH]) => $ws.map((w) => buildTile(w, $cfg, $run, $cpuH, $ramH))
);
