# VSCode Launcher

**For those tired of looking for *that* specific VSCode running.**

You have six VSCode workspaces open. One is the service you're actively debugging,
one is the infra repo with the terraform you opened yesterday, one is a dev
server you forgot about, one is the repo a colleague asked you to check, one
is your notes, and one is the fork you were going to submit a PR from. They
all share the same blue taskbar icon. The Windows Alt-Tab preview is small
and they all just look like "some code". You click through four of them to
find the one you meant, break your flow, and swear at your computer.

This launcher is a single window that shows you exactly which workspaces are
currently open — as big tiles with the workspace name and an icon you chose
yourself. One click brings that window to the foreground, and a small × on
each tile closes just that window. No more alt-tab roulette. A single stats
tile above the running tiles shows total VSCode CPU and RAM, with a live
5-minute sparkline so you can see at a glance if something is running hot.

It also doubles as a workspace picker: point it at your folder of
`.code-workspace` files and you get a tidy button-grid of every project you
work on, with the ones you care about pinned to the top. One click opens
them in VSCode.

Built with [Tauri 2](https://tauri.app), Rust, and Svelte.

## Features

- **One-click workspace launch** — click a button to open the workspace in
  VSCode. If it's already open, VSCode focuses the existing window instead
  of spawning a second one.
- **Running-workspace tiles** — tiles at the top of the window for every
  VSCode window currently holding a `.code-workspace` open, including ones
  outside your configured folder. A badge appears if you have the same
  workspace open in multiple windows, and each tile has an × button to
  close that specific workspace window.
- **Aggregate stats tile** — single tile above the running list showing
  total CPU and RAM across all VSCode processes, with a 5-minute sparkline.
  (Per-tile metrics aren't possible because VSCode hosts all workspace
  windows in a single main process with a shared UI thread.)
- **Pinned favourites** — right-click any workspace to pin it. Pinned
  workspaces get their own section between Running and All.
- **Custom per-workspace icons** — auto-picks a sibling PNG/SVG/JPG next to
  the `.code-workspace` file, or you can right-click → Set icon… to choose
  any image from disk.
- **Cross-platform** — Windows, macOS (universal: Intel + Apple Silicon),
  Linux.

## Install

Download the latest release from the
[Releases page](../../releases/latest). Each release ships both standard
installers and portable zips — pick whichever fits your workflow.

| OS | Installer | Portable |
|---|---|---|
| Linux | `.deb` | `.AppImage` (already portable) |
| macOS (universal) | `.dmg` | `.zip` containing `vscode-launcher.app` |
| Windows | `_x64-setup.exe` (NSIS) | `.zip` containing `vscode-launcher.exe` |

Config lives in the per-OS app-data directory, so installer and portable
versions share the same settings:

- Windows: `%APPDATA%\vscode-launcher\config.json`
- macOS: `~/Library/Application Support/vscode-launcher/config.json`
- Linux: `~/.config/vscode-launcher/config.json`

**Gatekeeper / SmartScreen:** builds are unsigned. On first launch, macOS
asks you to right-click → Open once to bypass Gatekeeper. On Windows,
SmartScreen shows a "Don't run" dialog — click "More info" → "Run anyway".
This is expected for an unsigned binary; code signing requires paid
certificates that aren't worth it for a small personal tool.

**Windows portable also requires WebView2 runtime.** It ships with Windows 11
and recent Windows 10 builds, so most machines already have it. If the
portable exe silently does nothing on launch, install WebView2 from
[Microsoft](https://developer.microsoft.com/microsoft-edge/webview2/) or
use the installer version (which bundles the WebView2 bootstrapper).

## Build from source

### Prerequisites

- Rust stable toolchain (install via [rustup](https://rustup.rs/))
- Node.js 20+ and npm
- Linux only: the GTK / WebKit dev headers Tauri needs:

  ```bash
  sudo apt install -y \
    libwebkit2gtk-4.1-dev libssl-dev libayatana-appindicator3-dev \
    librsvg2-dev libgtk-3-dev libsoup-3.0-dev
  ```

### Dev loop

```bash
npm install
npm run tauri dev
```

### Production build

```bash
npm run tauri build
```

Artifacts land in `src-tauri/target/release/bundle/`.

### Tests and lint

```bash
cargo test    --manifest-path src-tauri/Cargo.toml
cargo clippy  --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo fmt     --manifest-path src-tauri/Cargo.toml --check
npm test
npm run check
npm run build
```

CI runs the same checks on every push across Windows, macOS, and Linux
(`.github/workflows/ci.yml`).

## License

MIT.
