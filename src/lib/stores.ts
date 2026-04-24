import { derived, get, writable, type Readable } from "svelte/store";
import type { Config, WorkspaceEntry, WorkspaceStatus } from "./types";

const HISTORY_SIZE = 60;

export const config = writable<Config>({ root_folder: null, pinned: [], icons: {} });
export const workspaces = writable<WorkspaceEntry[]>([]);
export const running = writable<Map<string, WorkspaceStatus>>(new Map());
export const totalCpuHistory = writable<number[]>([]);
export const totalRamHistory = writable<number[]>([]);
export const scanError = writable<string | null>(null);

/** Call from the onRunningUpdated callback. */
export function applyStatuses(statuses: WorkspaceStatus[]) {
  const currentWorkspaces = get(workspaces);
  const byDisplayName = new Map(currentWorkspaces.map((w) => [w.display_name, w]));
  const m = new Map<string, WorkspaceStatus>();
  let cpuSum = 0;
  let ramSum = 0;
  for (const s of statuses) {
    cpuSum += s.cpu;
    ramSum += s.ram_bytes;
    let key = s.path;
    if (!key && s.displayNameHint) {
      const match = byDisplayName.get(s.displayNameHint);
      if (!match) continue;
      key = match.path;
    } else if (!key) {
      continue;
    }
    m.set(key, { ...s, path: key });
  }
  running.set(m);

  totalCpuHistory.update((arr) => {
    const next = [...arr, cpuSum];
    if (next.length > HISTORY_SIZE) next.splice(0, next.length - HISTORY_SIZE);
    return next;
  });
  totalRamHistory.update((arr) => {
    const next = [...arr, ramSum];
    if (next.length > HISTORY_SIZE) next.splice(0, next.length - HISTORY_SIZE);
    return next;
  });
}

export interface TileModel {
  path: string;
  displayName: string;
  icon: string | null;
  isRunning: boolean;
  isPinned: boolean;
  windowCount: number;
  hwnd: number | null;
}

const buildTile = (
  e: WorkspaceEntry,
  cfg: Config,
  runMap: Map<string, WorkspaceStatus>
): TileModel => {
  const status = runMap.get(e.path);
  const override = cfg.icons[e.path] ?? null;
  return {
    path: e.path,
    displayName: e.display_name,
    icon: override ?? e.auto_icon,
    isRunning: !!status,
    isPinned: cfg.pinned.includes(e.path),
    windowCount: status?.window_count ?? 0,
    hwnd: status?.hwnd ?? null,
  };
};

export const runningTiles: Readable<TileModel[]> = derived(
  [workspaces, running, config],
  ([$ws, $run, $cfg]) => {
    const known = new Map($ws.map((w) => [w.path, w]));
    const result: TileModel[] = [];
    for (const [path, status] of $run) {
      const w = known.get(path);
      if (w) {
        result.push(buildTile(w, $cfg, $run));
      } else {
        const name = path.split(/[\\/]/).pop()?.replace(/\.code-workspace$/, "") ?? path;
        result.push({
          path,
          displayName: name,
          icon: $cfg.icons[path] ?? null,
          isRunning: true,
          isPinned: $cfg.pinned.includes(path),
          windowCount: status.window_count,
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
