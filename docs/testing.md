# Manual smoke test

Run on each target OS (Windows, macOS, Linux) before every tag.

## Setup

1. Fresh install — delete the app-config dir if it exists:
   - Windows: `%APPDATA%\vscode-launcher\`
   - macOS: `~/Library/Application Support/vscode-launcher/`
   - Linux: `~/.config/vscode-launcher/`
2. Create a test folder with at least **three** `.code-workspace` files,
   one with a sibling PNG (e.g. `foo.code-workspace` + `foo.png`).
3. Ensure VSCode is installed and the `code` CLI is either on `PATH` or at
   the default OS install location.

## Checks

1. **First launch** — empty state is shown with the "Pick your workspaces folder" button.
2. **Pick folder** — list populates under "All workspaces". The file with a sibling PNG shows the PNG as its icon; others show the default SVG.
3. **Launch** — click a workspace → VSCode opens with that workspace loaded.
4. **Auto-detect running** — open a fourth `.code-workspace` outside the configured folder via `code some/other/path.code-workspace` from a terminal. Within ~5s it appears in the "Running" section as a large tile.
5. **Auto-remove running** — close that VSCode window. Within ~5s the tile disappears from Running.
6. **Focus, not duplicate** — click a Running tile. The existing VSCode window is focused; no second instance spawns.
7. **Stats tile** — a single tile above the Running tiles shows total VSCode CPU% and RAM, with sparklines that update every ~5s as you work.
8. **Close button** — each Running tile has an × in the top-right corner. Click it → the corresponding VSCode window closes (within ~5s the tile disappears).
9. **Window count badge** — open the same workspace in a second VSCode window. The Running tile shows `×2`.
10. **Pin** — right-click a workspace button → Pin. The workspace appears in the Pinned section.
11. **Persistence** — close and reopen the launcher. Pin is preserved.
12. **Set icon** — right-click → Set icon… → pick a PNG/SVG/JPG. The tile displays the chosen icon immediately.
13. **Clear icon override** — right-click → Clear icon override. Icon falls back to auto-match (if any) or the default SVG.
14. **Reveal in file manager** — right-click → Reveal in file manager. OS file manager opens at the workspace's folder.
15. **Missing workspace** — delete a `.code-workspace` file externally. Click its button. A toast shows "Workspace file no longer exists: …". After Rescan, the button is gone.
16. **Unreadable folder banner** — rename the configured root folder on disk, relaunch. A yellow banner shows the scan error.
17. **Change folder** — gear icon → Settings → Change… → pick a different folder. List repopulates; banner clears.
18. **Rescan** — add a new `.code-workspace` file to the configured folder. Gear → Rescan. New entry appears.
19. **VSCode CLI not found** — rename `code` off PATH and out of the default location. Click a workspace → toast says "Launch failed: VSCode CLI not found".
20. **Detected `code` path** — gear → Settings. The dialog shows the absolute path of the resolved `code` binary (or "(not found)").
21. **Outsider pinning** — right-click a Running "orphan" tile (one from outside the scanned folder) → Pin. It appears in Pinned; survives restart (though will not appear in Pinned while not running, since Pinned is drawn from the scanned-folder list).
