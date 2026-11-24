# Altered Audio Editor 0.69.0

Audacity projects open, four new workspaces, a new default spectrogram palette, and a round of fixes to the Waveform editor, Markers and QuickPass from tester reports.

## New

- **Open Audacity projects.** File > Import Project reads `.aup3` and `.aup4` files as a new, unsaved session. Tracks, linked stereo pairs, clips with their trims, gain, pan, mute, solo and label markers come across. The audio is written as WAV files in a folder named after the project, beside it, so the session does not depend on the original file. Track effects, clip envelopes with points and time stretch are left out, and the notice after import lists them. Opening the file from Open Recent works too. The original file is only read, never changed.
- **Four more workspaces.** Production Desk, Loudness, Video Edit and Broadcast join the built-in list in Window > Workspace. Default is unchanged, and each can be reset like the others.
- **Roseus is the default spectrogram palette.** It is easier to read in the Spectrum view than the old one. The old palette is still there as "Classic" in the Spectrogram colors menu.
- **Swap Stereo Channels and Split Stereo to Center Mono** are in the Track right-click menu and the Multitrack menu. They set the Source Channels of the stereo clips on the selected track in one undo step. No audio is rewritten.
- **Follow button in the Waveform editor.** The Follow Playhead setting now also governs the Waveform editor, with a toggle in its toolbar.
- **Close button on every dialog.** Nine older dialogs (Analysis, Keyboard Shortcuts, Controllers, Plug-in Manager, Edit Notes, Isolated Clips, Output Settings, Sample Settings and Settings) now have the same title bar as the rest, with a close X at the right.
- **Side panels can be narrowed to a strip.** Drag Files, Voice Chain, Audio FX, Libraries and the others down to a thin strip to push a column out of the way.

## Fixes

- **Clicking a marker brings it into view.** In the Markers list and in QuickPass results, the Editor now scrolls to the marker you pick when it is off screen. It used to move only the playhead.
- **Clicking a range marker selects the whole range**, so you can play it and loop it. Go To in the marker's right-click menu does the same.
- **QuickPass Undo restores the list.** After Strip, Split or another fix, Undo now puts the findings back in the list, active again and in their old positions.
- **Waveform editor double-click** selects what is in view, not the whole file. Select All still selects everything.
- **Zoom to Selection** fills the view from edge to edge. Before, it left a gap on each side.
- **A playing selection that fills the view no longer shifts the view** near its end.
- **Zooming into the Waveform editor** no longer gets choppy and then jumps to finer detail. Exact samples now draw wherever a pixel covers only a few frames.
- **The low rows of the Spectrum view** are smooth instead of flat blocks, and the resolution follows the zoom.

## Known limitations

Altered Audio Editor is young. Windows is the platform we test. Please keep a backup copy of any session that matters.

- **Audacity projects:** effects, envelopes with points and time stretch are not brought in. Project history in `.aup4` files is not read.
- **Save Selection As and codec options on Save** in the Waveform editor are not built yet.
- **Linux and macOS are experimental.** They are built from the same source but have had little testing. The macOS app is not notarized: the first time, right-click it and choose Open. A Linux arm64 build is planned for a later release.
- **ASIO has not been tested on real hardware.** If an ASIO driver cannot open, the system driver plays instead.
- **VST3 plug-ins, MIDI tracks, buses and 3D Space have had little hardware testing.** A plug-in that crashes is kept in its own process, so the editor should carry on, but tell us which plug-in it was.
- **Transcription, filler word removal, text-based editing, Music Remover and publishing from inside the app** are planned for after 1.0. Nothing in the app uses the network without you turning it on.
- **Some tests are by hand only.** Recording behavior, live meters and the look of the window are checked by a person, not by the automatic tests.
