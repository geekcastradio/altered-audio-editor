# Altered Audio Editor 0.66.0

A working Light Mode with a theme switcher, a level meter range you can set, AAC and M4A export on Linux and macOS, panels that fold down to their title, and five more batches of the move from TypeScript to Rust.

## New

- **Theme switcher:** three icons at the right of the tool strip choose Light Mode (sun), Sync with System (monitor) or Dark Mode (moon), each with a hover description. Sync follows your system and changes when it does. View > Theme has the same choices.
- **Level meter range** (issue 269): Settings > Meter range sets how far down the meters go (24, 48, 60, 72, 96 or 120 dB, default 60). Right-click the Levels panel for the same choice. The Voice Chain meters follow it.
- **AAC and M4A export on Linux and macOS** through an encoder built into the app. Windows keeps using its own system encoder.
- **Panels fold down to their title:** every side panel group can be dragged down to its tab strip, including Properties and Takes. The contents scroll when the panel is short.

## Fixes

- **Light Mode now works.** Choosing it from the View menu only changed a setting and never applied it.
- **Window headers match.** Settings, Keyboard Shortcuts, Controllers, Plug-in Manager, the effect windows, Edit Notes and the color popover share one title bar in both looks. The effect windows had a fixed off-white bar.

## Moved to Rust, with tests that compare both sides

The interface still uses its TypeScript code; the Rust copies are tested against the same fixtures so they can take over later.

- Key dispatch: which shortcut or key the app acts on in each context.
- Voice Chain run notices (what to tell you when a chain cannot run as set).
- Settings draft and size rules.
- Panel placement rules for the docking layout.
- Effect envelopes on a clip.

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
