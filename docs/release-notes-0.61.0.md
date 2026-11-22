# Altered Audio Editor 0.61.0

Another batch of the move from TypeScript to Rust, plus a dependency fix. Nothing here changes how the app looks or works. The interface still uses its TypeScript code; the Rust copies are tested against the same fixtures so they can take over later.

## Moved to Rust, with tests that compare both sides

- Automation take recorder: record modes, arming lanes, gliding back to the stored value, and merging a take into a lane.
- Effect controls: favorite effects, control columns, and value limits and parsing for each setting.
- Curve EQ presets: the 19 built-in curves.
- Effect preset bank: saving, removing, applying and cleaning up presets.
- Tag sets, soundboard pads and library bins: reading saved lists, labels, starter bins and colors.
- Batch Convert: level settings, saved settings, name tokens, where each file converts to, the checks for two files writing one name, and saved chains with the built-in starters.

## New

- Help > Ask a Question or Share an Idea... opens the project's GitHub Discussions page in your browser. Questions and ideas go there, bugs stay in Issues.

## Fixed

- The macOS build now retries its disk image step, which failed once in the last release run. Only a release run can show whether this helps.

- Updated the URL parsing library (url 2.5.9) after a published security advisory. The app makes no network calls beyond the update check, so this was housekeeping.

## Notes

- No new settings or shortcuts. One new Help menu item.
- Free, GPLv3, no telemetry.
