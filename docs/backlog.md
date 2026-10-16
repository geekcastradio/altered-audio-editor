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
2. **Background noise and music removal.** Two tiers. Noise suppression (hiss, hum, room tone) uses small models that can ship inside the app. Music and background removal (voice separation) uses a larger model that downloads on first use. Always non-destructive: runs as a new clip or a render, with an audition before and after.
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

- Batch Convert: metadata and marker export (ID3, chapters).
- AAC encoder priming trim; HE-AAC.
- `.aup3` import.
- Input monitoring, latency compensation, punch-in.
- Automatic release signing.
- Effects, VST3, automation.

## QuickPass follow-ups

- Scan types beyond silence: clicks and pops, clipping.
- Presets (Trim Long Silent Pauses, Mark Non-Silent Areas) with the P5 presets.
- Keep a minimum pause (shorten long silences instead of removing them whole).
