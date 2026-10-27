# Altered Audio Editor: release notes since 0.3.0

Two releases are covered here. Use the 0.3.1 section for the updater release and the 0.4.0 section for the multichannel release. If you ship both together as 0.4.0, paste them one after the other.

---

## 0.3.1

### In-app updates
- The editor now checks for new versions and can install them for you. Open Help > About or Settings to check by hand.
- Installed copies (the setup installer) update themselves in place. Portable copies (the zip) open the release page instead, since they have nowhere to install to.
- A check runs about once a week in the background. It only contacts this project's own GitHub releases, sends nothing about you, and can be turned off in Settings.
- Updates are signed. The editor refuses anything that does not match the project's key.
- Every release is now published as a normal release, so "latest" always points to the newest version.

### Voice Chain panel
- Rebuilt with a clearer layout, in the app's own colors.
- The slot list now grows and shrinks with the panel, so it uses all the room you give it.
- The Input, Output and Mix block and the four buttons are pinned to the bottom of the tab at any size.
- Button 1 turns all effects on or off. Button 2 shows or hides the Input/Output/Mix controls.
- Buttons 3 (pre/post fader) and 4 (pre-render track) are in place but greyed out until the engine work for them lands.
- Knob values now show a sign and one decimal place, for example +3.0.

### Editor
- The overview bar now runs the full width of the Editor, all the way to the left edge.
- New zoom commands: Zoom Out Full, Zoom In Vertical, Zoom Out Vertical, and Zoom All Axes.
- New Nudge Left and Nudge Right commands for selected clips. The step is one screen pixel at the current zoom, and locked clips stay put.
- Track height can be changed for all tracks at once.

### Under the hood
- Amp Model internals renamed for clarity. No change to how it sounds.
- Privacy check updated so the updater is only allowed to talk to this project's own releases.

---

## 0.4.0

### Multichannel files
- Files with more than two channels (5.1, 7.1 and similar) now keep every channel instead of being folded to stereo on import. Up to 64 channels are supported in the waveform cache.
- Each channel is drawn in the editor.
- Clips from multichannel files get a Source Channels section in Properties. It sets how much of each channel goes to left and right, with presets.
- The default fold-down keeps front channels straight, sends center and surrounds at -3 dB, and leaves the LFE out.
- Stereo and mono files are unchanged and do not show the new section.

### Channel Routing
- New Channel Routing settings, plus a Speakers listen mode, so each speaker can play from its own output on a multichannel audio device.
- The audio device opens at its widest setting when it needs to.

### Waveform editor
- Multichannel files open with every channel shown, each with its own button.
- Edits and effects apply across channels, and saving keeps every channel.
- Multichannel files save as WAV. FLAC and MP3 are refused above two channels, with a clear message, rather than silently losing channels.

### Fixes
- Shift+Space now records.
- Removed an old rule that dragged the view back to the playhead while you scrolled.

### Credit
Multichannel work in this release started in the Audiform fork by nwh212 and was brought over with changes to fit this project. Thank you.

---

## Test checklist before publishing
- Stereo and mono playback sound the same as 0.3.0
- Recording works and lands on the right track
- Mixdown of a normal stereo session matches 0.3.0
- A multichannel WAV opens, plays, and saves with all channels in the Waveform editor
- Channel Routing at its defaults changes nothing for a stereo setup
- Music Mode on and off still behaves as in 0.3.0
- Self-update: install 0.3.1, then publish a 0.3.2 test release and confirm the app offers it
