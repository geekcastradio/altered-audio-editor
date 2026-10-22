# Release 0.1.7 plan (bug fix release)

Status: version bumped to 0.1.7. Devices crash fix written in the cloud copy (not yet run on Windows). Other items below are not started.

## Scope, in order

1. Settings > Devices crash. Split test first (`npm run tauri dev` vs release build), then: list ASIO drivers from the registry without loading them, run lookups one at a time, crash-guard marker file, "loading" state.
2. macOS microphone fix (config files already prepared: `Info.plist`, `Entitlements.plist`, `bundle.macOS` in `tauri.conf.json`). Add the "No sound is arriving" note for silent recordings. Needs a CI macOS build and a Mac retest.
3. Cache and Files default to `Documents\Altered Audio Editor\`, changeable, with a OneDrive warning.
4. Live recording waveform drawn from an incrementally built peak cache, in both editors.
5. Long jobs: Apply to Whole File tagged to its source file; export progress in the status bar instead of a blocking overlay; lock files so a second copy does not offer to recover a live session.
6. Small items: Help and About links, Report a Problem, remove Show Log Files, Invert and Reverse, empty window on launch.

Items 4 to 6 can slip to 0.1.8 if 1 to 3 take the credits. Shipping the crash fix alone is better than holding it for the rest.

## Checklist

- [ ] `npm run bump -- 0.1.7`
- [ ] `npm run check`, `npm test`, `cargo fmt --check`, `cargo clippy`, `cargo test` pass
- [ ] CI green on all three platforms, including the macOS build
- [ ] Tag `v0.1.7`, push the tag, let the Release workflow make the draft
- [ ] Install on a clean Windows machine; open Settings > Devices with and without an interface
- [ ] Mac tester confirms the microphone prompt and a live waveform
- [ ] Publish the draft; mark as pre-release

## Draft release notes

**Fixes**

- Opening Settings > Devices no longer closes the app.
- macOS: the app now asks for microphone permission, so recording no longer gives a flat line.
- (Add each of items 3 to 6 that ships.)

**Known issues**

- Builds are unsigned (Windows) or ad hoc signed (macOS); expect the usual first-run warnings.
- Audio, ASIO and macOS or Linux behavior is still lightly tested on real hardware. Reports welcome.
