# Altered Audio Editor 0.70.0

A Favorites menu and bar, one Import Session item that reads five formats, codec options on Save Audio As, an optional ask-before-Save, a calmer light theme, and fixes from tester reports. This release starts with the full document and licence review.

## New

- **Favorites menu and Quick Actions bar.** Right-click any menu command and choose Add to Favorites. Pinned commands appear in the new Favorites menu, and in a bar in the tool strip when View > Quick Actions Bar is on. The bar is off by default and is not drawn at all when off. Commands that only work in the Waveform editor or the Multitrack editor grey out in the other one. The list starts empty and there are no new keyboard shortcuts.
- **One Import Session item.** File > Import is now a submenu with Files, Folder and Session. Session opens one dialog with a file type list: all session and project files, `.sesx`, `.rpp`, `.aup3` and `.aup4`, or `.otio`. File > Open reads the same formats.
- **Codec options on Save Audio As.** In the Waveform editor, Save Audio As now opens the same format dialog as Save Selection As: file name and folder, then format, bit depth or sample type, bit rate and the other encoder settings. It saves the whole file.
- **Ask before Save (optional).** Edit > Settings > Autosave has a new option, off by default. With it on, Save in the Waveform editor asks before it writes over the original audio file.
- **Multitrack right-click menu.** The menu for empty space in a lane is regrouped, and gains Stretch and Fit to Length submenus. Clip Properties is listed and greyed out until it is built.
- **Calmer light theme.** Surfaces are a little darker for better contrast, the pale blue fills are gone, and the selected track name is dark instead of white on pale blue. The dark theme is unchanged. The Voice Chain panel is easier to read in the light theme too.

## Fixes

- **Clicking a marker clears the selection.** Press Space and playback starts from the marker. Before, it jumped back to the old selection. A range marker still selects its span.
- **The Waveform editor's grid and selection show in the light theme.** They were white on a light background. A "Selected track lane" color set in Settings now also colors the selection.
- **Dropping several files side by side** no longer puts a file on top of audio that is already on a track. The first file goes where you drop it, the rest go on the free tracks below, and new tracks are made when none are free.

## Behind the scenes

- The full document and licence review for this release is done. The Rust and npm dependency licences are clean (see `docs/doc-review-0.70.0.md`).
- The Opus decoder copy that ships with the source now includes its licence file.

## Known limitations

Altered Audio Editor is young. Windows is the platform we test. Please keep a backup copy of any session that matters.

- **Session import:** old Cool Edit and Audition `.ses` files are not read yet. Effects, envelopes with points and time stretch are not brought in from Audacity projects.
- **Clip Properties** in the multitrack right-click menu is greyed out until it is built.
- **Linux and macOS are experimental.** They are built from the same source but have had little testing. The macOS app is not notarized: the first time, right-click it and choose Open. A Linux arm64 build is planned for a later release.
- **ASIO has not been tested on real hardware.** If an ASIO driver cannot open, the system driver plays instead.
- **VST3 plug-ins, MIDI tracks, buses and 3D Space have had little hardware testing.** A plug-in that crashes is kept in its own process, so the editor should carry on, but tell us which plug-in it was.
- **Transcription, filler word removal, text-based editing, Music Remover and publishing from inside the app** are planned for after 1.0. Nothing in the app uses the network without you turning it on.
- **Some tests are by hand only.** Recording behavior, live meters and the look of the window are checked by a person, not by the automatic tests.
