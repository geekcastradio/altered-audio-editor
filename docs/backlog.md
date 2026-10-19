# Backlog

Planned work, roughly in the order it will be picked up. Nothing here is built yet. Controls for unbuilt features show as disabled with the reason.

## Multitrack: right-click menu on an empty track (built)

Right-clicking empty space in a lane opens the session menu, with submenus: Ripple Delete, Select, Groups, Track, Insert, Marker, Mixdown Session to New File, Export Mixdown, Bounce to New Track, plus Paste and Session Properties. Every entry runs the same action as the menu bar. Entries for features that don't exist yet show disabled, with the reason as a tooltip.

Still disabled, waiting on their features:

- Add 5.1 Audio Track and Add 5.1 Bus Track (surround is not planned).
- Add Mono, Stereo and 5.1 Bus Track (bus tracks arrive with effects, P5).
- Add Video Track (video is not planned).
- Add Tracks (a dialog for adding several tracks at once).
- Insert > Silence (needs the Generate tools).
- Marker > Add Chapter Marker, Add Ad Slot, Add Social Clip (podcast mode).
- Select > Clips of Audio Type (clips have no audio type yet).

Shortcuts seen in the reference for these actions are not adopted. Every new shortcut needs approval before it is added.

## Transport bar (check what already exists first)

Loop playback and skip selection already exist as play modes. Next to Record:

- Loop playback
- Skip selection (play through the session but skip the selected range)

## Zoom buttons (below the transport)

- Zoom in amplitude
- Zoom out amplitude
- Zoom in time
- Zoom out time
- Zoom out full (all axes)
- Zoom in at in point
- Zoom in at out point
- Zoom to selection
- Zoom to time
- Zoom to selected track

## On-device AI (everything runs locally)

Audio never leaves the computer. Models download once, then work offline. No account, no cloud service, no telemetry.

Suggested order, easiest and safest first:

1. **Auto-captions.** Speech to text on the user's own machine, with word timestamps. Output as a caption track and as SRT, VTT and plain-text transcript files. Works on audio now, and on video once video exists. Candidate engine: a Whisper-family model through a Rust or C++ runtime (MIT-licensed code, several model sizes so slow machines can pick a small one).
2. **Background noise and music removal.** Two tiers. Noise suppression (hiss, hum, room tone) uses small models that can ship inside the app. Music and background removal (voice separation) uses a larger model that downloads on first use. Always non-destructive: runs as a new clip or a render, with a before and after preview.
3. **Text to speech.** Type text, get a voice track on the timeline. Candidate: a small open voice engine with a short list of bundled-license voices.
4. **Voice cloning from a few seconds of audio.** Last, because of the license and misuse questions below.

### Model downloads and the privacy rule

The app has zero network calls today, and the CI privacy guard enforces that. Downloading a model breaks that rule once, so it needs an explicit exception:

- Off by default. The only network use is a button the user clicks ("Download model, 460 MB").
- One fixed host, nothing else, and the file's checksum is verified after download.
- A manual route for fully offline machines: download the model file elsewhere and point the app at it.
- The privacy guard is updated to allow exactly that one code path and nothing else. This is a decision to approve before any of it is built.

### Open questions

- **Model licenses.** Code licensed GPL-compatible is not enough: each model's weights carry their own license, and several well-known voice-cloning models are non-commercial only. Every model gets its license checked before it is offered, and listed in `ATTRIBUTION.md`.
- **Voice cloning safety.** Consent prompt before cloning ("I have the right to use this voice"), a visible label on generated audio, and no hidden watermark removal. This is a policy decision and goes in `docs/decisions.md`.
- **Speed and size.** CPU first, GPU later. Large models make a big download and slow renders on older laptops, so the model picker shows size and expected speed.
- **Video.** Captions on a video waveform need a video track, which isn't built yet. Captions and transcripts for audio come first.

## Carried over

- Batch Convert: tags for M4A output.
- AAC encoder priming trim; HE-AAC.
- `.aup3` import (parked; see below).
- Automatic latency measurement.
- Automatic release signing.
- Effects, VST3, automation.

## QuickPass follow-ups

- Scan types beyond silence: clicks and pops, clipping.
- Presets (Trim Long Silent Pauses, Mark Non-Silent Areas) with the P5 presets.
- Keep a minimum pause (shorten long silences instead of removing them whole).

## Community requests (October 2026)

Collected from the project's Discord group. Each entry says what we will build, where it fits, and the catch to settle before building. Nothing here is started. Order of work is at the end.

### 1. Scripting, schemas and an AI-assistant endpoint

- **Scripting.** User macros and batch transforms in a sandboxed script engine. Plan: JavaScript run by an engine embedded in the Rust core (QuickJS through the `rquickjs` crate), with no file, network or process access and only a small, versioned API that calls the same actions the menus use. Every script edit goes through the undo stack as one step. Lua is the alternative; JavaScript wins because the interface and users already know it.
- **Schemas and types.** Schemas for every file format stay in `/schemas`, generated from the Rust types so they cannot drift. Add generated TypeScript definitions for the script API and the file formats. We will not publish an OpenAPI spec: there is no HTTP API, and adding one would break the no-network rule. Our real file names are `.altaudio` (session), `.altemplate`, `.alworkspace`, `.alhost`, `.alkeys`, `.alprefs` and `library.json`; the request's `.altaudio.session` style names are not used.
- **MCP server.** A small companion program speaks the Model Context Protocol over stdio and talks to the running editor through a local pipe, never the network. Off by default, switched on in Preferences with a plain explanation. Starts read-only (inspect the session); editing and export need a second, explicit switch. Every change an assistant makes is labelled in History and can be undone.
- **Phase:** after the effects engine settles the data model (P5). Schemas and generated types can start sooner.
- **Catch:** a script or assistant that can export or delete is a safety surface. Default read-only, per-action permissions, no write access to files outside the session folder.

### 2. Time stretch (shrink and stretch): built 2026-10-17 as Clip > Time Stretch; drag-to-stretch waits on the shortcut decision

- Stretch a clip by dragging its edge to fit an exact length (promo and ad slots), non-destructive, with pitch kept. Range about 25 percent either way. Also a tape-style speed change that moves pitch with speed.
- **How:** clips gain a `stretch` field (a session schema step with a migration, so old files still open). Preview uses a fast stretch; export and a cached render use the high-quality one. Candidate engines: Rubber Band (GPL, compatible with ours, but a C++ dependency and licence note in `ATTRIBUTION.md`) or a Rust phase-vocoder and WSOLA of our own. Decide with listening tests on speech.
- **Shortcut:** the request says Alt+Drag. New shortcuts need project lead approval, so this is listed, not assigned. Until approved, stretch is a Clip menu command with a length field and a tool-strip mode.
- **Phase:** P3 or P4 editing. High value for ad work. Early candidate.

### 3. Plugin hosting

- **VST3** hosting on tracks and the master, then optionally **CLAP** (an open, simpler format). The VST3 SDK is MIT licensed, so hosting inside a GPL app is clean.
- **VST2 is not planned.** Steinberg stopped licensing the VST2 SDK, so shipping a VST2 host is a legal risk for a public project. We can revisit if the terms change. This is a deliberate decision, logged in `decisions.md`.
- **Crash isolation:** plugins run in a separate helper process and exchange audio through shared memory, so a crashing plugin shows an error on its slot and the session carries on. This is the largest piece of the work and sets the schedule.
- **Plugin windows:** open each plugin's own window beside the editor. Embedding it inside the main window is fragile on every platform and is not promised.
- **Not the `nih-plug` crate:** that is for writing plugins. Hosting needs the VST3 interfaces directly (for example the `vst3` crate) or our own bindings.
- **Phase:** P5, with Realtime Export (some live plugins misbehave when a mixdown renders faster than playback).

### 4. Low-latency audio on Windows (ASIO) and discrete channels

- Add ASIO next to the current Windows audio paths, through the audio library we already use. Check the ASIO SDK licence terms before shipping a build that includes it; do not assume they fit a GPL release.
- Per-channel input and output routing so each track can take its own channel from a multi-microphone interface, plus loopback channels for system audio.
- **Phase:** P4 recording. Needs real hardware to test. Latency compensation and punch-in already sit in that phase.

### 5. MIDI and virtual instruments

- A MIDI track type with clips, playback and a piano roll, virtual instruments on those tracks (depends on plugin hosting), and external controller mapping to transport, faders, pan and trigger pads.
- **Phase:** its own phase after Podcast mode, as already planned. The soundboard-trigger use (pads that fire clips) is small and could ship earlier without the piano roll.
- **Catch:** the piano roll and instrument hosting are a second product. Controller mapping for transport and faders is cheap and could come first.

### 6. Markers, playlist and video

- **Markers:** point, range, chapter and cue markers with labels, colors and notes in both editors. Most exist already. Add colors, notes and the Waveform editor.
- **Playlist panel:** sequence marked ranges, preview the arrangement without changing the clips, and export each segment as its own file.
- **Chapter export:** a timestamp list, CSV and podcast chapters JSON are built (2026-10-17). Still to do: chapters embedded in the file itself (ID3v2 in MP3, Vorbis comments in FLAC and Ogg).
- **Video track and ADR:** import video as a reference track in sync with the audio, a resizable preview, count-in beeps and a visual pre-roll for re-recording, loop regions, audio extraction on import, and writing the finished mix back into the original video file without re-encoding the picture.
- **Catch:** a web view's video element cannot step exact frames. Frame-accurate preview needs our own decoder path, probably FFmpeg (LGPL build, dynamically linked, licence noted). Writing audio back without touching the video is a stream copy, which is the easy half. Check which containers to support.
- **Phase:** markers, playlist and chapter export in P3 to P7 as listed. Video and ADR as their own phase, tied to Voiceover mode.

### 7. Podcast tools

- **Auto-ducking:** one setting that lowers music tracks while chosen speech tracks are active. Fits the effects engine (P5). A simple version needs only volume envelopes and QuickPass's speech detection, so a first release can come before the full compressor-style version.
- **One-click loudness:** built 2026-10-17 as Effects > Loudness Match, with broadcast targets by region (US -24 by default).
- **Multi-mic alignment and bleed:** estimate each microphone's delay against the others and shift it to remove comb filtering. Delay estimation is well understood and feasible. Removing bleed itself (one voice leaking into another microphone) is harder and quality varies; ship alignment first, a gate-based bleed reduction second.
- **Silence and breath management without cutting:** built 2026-10-17 as QuickPass Turn Down (volume envelope dips, clips stay whole). Still to do: a breath detector that tells breaths from quiet speech, which needs spectral analysis.
- **Phase:** loudness and non-destructive silence now; ducking and alignment in P5.

### Order of work

1. Fix what is broken (waveforms, Levels panel) and ship v0.1.1.
2. Non-destructive silence and breath handling in QuickPass; one-click loudness targets (both built 2026-10-17).
3. Time stretch (built 2026-10-17).
4. Chapter export, marker colors and notes, Playlist panel.
5. Effects engine (P5) with plugin hosting, ducking and mic alignment.
6. Recording hardware work: ASIO and per-channel routing.
7. Scripting, then the assistant endpoint.
8. Video and ADR, MIDI.

## Built 2026-10-17 (second batch)

AAC (M4A) export on Windows, Alt+drag stretch, Generate Tone and Noise, Insert Silence, punch-in recording with pre-roll, recording latency offset, marker colors and notes.

Still open from editing and recording: Paste to New File and Mix Paste checks, Clips of Audio Type, the Playlist panel, input monitoring through the speakers, automatic latency measurement, ASIO and per-channel routing, AAC on macOS (Audio Toolbox) and Linux (bundled encoder).

## Built 2026-10-18

Silence over a time selection, audio types and Select Clips of Audio Type, Add Tracks, ad slot and social clip markers, chapters embedded in MP3 and FLAC, the Playlist panel.

Next in editing and recording: markers in the Waveform editor (saved as WAV cue points), input monitoring through the speakers, automatic latency measurement, ASIO and per-channel routing. Formats: Batch Convert metadata (copy tags across, set title and artist), AAC encoder-delay trim, `.aup3` import, AAC on macOS and Linux, HE-AAC (needs a bundled encoder).

## Built 2026-10-18 (second batch)

Markers in the Waveform editor (saved in the WAV as cue points, moved by cuts, pastes and crops, shown and edited in the Markers panel), input monitoring through the speakers, Batch Convert tags (copy each file's tags and WAV markers, set title, artist, album, year, genre and comment with name tokens).

Next: the effects phase (P5). Still open: ASIO and per-channel routing (needs the project lead's interface to test and an SDK licence check), automatic latency measurement, M4A tags, AAC on macOS and Linux, HE-AAC.

## Parked: `.aup3` import

Parked by the project lead on 2026-10-18 as a possible later feature. What it would take: an `.aup3` file is an SQLite database holding the project layout and the audio in compressed blocks. Import would read the track and clip layout and rebuild each clip's audio into WAV files beside the session, then open it like a `.sesx` import. Needs real sample projects to test against, an SQLite reader (a bundled build of the SQLite library, public domain), and our own parser written from the file's observable structure. Effects in the project would be skipped with a notice, as with `.sesx`.

## Effects (P5), first batch built 2026-10-18

Built: Voice Chain on clips and tracks, eight effects (Parametric EQ, Hum Cut, Dynamics, Limiter, Breath Gate, De-Esser, Speech Leveler, Gain), built-in and user presets, Effects menu entries, look-ahead compensation.

Next batches, in order:
1. Noise Reducer (learn a noise print from a selection, or an automatic speech model; licence check first for any model), Click Repair, hum detection.
2. Chain input and output meters; live slider changes without a restart.
3. Waveform editor: apply an effect or a chain to the selection or file (destructive, undoable), with preview.
4. Pitch and Time: pitch shift with formant keeping, speed change.
5. Master chain and buses (moves the master mix to the render thread), sends.
6. Speaker Balance, host profiles, Guest track mode, Audio FX Panel.
7. VST3 hosting in a helper process.

## Effects to build, renamed (planned 2026-10-18)

- Restoration, done: Noise Reducer, Noise Print Reducer, Sound Eraser, Hiss Reducer, Room Reducer, Click Repair, Clip Repair, DC Offset Fix, Phase Align (Clip menu).
- Filter and EQ, done: Band EQ (10, 20, 30), Curve EQ with a draggable graph, Notch Filter, Precision Filter (Butterworth, Chebyshev, Bessel, Elliptic), Center Remover, response graphs for every EQ and filter.
- Time and pitch: Pitch Shift done. Speed and Pitch is covered by Time Stretch plus Pitch Shift. Left: Pitch Curve (pitch over time), Pitch Tune (automatic correction to a scale), Pitch Draw (manual correction), formant keeping.
- Others asked for in batch presets: Center Remover (vocal removal) done, Fade In and Fade Out and Normalize done (Batch Convert level row), Telephone Voice.
- Music Fit: built 2026-10-18 (Clip > Fit Music to Length), with a choice of up to three plans and where each joins. Later: fit to the end of a time selection by dragging.

## QuickPass and Batch Convert layout (agreed 2026-10-18)

- Batch Convert (built 2026-10-18): file list with Name, Status and Stage; a processing preset picked per run (a Voice Chain preset or one effect) applied before export; an Export checkbox; Export Settings dialog with file naming (prefix and suffix, or a template), location (next to the source, or a folder), replace existing files, format list, sample type (rate, channels with left and right mix, bit depth with dither options) and format settings; remove finished files from the list; keep markers and metadata.
- QuickPass (built 2026-10-18): scan type list (Click Repair, Clip Repair, Silence, Mark Audio), presets with save and delete, Scan and Settings views, results table (fixed, start, duration, clip) with Fix, Fix All and Clear Fixed, and a footer count of problems found and fixed.


## Built 2026-10-18 (fourth batch)

Live chain edits (sliders apply while dragging, no playback restart unless an effect's look-ahead changes), chain meters in and out, Voice Chain preview and apply in the Waveform editor, Noise Print Reducer, Sound Eraser, Phase Align, Elliptic filters, response graphs with a draggable Curve EQ, Batch Convert normalize and fades, M4A tags, Music Fit plan choice.

Still open, and why:
- Pitch Tune, Pitch Curve, Pitch Draw and formant keeping. Pitch Tune needs a frame-by-frame pitch tracker driving a time-varying shift, and formant keeping needs a spectral envelope pass on top of the shifter. Each is a batch of its own, best done together since they share the envelope work.
- Master chain and buses. Today faders and the master sum run on the audio thread, so they react within one buffer. A master chain or a bus chain must run on the render thread, half a second ahead, so the mix has to move there too, and faders would then lag by that half second unless the fader stays on the audio thread after the chain. Needs a design call from the project lead before building.
- Clip chain meters (track chains are metered).
- AAC encoder-delay trim. Windows' encoder adds silence at the start; how much it adds and whether players skip it needs checking on Windows with a test file before the edit list is written.
- HE-AAC, AAC on macOS and Linux, release signing: see the notes in the session reply of 2026-10-18 (need a bundled encoder, Mac hardware, and the project lead's signing keys).

## Settings (built 2026-10-18)

Edit > Settings with every page; see decisions.md. Greyed-out items waiting on other work: channel routing and driver type (ASIO), buffer size, numbered session backups, Waveform edge crossfades, pan law (buses), spectrum detail and range, timecode frame rate, scrubbing, record countdown, weekly update check, a light theme, moving the cache folder.

## Built for 0.1.3 (2026-10-19)

Speaker Balance, host profiles with Guest track mode, session backups, record count-in, new-marker name, timecode frame rate. Next: the master chain and bus decision (see decisions.md), then Pitch Tune, Pitch Curve, Pitch Draw and formant keeping, then Waveform editor edge crossfades and scrubbing.

## Voice Chain effect windows (built for 0.1.4, first step)

Built: adding an effect, clicking its name or its gear button opens its controls in a floating, movable window inside the app (one at a time), with the response graph and Learn button, an On/Off button, and the track and slot number. Esc or the close button hides it. Still to do: several windows open at once, remembering where each window was, and separate system windows for a second monitor.

## Optional AI connections (idea, not approved)

Spitballed by the project lead: off by default, never required. Local first (a local language model, local text-to-speech, local speech-to-speech), and cloud services only with the user's own key. Needs a decision entry, a privacy review and a privacy-check change before any code. See docs/design-principles.md.
