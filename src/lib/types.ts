export interface WorkspaceEntry {
  path: string;
  display_name: string;
  auto_icon: string | null;
}

export interface Config {
  root_folder: string | null;
  pinned: string[];
  icons: Record<string, string>;
}

export interface WorkspaceStatus {
  path: string;
  cpu: number; // raw per-tree sum, 100 = one full core
  ram_bytes: number;
  window_count: number;
  displayNameHint?: string | null;
}
