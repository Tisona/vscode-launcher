import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Config, WorkspaceEntry } from "./types";

export const getConfig = () => invoke<Config>("get_config");
export const setRootFolder = (path: string | null) =>
  invoke<Config>("set_root_folder", { path });
export const getWorkspaces = () => invoke<WorkspaceEntry[]>("get_workspaces");
export const getRunning = () => invoke<string[]>("get_running");
export const launch = (path: string) => invoke<void>("launch", { path });
export const setPinned = (path: string, pinned: boolean) =>
  invoke<Config>("set_pinned", { path, pinned });
export const setIcon = (workspace: string, icon: string | null) =>
  invoke<Config>("set_icon", { workspace, icon });
export const resolvedCodeBinary = () =>
  invoke<string | null>("resolved_code_binary");

export const onRunningUpdated = (cb: (paths: string[]) => void): Promise<UnlistenFn> =>
  listen<string[]>("running-workspaces-updated", (e) => cb(e.payload));
