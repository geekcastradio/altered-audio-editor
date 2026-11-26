# Altered Audio Editor 0.70.1

A bug-fix release after 0.70.0: Audition 3 `.ses` sessions open, dropping files twice no longer fails or lists them twice, Music Mode no longer sticks on, Convert Sample Type works again on a second try, and the menus are regrouped. No new keyboard shortcuts.

## New

- **Audition 3 `.ses` sessions open.** File > Open, drag and drop and File > Import > Session accept `.ses`. Track names, where each clip sits and which track it is on are read. Effects and envelopes are not brought in. A clip that plays only part of its audio file is placed as the whole file, with a notice.
- **Regrouped menus.** Edit, Multitrack, Clip, View and Window are grouped more like the editors people already know. Edit keeps Undo, Redo and Repeat Last Command together, and has a Zero Crossings submenu. Multitrack has a Tracks submenu and a Playback and Recording submenu. Clip has a Stretch submenu. View has Zoom and Waveform Channels submenus. Window lists panels in groups that follow the default layout. Every command is still there, and shortcuts are unchanged.

## Fixes

- **Dropping the same files twice** no longer fails with "The process cannot access the file" (error 32), freezes, or lists the same audio twice in the Files panel. A second drop now waits for the first and reuses its copy.
- **Music Mode stays off** after you leave the Music workspace. It was staying on, which put music items in the other workspaces.
- **The red "ResizeObserver loop" message** no longer shows in the status bar when the app opens.
- **Add to Favorites** no longer leaves a hanging box behind when you right-click a menu item.
- **Convert Sample Type** no longer refuses a second conversion of the same file. A name that is taken gets a number, the notice says which file was saved, and a real failure now says why.
- **A flaky startup test** no longer fails the build now and then. This is test code only.

## Known limitations

Altered Audio Editor is young. Windows is the platform we test. Please keep a backup copy of any session that matters.

- **Session import:** old Cool Edit and Cool Edit Pro `.ses` files are refused with a message, because their layout cannot be read safely yet. Audition 3 `.ses` files have been checked against one sample, so tell us about any that open wrong. Effects, envelopes with points and time stretch are not brought in from Audacity projects.
- **Convert Sample Type** writes a new file. It does not change the file open in the tab.
- **The left and right channel buttons** in the Waveform editor choose which channels edits reach. Playback still plays both.
- **Not yet reproduced:** 48 kHz files playing at double speed, and a blank Video panel with some files. Both need a sample file from the reporter.
- **Clip Properties** in the multitrack right-click menu is greyed out until it is built.
- **Linux and macOS are experimental.** The macOS app is not notarized: the first time, right-click it and choose Open.
- **ASIO has not been tested on real hardware.**
- **VST3 plug-ins, MIDI tracks, buses and 3D Space have had little hardware testing.**
- **Transcription, filler word removal, text-based editing, Music Remover and publishing from inside the app** are planned for after 1.0. Nothing in the app uses the network without you turning it on.
