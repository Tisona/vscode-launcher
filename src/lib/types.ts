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
