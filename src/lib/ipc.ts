import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Config, WorkspaceEntry, WorkspaceStatus } from "./types";

export const getConfig = () => invoke<Config>("get_config");
export const setRootFolder = (path: string | null) =>
  invoke<Config>("set_root_folder", { path });
export const getWorkspaces = () => invoke<WorkspaceEntry[]>("get_workspaces");
export const getRunningWorkspaces = () =>
  invoke<WorkspaceStatus[]>("get_running_workspaces");
export const launch = (path: string) => invoke<void>("launch", { path });
export const focusWindow = (hwnd: number) => invoke<void>("focus_window", { hwnd });
export const closeWorkspaceWindow = (hwnd: number) =>
  invoke<void>("close_workspace_window", { hwnd });
export const setPinned = (path: string, pinned: boolean) =>
  invoke<Config>("set_pinned", { path, pinned });
export const setIcon = (workspace: string, icon: string | null) =>
  invoke<Config>("set_icon", { workspace, icon });
export const resolvedCodeBinary = () =>
  invoke<string | null>("resolved_code_binary");

export const onRunningUpdated = (cb: (statuses: WorkspaceStatus[]) => void): Promise<UnlistenFn> =>
  listen<WorkspaceStatus[]>("running-workspaces-updated", (e) => cb(e.payload));
