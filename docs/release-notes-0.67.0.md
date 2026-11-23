# Altered Audio Editor 0.67.0

Fixes from the 0.66.0 hand test: the tool buttons are back on the left, Light Mode is easier on the eyes, and switching themes repaints the timeline at once. One more batch of the move from TypeScript to Rust.

## Fixes

- **Tool buttons are back on the left** of the tool strip, after the Waveform and Multitrack switch, where they were in 0.65.0. Only the three theme icons sit at the right end.
- **Light Mode is easier to read.** The lanes and track headers are darker, so clips stand out and the timeline is no longer glaring. In Light Mode a clip has a stronger tint, its waveform, envelope lines and name are dark, and a selected clip shows a blue wash with a dark outline. The blue value and accent colors are darker, so blue text reads on gray buttons.
- **Switching themes repaints the timeline in the same moment.** The lanes, overview strip, ruler, Waveform editor and Spectrum view no longer lag behind the rest of the window.
- **A color check keeps it that way.** An automatic test measures the contrast of text, value and accent colors against every surface in both looks, so a later color change cannot quietly make something hard to read.

## Moved to Rust, with tests that compare both sides

The interface still uses its TypeScript code; the Rust copies are tested against the same fixtures so they can take over later.

- Shortcut import: reading another editor's shortcut export and applying it to your own shortcuts.

## Known limitations

Altered Audio Editor is young. Windows is the platform we test. Please keep a backup copy of any session that matters.

- **Linux and macOS are experimental.** They are built from the same source but have had little testing. The macOS app is not notarized: the first time, right-click it and choose Open. Cmd is not yet used in place of Ctrl on macOS.
- **ASIO has not been tested on real hardware.** If an ASIO driver cannot open, the system driver plays instead. Reports from people with ASIO interfaces are welcome.
- **VST3 plug-ins, MIDI tracks, buses and 3D Space have had little hardware testing.** A plug-in that crashes is kept in its own process, so the editor should carry on, but tell us which plug-in it was. A session that uses a VST3 plug-in may need the plug-in picked again on another operating system.
- **AAC and M4A export off Windows uses a built-in encoder** that has been tested against other decoders but not by ear on Linux or macOS. Windows uses its own system encoder. Monkey's Audio and Windows Media formats are not planned.
- **Hardware controllers** map faders, knobs, buttons and the transport, and motor faders, button lights and pan rings follow the screen (Mackie Control layout, not tried on a real surface). Scribble strips and jog wheels are not built.
- **Opening other programs' sessions** brings in tracks, clips and markers. Effects and some edits are left out, and the import says what was skipped.
- **Transcription, filler word removal, text-based editing, Music Remover and publishing from inside the app** are planned for after 1.0. Nothing in the app uses the network without you turning it on.
- **Some tests are by hand only.** Recording behavior, live meters and the look of the window are checked by a person, not by the automatic tests.
