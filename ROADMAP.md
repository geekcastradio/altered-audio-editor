# Roadmap and starter issues

What is planned, and where outside help is most useful. Everything here runs locally: no accounts, no telemetry, no network calls. See [CONTRIBUTING.md](CONTRIBUTING.md) for how to take one.

Sizes: **S** = an afternoon, **M** = a weekend, **L** = talk to a maintainer first.

## Starter issues

Copy each into a new GitHub issue with the labels shown. Items already built are not listed.

### 1. QuickPass panel layout
Labels: `help wanted`, `ui`
The QuickPass panel is cramped in the narrow left column and scrolls sideways. Rework it so there is no horizontal scroll and controls wrap, without changing behavior (details in docs/backlog.md, "QuickPass layout").

### 2. Window on launch: start empty
Labels: `good first issue`
0.1.8 stops listing a blank, untouched session as "Untitled Session" in the Files panel. The rest is open: the app should open to a clear "Open or record something" start state, with a setting for last session or new session. Check that crash recovery still prompts when it should.

### 3. Voice Chain slot menu grouped by family
Labels: `help wanted`, `effects`, `ui`
The "add effect" menu in a Voice Chain slot is one flat list. Group it by family (Dynamics, EQ, Noise, Time and pitch, Tools), driven by registry data rather than a hard-coded list.

### 4. Test on a Mac: microphone permission and recording
Labels: `needs hardware testing`, `macOS`, `help wanted`
A config-only fix for a silent-recording problem on macOS needs a real-Mac check. Install a build, record 10 seconds, and report whether the permission prompt appears and the waveform moves. Also run `codesign -d --entitlements - <app>` and paste the result.

### 5. Test on Linux and with audio interfaces
Labels: `needs hardware testing`, `Linux`, `help wanted`
Run the app on your distro (PipeWire, PulseAudio or ALSA), record and play back, and report anything odd. For Windows users with an ASIO interface: open Edit > Settings > Devices and report whether it opens, with your interface name and driver version.

### 6. Rust test: incremental waveform levels match a rebuild
Labels: `help wanted`
Write a test for the live recording waveform, which now keeps its peaks in the same form as the finished file. Write a test that builds peak levels incrementally (frames appended in chunks of different sizes) and checks they are byte-identical to levels built from the whole file in one pass. See `levels_for` in the peak cache module.

### 7. Cache folder: first-run note, sync-folder warning, move with copy
Labels: `help wanted`
0.1.8 defaults working files to Documents\Altered Audio Editor\Cache and lets the user pick another folder. Still open: a first-run note saying where files go, a warning when Documents is inside a sync folder such as OneDrive, a fallback notice when a chosen folder goes missing, a "move" that copies existing files with progress, and a second overflow folder. Recordings never get a Clean button.

### 8. Apply to Whole File as a background task
Labels: `bug`, `help wanted`
0.1.8 refuses to apply a result to a different file than it started on. Better: run Apply as a background task tied to its file, with progress in the status bar and a Cancel, so several files can be processed in turn.

### 9. Work without a multitrack session
Labels: `help wanted`, `ui`
The app should not need a multitrack session. With nothing open, show a start screen (open an audio file, new session, record, recent files); File > Close should close the session instead of starting a blank one; the multitrack view appears only when a session exists. Needs a core command to close the session. Details: docs/backlog.md, item 3e. Pairs with issue 2.

## Where the project is heading

These are the goals behind the bigger features, so you can build toward them without a long spec. Each describes what the user should be able to do. Names, layouts and code are always our own.

- **Scripting and assistants.** Users can automate repeat jobs with small sandboxed scripts, and an optional local-only endpoint lets an assistant read the session. No network, off by default, every change one labelled undoable step. Schemas for every file format already live in `/schemas`.
- **Time and pitch.** Stretch or shift speech without a chipmunk or robot sound. Stretch and tape-style speed changes exist; the goal is better quality on voices.
- **Plugins.** Load VST3 effects in a separate helper process, so a bad plugin can never crash or freeze the editor.
- **Audio hardware.** Reliable ASIO and system drivers, buffer and latency controls, per-channel inputs and outputs, and clear messages when a device gives no sound. Hardware testers are the bottleneck here.
- **Controllers.** Map MIDI faders and pads to mixer and transport, then trigger sound effects live.
- **Markers, video and ADR.** Markers that carry meaning (chapters, ad slots, social clips) and flow into exports. A reference video track and replacement-dialogue cues come later.
- **Podcast workflow.** One flow from raw recording to published episode: local transcription, text-based editing, reviewed filler-word cleanup (a human keeps or cuts each item, never automatic), chapters in the file and the feed, loudness checks, and a one-click export package.
- **Effects.** Time-based effects (delay, echo, modulation, reverb), master and bus tracks, automation, and analysis panels. Everything non-destructive and undoable.

## Bigger areas (talk to a maintainer first)

- Master and bus tracks, automation modes
- Plugin hosting (VST3) in a separate helper process
- Local transcription and caption export
- Chapter markers in exports

The detailed plan is in [docs/backlog.md](docs/backlog.md).
