# Hand test 0.42.0: Fill Single Click texture, plughost safety pass, pure-aac build

## Fill Single Click texture
1. Open Settings > Effects. Find "Fill Single Click texture": Natural (default) and Smooth.
2. With Natural selected, open a recording with a click or pop, select a few milliseconds around it, and run Fill Single Click. The gap should be filled and blend into the room tone. Undo should read "Fill Single Click".
3. Switch to Smooth and repeat on a clean, quiet passage. Undo should read "Fill Single Click (Smooth)". Zoom in: Smooth should look like a clean interpolation with no added grain.
4. On a longer gap (4 to 5 ms or more) Natural adds matched noise, Smooth never does. Compare the two by ear.
5. Close and reopen the app: the choice is remembered. Edit > Keyboard Shortcuts is unchanged.
6. Help > Guide, Fill Single Click: the new bullet about the setting is there.

## Plugin hosting (VST3), smoke test for the safety pass
No behaviour change is intended. This pass added a comment to every unsafe block, null checks in the host callbacks, and lint rules that fail the build if a new unsafe block has no comment.
1. Scan plugins, then load one VST3 effect on a track. It should load, show its parameters, and process audio.
2. Open its editor window, move a control, close the window, reopen it. No crash.
3. Remove the plugin, and quit the app with a plugin still loaded. No crash or hang.
4. Repeat with a second VST3 from a different vendor, ideally one with a resizable editor.
Tell me about any crash, even if it only happens on close.

## pure-aac build (optional, not in the release installer)
1. From the repo: `cd src-tauri; cargo test --lib --features pure-aac`. All tests pass (611 on Linux).
2. `npm run tauri dev -- --features pure-aac` (or `cargo build --features pure-aac`). Export a short mix as AAC (M4A). It should play in the app, Windows Media Player or Groove, and your phone.
3. Export mono and stereo. Export at 44.1 kHz and 48 kHz.
4. Normal build without the feature: AAC export still uses the Windows encoder, as before.
