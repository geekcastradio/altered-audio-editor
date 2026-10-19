# Altered Audio Editor 0.1.5: how to merge and release

If you have not released 0.1.4 yet, skip it: 0.1.5 includes everything in it. If you already unzipped the 0.1.4 package, unzipping this one on top is fine.

This zip holds every file that changed since your last sync, laid out like the repo. Unzipping it over the repo adds or replaces those files and touches nothing else.

## 1. Merge into your PC repo

1. Close the app, VS Code and any `npm run tauri dev` window.
2. In PowerShell:

   ```powershell
   cd C:\Users\scp21\GCRN\altered-audio-editor
   git status
   ```

   It should say "nothing to commit". If not, commit or stash your own changes first.
3. Unzip `altered-audio-editor-0.1.5.zip` into `C:\Users\scp21\GCRN\altered-audio-editor`. Say yes to replacing files. It includes `.github/workflows/ci.yml` and `release.yml`, so you can skip the older `ci-fix.zip` and `ci-linux-mac.zip`.
4. Delete the retired screenshot if it is still tracked:

   ```powershell
   git rm docs/images/per-speaker.png
   ```

   If git says it does not match any files, it is already gone.

## 2. Check it builds

```powershell
npm install
npm run check
npm test
cd src-tauri
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cd ..
```

All of these passed in my workspace. Your normal build leaves ASIO out, so you do not need anything new. To try ASIO locally, install LLVM (https://releases.llvm.org, "Pre-Built Binaries", Windows 64-bit), then run `setx LIBCLANG_PATH "C:\Program Files\LLVM\bin"`, open a new PowerShell, and use `npm run tauri dev -- --features asio`. The ASIO code could not be compiled in my workspace (it only builds on Windows), so the "Windows ASIO build" job in CI is its first real compile. The first Rust build takes a few minutes longer than usual because it builds the patched Opus decoder in `src-tauri/vendor/opus-decoder`.

## 3. Try it by hand (10 minutes)

1. `npm run tauri dev`
2. **Input picker:** click a track's input. You should see Mono and Stereo submenus, then devices, then channels.
3. **Effect window:** add an effect in the Voice Chain. Its settings open in a movable window. Click the effect name or the gear to reopen. Esc closes it.
4. **Opus:** File > Open an `.opus` file. It should open in a second or two.
5. **Switching files:** open file A in the Waveform editor, make an edit, double-click file B in Files. No save prompt. Double-click A again: your edit and Undo are still there.
6. **Record more in the Waveform editor:** record 5 seconds on a track, double-click the clip, record more at the end in the Waveform editor, then Save. Back in Multitrack, the clip now runs to the new end.
7. **Wheel zoom:** scroll the mouse wheel over the multitrack lanes: it zooms. Over the track headers it scrolls the tracks.
8. **Loop:** turn on Loop Playback with no time selection and play: it loops back at the end of the last clip.
9. **Files panel:** the session is the first row. Double-click it from the Waveform editor to go back.
10. **Settings:** Edit > Settings opens. Audio Devices has "Waveform editor records from".
11. **Time Info:** play a file in the Waveform editor, and also a session in Multitrack: the Playhead time moves in both.
12. **Phase Rotate:** put it on a music clip, set Angle to 180: the music flips. Then click the clip's envelope tag > Effect Setting > Phase Rotate: Angle and draw a few points; play it and hear it move.
13. **Export .sesx:** File > Export > Multitrack Session (.sesx). Open the file in the other editor and check tracks, clips and fades line up. Tell me what it says or gets wrong.
14. **ASIO (needs the ASIO build):** Settings > Audio Devices > Driver: ASIO, OK, then play and record.
15. **Long timeline bug:** in Multitrack, double-click a FLAC clip: the Waveform editor should show the whole song filling the view, not a much longer timeline.
16. **Waveform save with effects:** add Pitch Shift in the Waveform editor, Save, reopen the file. The pitch change is in the file.

## 4. Commit, push and tag

```powershell
git add -A
git commit -m "0.1.5: ASIO, Phase Rotate, effect automation, .sesx export, user report fixes, Linux and macOS builds"
git push
```

Wait for CI to go green on GitHub (Actions tab). Then:

```powershell
git tag v0.1.5
git push origin v0.1.5
```

## 5. Publish the release

1. On GitHub, open Releases. A draft "Altered Audio Editor v0.1.5" appears after about 10 minutes with the Windows files.
2. The Linux and macOS files are added to the same draft a few minutes later. They are experimental. If either job fails, the Windows release is still fine; send me the log.
3. Paste in `release-notes-0.1.5.md` (attached separately), then Publish.
4. Optional: delete the old v0.1.3 draft if one is still sitting there.

## If something goes wrong

- **`cargo` cannot find `opus-decoder`:** make sure `src-tauri/vendor/opus-decoder` exists after unzipping (it has a `Cargo.toml` and a `src` folder).
- **CI red:** open the failed job, copy the last 40 lines, and send them to me.
