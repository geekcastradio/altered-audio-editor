# Altered Audio Editor 0.65.0

Bounce a track or a time selection to a new track, favorite Voice Chain presets, and fourteen more batches of the move from TypeScript to Rust.

## New

- **Bounce to New Track** (Multitrack menu and the lane right-click menu): bounce the selected track, or the selected clips inside a time selection, to a new track with one clip. It is one undoable step. The items are greyed out with a reason when there is nothing to bounce.
- **Voice Chain preset favorites:** a star beside the Presets row marks the picked preset as a favorite. Favorites sit at the top of the Presets menu in the order you starred them, and are kept between runs.

## Fixes

- Linux and the security check in our build pipeline were hardened (a disk shortage that broke test builds, and a patch update to a library the app only uses through Tauri). Nothing changes in the app itself.

## Moved to Rust, with tests that compare both sides

The interface still uses its TypeScript code; the Rust copies are tested against the same fixtures so they can take over later.

- Waveform editor edits and the Spectrum view maths.
- Video open rules and the Video Reference bar and drag.
- Plug-in list syncing and cleaning of saved settings.
- Settings store rules, stretch edits and Auto Mic Mix helpers.
- Hardware controller mapping and feedback frames.
- Audio FX looks and the miscellaneous Voice Chain panel rules.
- Clip edit helpers: silence a time range in selected clips, bring to front or back, the Phase Align pair and edit, and snap targets.
- View helpers: scroll to reveal a position, session tabs, and the Waveform editor channel switches.
- The bars and beats ruler: bar.beat.tick text and parsing, ruler ticks and grid lines through tempo changes.
- List selection (click, Ctrl and Shift), the parent of a folder path, exclusive solo, which tracks are silent, and remix time text.

## Known limitations

Altered Audio Editor is young. Windows is the platform we test. Please keep a backup copy of any session that matters.

- **Linux and macOS are experimental.** They are built from the same source but have had little testing. The macOS app is not notarized: the first time, right-click it and choose Open. Cmd is not yet used in place of Ctrl on macOS.
- **ASIO has not been tested on real hardware.** If an ASIO driver cannot open, the system driver plays instead. Reports from people with ASIO interfaces are welcome.
- **VST3 plug-ins, MIDI tracks, buses and 3D Space have had little hardware testing.** A plug-in that crashes is kept in its own process, so the editor should carry on, but tell us which plug-in it was. A session that uses a VST3 plug-in may need the plug-in picked again on another operating system.
- **AAC and M4A export works on Windows only.** The other export formats work everywhere. Monkey's Audio and Windows Media formats are not planned.
- **Hardware controllers** map faders, knobs, buttons and the transport, and motor faders, button lights and pan rings follow the screen (Mackie Control layout, not tried on a real surface). Scribble strips and jog wheels are not built.
- **Opening other programs' sessions** brings in tracks, clips and markers. Effects and some edits are left out, and the import says what was skipped.
- **Transcription, filler word removal, text-based editing, Music Remover and publishing from inside the app** are planned for after 1.0. Nothing in the app uses the network without you turning it on.
- **Some tests are by hand only.** Recording behavior, live meters and the look of the window are checked by a person, not by the automatic tests.
