# VSCode Launcher

A small cross-platform desktop launcher for VSCode workspaces.

Scans a folder of `.code-workspace` files and shows them as clickable buttons.
Running workspaces appear as large tiles at the top with live CPU and RAM
sparklines, so you can quickly switch between the projects you have open.
Pin your favourites. Give each workspace a custom icon.

Built with [Tauri 2](https://tauri.app), Rust, and Svelte.

## Features

- **One-click workspace launch** — click a button to open the workspace in
  VSCode. If it's already open, VSCode focuses the existing window instead
  of spawning a second one.
- **Running-workspace tiles** — large tiles at the top of the window for every
  VSCode window currently holding a `.code-workspace` open, including ones
  outside your configured folder. Each tile shows live CPU and RAM
  sparklines over a 5-minute window, plus a badge if you have the same
  workspace open in multiple windows.
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

## How it works

### Finding VSCode

On launch, the app looks for the `code` binary in the standard install
locations per-OS, then falls back to `PATH`. Known locations:

- Windows: `%LOCALAPPDATA%\Programs\Microsoft VS Code\bin\code.cmd`,
  `C:\Program Files\Microsoft VS Code\bin\code.cmd`
- macOS: `/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code`
- Linux: `/usr/bin/code`, `/usr/local/bin/code`, `/snap/bin/code`

If none of those exist and `code` isn't on `PATH`, you'll get a clear error
message with a list of the paths that were tried.

### Detecting running workspaces

Every 5 seconds the app enumerates running processes (via `sysinfo`) and
keeps those whose command-line args contain a path ending in
`.code-workspace`. This works identically across Windows, macOS, and Linux
and doesn't depend on VSCode's process name (covering `code`, `code-oss`,
`vscodium`, `Code.exe`, etc.) or window titles.

RAM and CPU metrics are summed across each workspace's whole process tree
— the main Electron process plus renderer, extension host, language
servers, the integrated terminal, and anything spawned from the terminal.
If you have `npm run dev` or Claude running in a workspace terminal, those
bytes and cycles count toward that workspace — which is usually the
information worth having.

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

## Releases

Release builds happen on GitHub Actions when a `v*.*.*` tag is pushed —
see [`docs/releasing.md`](docs/releasing.md) for the checklist. No code
signing; three runners (Linux x64, macOS universal, Windows x64) build in
parallel, ~15 min on a warm cache.

## License

MIT.
