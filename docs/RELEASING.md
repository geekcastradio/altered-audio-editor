# Releasing

1. Set the same version in `package.json`, `src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json`.
2. Make sure CI is green on `main`.
3. Tag and push:

   ```
   git tag v0.1.0
   git push origin v0.1.0
   ```

4. The Release workflow builds the installer and a portable zip, then makes a **draft** release.
5. Install the installer on a clean Windows machine and run it. Release immutability means a published release can't be changed, so test first.
6. Edit the notes, then publish the draft. Versions starting `v0.` are marked pre-release.

Local build: `npm run tauri build`. The installer lands in `src-tauri/target/release/bundle/nsis/`.

Files are unsigned for now, so Windows SmartScreen shows a warning. `SHA256SUMS.txt` is attached to each release so people can check their download.
