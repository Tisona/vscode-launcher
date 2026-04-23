# Releasing

Release artifacts are built automatically on tag push by
`.github/workflows/release.yml`. Each release contains **both** OS-standard
installers and **portable zips** of the raw binaries — users pick whichever
fits their workflow.

## Process

1. Bump `version` in **both** `src-tauri/tauri.conf.json` and `src-tauri/Cargo.toml`
   so they match the tag you are about to push (e.g. `0.1.0`).
2. Commit the version bump:
   ```
   git commit -am "chore: bump version to 0.1.0"
   ```
3. Push the branch, then tag and push the tag by name:
   ```
   git push
   git tag -a v0.1.0 -m "v0.1.0"
   git push origin v0.1.0
   ```
4. GitHub Actions runs the `Release` workflow (watch it in the Actions tab).
   Once complete, the release is published with all artifacts attached.

## Notes

- Builds run on free public-repo runners: first release takes ~15–25 min (cold
  Rust compile on each OS). Subsequent releases are ~5–10 min with the Swatinem
  cache warmed.
- Artifacts are **unsigned**. macOS users will see a Gatekeeper warning on first
  launch (right-click → Open to bypass). Windows users will see SmartScreen
  ("Don't run" → "More info" → "Run anyway"). Acceptable trade-off for v1 — code
  signing requires a paid Apple Developer ID ($99/yr) and a Windows certificate.

## Artifacts per OS

| OS | Installer | Portable |
|---|---|---|
| Linux | `vscode-launcher_<version>_amd64.deb` | `vscode-launcher_<version>_amd64.AppImage` (AppImage is portable by design) |
| macOS arm64 | `vscode-launcher_<version>_aarch64.dmg` | `vscode-launcher-macos-arm64.zip` (contains `vscode-launcher.app`) |
| macOS x64 | `vscode-launcher_<version>_x64.dmg` | `vscode-launcher-macos-x64.zip` |
| Windows | `vscode-launcher_<version>_x64-setup.exe` (NSIS) | `vscode-launcher-windows-x64.zip` (contains bare `vscode-launcher.exe`) |

**Config location is the same for both** — the app always reads / writes
`%APPDATA%\vscode-launcher\config.json` on Windows,
`~/Library/Application Support/vscode-launcher/` on macOS,
`~/.config/vscode-launcher/` on Linux. The "portable" in portable refers to
the binary's placement, not the config.

The Windows portable `.exe` requires the **WebView2 runtime**, which ships as
part of Windows 11 and modern Windows 10 (so it's already present on
virtually any machine you'd run it on). The NSIS installer bundles the
WebView2 bootstrapper as a fallback.

- If you need to re-run a failed release job, delete the tag locally and
  remotely, then recreate it (annotated) at the current commit and push it by
  name:
  ```
  git tag -d v0.1.0
  git push origin :refs/tags/v0.1.0
  git tag -a v0.1.0 -m "v0.1.0"
  git push origin v0.1.0
  ```
