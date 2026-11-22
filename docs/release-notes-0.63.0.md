# Altered Audio Editor 0.63.0

A clearer update notice that shows what is new, a button to show your file when an export finishes, a plain list of known limitations, and two more batches of the move from TypeScript to Rust.

## New

- The update notice is now a card: the new and current version, the first few changes from the release notes, and Later, All changes and Install and Restart buttons. It still never blocks your work and never appears during playback or recording.
- When a mixdown, stem export or playlist export finishes, the notice has a button that shows the file in Explorer, Finder or your file manager. Settings > Effects turns it off.
- Known limitations: a short, honest list of what is untested or not built yet, kept in `docs/known-limitations.md` and repeated at the end of these notes.

## Moved to Rust, with tests that compare both sides

The interface still uses its TypeScript code; the Rust copies are tested against the same fixtures so they can take over later.

- The User Guide reader: sections, lists, pictures, bold and code runs, and search.
- Session document rules: which files are copied into a session folder, Session Properties choices and edits, replacing clips with bleed-cleaned copies, and the built-in host profile list.
- Mix routing: which buses a track may send to without a loop, sidechain key choices, output names and send edits.
- Duck Music Under Voice: the settings for each depth, which tracks are voice and which are music, and adding or removing the duck.
- Sync Tracks: moving a synced track (before the start, blocked, locked) and the wording of the sync result.

## Known limitations

Altered Audio Editor is young. Windows is the platform we test. Please keep a backup copy of any session that matters.

- **Linux and macOS are experimental.** They are built from the same source but have had little testing. The macOS app is not notarized: the first time, right-click it and choose Open. Cmd is not yet used in place of Ctrl on macOS.
- **ASIO has not been tested on real hardware.** If an ASIO driver cannot open, the system driver plays instead. Reports from people with ASIO interfaces are welcome.
- **VST3 plug-ins, MIDI tracks, buses and 3D Space have had little hardware testing.** A plug-in that crashes is kept in its own process, so the editor should carry on, but tell us which plug-in it was. A session that uses a VST3 plug-in may need the plug-in picked again on another operating system.
- **AAC and M4A export works on Windows only.** The other export formats work everywhere. Monkey's Audio and Windows Media formats are not planned.
- **Hardware controllers** map faders and knobs. Motor fader feedback, scribble strips and jog wheels are not built.
- **Opening other programs' sessions** brings in tracks, clips and markers. Effects and some edits are left out, and the import says what was skipped.
- **Transcription, filler word removal, text-based editing, Music Remover and publishing from inside the app** are planned for after 1.0. Nothing in the app uses the network without you turning it on.
- **Some tests are by hand only.** Recording behavior, live meters and the look of the window are checked by a person, not by the automatic tests.
