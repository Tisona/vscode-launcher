# Releasing

Release artifacts (AppImage for Linux, .dmg for macOS, NSIS installer for Windows)
are built automatically on tag push by `.github/workflows/release.yml`.

## Process

1. Bump `version` in **both** `src-tauri/tauri.conf.json` and `src-tauri/Cargo.toml`
   so they match the tag you are about to push (e.g. `0.1.0`).
2. Commit the version bump:
   ```
   git commit -am "chore: bump version to 0.1.0"
   ```
3. Tag and push:
   ```
   git tag v0.1.0
   git push --follow-tags
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
- If you need to re-run a failed release job, re-push the tag:
  ```
  git tag -d v0.1.0
  git push origin :refs/tags/v0.1.0
  git tag v0.1.0
  git push --follow-tags
  ```
