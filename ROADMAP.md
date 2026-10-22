# Roadmap and starter issues

What is planned, and where outside help is most useful. Everything here runs locally: no accounts, no telemetry, no network calls. See [CONTRIBUTING.md](CONTRIBUTING.md) for how to take one.

Sizes: **S** = an afternoon, **M** = a weekend, **L** = talk to a maintainer first.

## Starter issues

Copy each (1 to 15) into a new GitHub issue with the labels shown.

### 1. Invert and Reverse in the Waveform editor
Labels: `good first issue`, `effects`, `size S`
Add Invert (flip polarity) and Reverse (play the selection backwards) as selection effects. Both are simple sample operations in the Rust core, plus a menu entry each. Include unit tests. Undo must work.

### 2. Help and About links
Labels: `good first issue`, `size S`
Fill in the Help menu and About dialog with the project's GitHub and community links. External links must go through the allow list in `src-tauri/src/commands/links.rs`; add the new URLs there and nowhere else.

### 3. Report a Problem opens a prefilled GitHub issue
Labels: `good first issue`, `size S`
Help > Report a Problem should open the bug form on GitHub in the system browser, via the link allow list. Do not send any data from the app; the user types it.

### 4. Remove "Show Log Files" or make it work
Labels: `good first issue`, `size S`
The Help entry has nothing to show. Remove it, or open the log folder if one exists. Update any docs that mention it.

### 5. QuickPass panel layout
Labels: `help wanted`, `ui`, `size S`
The QuickPass panel is cramped in the narrow left column and scrolls sideways. Rework it so there is no horizontal scroll and controls wrap, without changing behavior (details in docs/backlog.md, "QuickPass layout"). Screenshots before and after in the pull request.

### 6. Window on launch: start empty
Labels: `good first issue`, `size S`
The app should open to an empty window (no session, no file) with a clear "Open or record something" start state. Check that crash recovery still prompts when it should.

### 7. Voice Chain slot menu grouped by family
Labels: `help wanted`, `effects`, `ui`, `size M`
The "add effect" menu in a Voice Chain slot is one flat list. Group it by family (Dynamics, EQ, Noise, Time and pitch, Tools). Registry data should drive it, not a hard-coded list.

### 8. Test on a Mac: microphone permission and recording
Labels: `needs hardware testing`, `macOS`, `help wanted`
A config-only fix for a silent-recording problem on macOS is waiting for a real-Mac check. Install a build, record 10 seconds, and report whether the permission prompt appears and the waveform moves. Also run `codesign -d --entitlements - <app>` and paste the result.

### 9. Test on Linux and with audio interfaces
Labels: `needs hardware testing`, `Linux`, `help wanted`
Run the app on your distro (PipeWire, PulseAudio or ALSA), record and play back, and report anything odd. For Windows users with an ASIO interface: open Edit > Settings > Devices and report whether it opens, with your interface name and driver version.

### 10. Rust test: incremental waveform levels match a rebuild
Labels: `help wanted`, `testing`, `size M`
Prepares the live-recording waveform work. Write a test that builds peak levels incrementally (frames appended in chunks of different sizes) and checks they are byte-identical to levels built from the whole file in one pass. See `levels_for` in the peak cache module.

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

## Known bugs open for help

Each is a real report. Say you are taking it in the issue before you start; for these, agree the approach in the issue first.

### 11. Settings > Cache and Files: visible default folder
Labels: `help wanted`, `size M`
Waveform drawings, effect results and unsaved recordings go to a hidden app folder. Default them to `Documents\Altered Audio Editor\` (subfolders Cache, Recordings, Sessions, Templates), let the user change each with a Browse button, warn when Documents is inside a sync folder such as OneDrive, and fall back to the default with a notice if a chosen folder goes missing. Recordings never get a Clean button. Details: docs/backlog.md, "Cache and Files".

### 12. Live recording waveform must match the finished one
Labels: `help wanted`, `size L`
In the Waveform editor nothing draws until Stop, and in the multitrack the live picture is a different drawing that changes at Stop. Build the peak cache while recording, so both editors draw the growing take with the same data and renderer as the finished file, with nothing swapped at Stop. Pairs with issue 10 (the test). Details: docs/backlog.md, "Live recording waveform".

### 13. Apply to Whole File breaks if you switch files while it runs
Labels: `bug`, `help wanted`, `size M`
The result is applied to whichever file is open when the job ends, not the file it started on. Tag every long job with its file and apply the result there. A "please wait" lock on file switching is an acceptable first step.

### 14. Multitrack export freezes the app, and a second copy offers to recover a live session
Labels: `bug`, `help wanted`, `size M`
Show export progress in the status bar with Cancel instead of a blocking overlay, and have each running copy write a lock file with its process id so crash recovery skips autosaves from a running copy.

### 15. Silent recording note
Labels: `good first issue`, `size S`
When a recording is silent from the first second, show one line naming the likely cause (microphone privacy setting on Windows, System Settings > Privacy & Security > Microphone on Mac) instead of a flat line with no explanation.

## Bigger areas (talk to a maintainer first)

- Recording waveform that matches the final file exactly
- Moving the cache and recordings to a visible, changeable folder in Documents
- Background jobs with progress and cancel, so work continues during long exports
- Master and bus tracks, automation modes
- Plugin hosting (VST3) in a separate helper process
- Local transcription and caption export
- Chapter markers in exports

The detailed plan is in [docs/backlog.md](docs/backlog.md).
