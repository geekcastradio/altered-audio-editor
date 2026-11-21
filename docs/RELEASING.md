# Releasing

Every update that goes out gets its own version, so a bug report can name the build. Releases are normal releases, not pre-releases, so GitHub's "latest" points at them and the in-app update check finds them.

1. Bump the version. One command changes all five files that carry it (`package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/tauri.conf.json`):

   ```
   npm run bump              # 0.49.0 -> 0.49.1
   npm run bump -- minor     # 0.49.1 -> 0.50.0
   ```

   Stay on 0.x until the editor is stable for daily podcast work; 1.0.0 marks that.
2. Open a pull request with the version bump (and the release notes and hand-test page), make sure CI is green, and merge it into `main`.
3. Start the build without making a tag: Actions, Release, Run workflow, branch `main`, and type the version (for example `0.50.0`). A version tag pushed by hand also works.
4. The Release workflow checks that the version matches the files, then builds the Windows installer, portable zip and MSI, the Linux AppImage and `.deb`, and the macOS builds for Apple silicon and Intel (Linux and macOS are experimental; if one fails, the draft still gets the Windows files). It then makes a **draft** release with `latest.json` and `SHA256SUMS.txt`.
5. Install the installer on a clean Windows machine and work through the hand-test page (`docs/hand-test-<version>.md`). A published release can't be changed, so test first.
6. Edit the notes, then publish the draft. GitHub creates the tag on `main` when you publish. That tag push does not build again: the workflow sees a published release already exists and stops.

Local build: `npm run tauri build`. The installer lands in `src-tauri/target/release/bundle/nsis/`.

Files are unsigned for now, so Windows SmartScreen shows a warning and the macOS app needs a right-click, Open the first time. `SHA256SUMS.txt` is attached to each release so people can check their download. Updates are signed with the project's update key, and the app checks that signature before it installs.
