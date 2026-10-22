# Releases 0.1.7 and 0.1.8

Neither version has been run on real hardware yet. Test before publishing.

## 0.1.7 (crash fix)

- Settings > Devices no longer loads every ASIO driver to list it; the three device lookups run one after another.
- macOS: microphone usage text and audio-input entitlement added.

## 0.1.8 contents (bump with `npm run bump -- 0.1.8`)

- Help menu: Join the Community on Discord, Source Code on GitHub, Report a Problem (opens the bug form). Show Log Files removed. Links limited to the allow list.
- A blank, untouched session is no longer listed as "Untitled Session" in the Files panel.
- Selected clips in the multitrack have a clear outline and tint.
- A recording that is silent for its whole length now explains the likely cause (microphone privacy settings, input choice).
- Apply Voice Chain no longer lands on a different file if you switch files while it runs; it says so instead.
- A second running copy no longer offers to recover the first copy's session (lock file beside each autosave, fresh for 150 seconds).
- Working files default to `Documents\Altered Audio Editor\Cache`, with Change Folder and Use Default in Settings > Cache and Files.

Also in 0.1.8 (second pass):

- Invert and Reverse in the Waveform editor (Effects menu), as undoable edits.
- Live recording waveform: the Waveform editor now draws the take as it records, and both editors draw it from peak data in the same form and block size (32 frames, coarser past 10 and 60 minutes) as a finished file, with one drawing path. It stays on screen until the finished take's own drawing is ready, so nothing flickers at Stop.
- Export, bounce, stems and playlist export show a small progress card in the corner instead of a window-blocking dialog; the app stays usable. Each export works on its own copy of the session.
- Mouse: Ctrl+wheel over a track header resizes that track, Ctrl+Shift+wheel resizes all tracks. Select All Clips on Track (track menu), Shift+click selects a range of clips on a track, and a count of selected clips shows above the headers.

Not in 0.1.8, still open as contributor issues in ROADMAP.md: QuickPass layout, empty-launch start screen and closing the session, Voice Chain slot menu, Apply as a background task, cache extras.
Known gap: with a recording offset set, the finished take is placed a little later than the live drawing; in the Waveform editor, audio after the insert point shifts right at Stop.

## Checklist

- [ ] `npm run bump -- 0.1.8`
- [ ] `npm run check`, `npm test`, `cargo fmt --check`, `cargo clippy`, `cargo test`
- [ ] Settings > Devices opens, with and without an interface
- [ ] Record a few seconds: waveform moves; unplug the input and record again: the "No sound arrived" note shows
- [ ] Open two copies of the app; the second shows no recovery prompt for the first copy's session
- [ ] A new empty window shows no "Untitled Session" row in Files
- [ ] Settings > Cache and Files: Change Folder, Use Default, files land in the chosen folder
- [ ] Help > Report a Problem opens the bug form in the browser
- [ ] Record in the Waveform editor and in the multitrack (mono and stereo): the waveform draws while recording and looks the same after Stop
- [ ] Effects > Invert and Reverse on a selection, then Undo
- [ ] Export a mixdown and keep editing while the corner card shows progress
- [ ] Ctrl+wheel over a track header resizes the track; Ctrl+Shift+wheel resizes all
- [ ] CI green on all platforms; Mac tester retests the microphone prompt
- [ ] Tag, publish the draft, mark pre-release

## Draft release notes

**Fixes**: Settings > Devices crash; macOS microphone permission; clearer clip selection; silent-recording note; Apply no longer lands on the wrong file; no false recovery prompt from a second copy; blank "Untitled Session" row hidden.
**New**: Discord, GitHub and Report a Problem links in Help; working files default to Documents with a Change Folder option.
**Known issues**: builds are unsigned (Windows) or ad hoc signed (macOS); audio, ASIO, macOS and Linux behavior is lightly tested on real hardware.
