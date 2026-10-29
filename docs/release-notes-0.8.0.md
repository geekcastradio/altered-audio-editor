# Altered Audio Editor 0.8.0

A big editing and export release: new effects, an Analysis menu, a redesigned Export dialog, better sample-type conversion, markers that survive export, and new editing tools.

## Added
- **New effects (Effects menu).** Chorus, Flanger and Phaser; Room Reverb (needs no impulse file) and Tape Delay; Distortion with four curves; Channel Mixer; Stereo Expander; and Expander / Gate with a sidechain option. Every one passes the sound untouched when its mix is at zero, adds no latency, and is covered by the repeat-render test.
- **Analysis menu.** Frequency Analysis, Amplitude Statistics, Histogram, and Find Problems, which lists clicks, clipped or flattened peaks and 50/60 Hz mains hum with times. It reports; it does not repair. All of it measures the mix, or your time selection, through the same render a mixdown uses.
- **Export dialog redesign.** Choices appear as summary boxes (format, sample rate and depth, loudness, metadata) with a Change button each, an (i) tip beside Format, and settings remembered per format. Raw AAC (`.aac`) joins M4A under "Other formats".
- **Convert Sample Type.** Presets (CD, podcast, video and broadcast, high resolution, voice only, telephone), four resample quality steps, dither shape and amount, and light or strong noise shaping. The defaults are the old behavior, so existing exports do not change. The same controls are in Export Mixdown and Batch Convert.
- **Markers in exported files.** WAV can carry every marker as a cue point with its name and range length. FLAC can carry a cue sheet. The README has a table of what each format holds.
- **Editing.** Exponential and Equal Power fade shapes (Clip > Fades > Shape, plus a Settings default); Edit > Snap Selection to Zero Crossings; Edit > Paste with Crossfade; and Loudness Match by Peak or RMS as well as loudness.
- **Edit File Tags** (File menu) for WAV, FLAC, MP3 and M4A. Only the fields you fill in are written, so chapters and audio are untouched.
- **Update dialog** shows the release notes for the new version.

## Changed
- Multitrack track headers are full-view, with a draggable width and a softer color strip. The Help menu is reordered and links to GCRN Plugins. Dialog title bars are light gray.
- Updated the `zip` library to 9.1.3 for a security advisory.

## Known limits
- Raw AAC and M4A export run on Windows only, and the raw `.aac` path has not been tried on every machine yet. Please export one by hand and report any trouble.
- FLAC cue sheets carry marker positions but no names, and ffmpeg-based players show them as chapters without titles. That option is off by default.
- Find Problems is tuned on test signals, not yet on a wide range of real recordings. Treat its findings as pointers to listen to.
- Dither and noise shaping use our own settings and are not claimed to match any other editor's presets one for one.
- Still untested on real hardware: buses and sends on real interfaces, VST3 instruments and effects, MIDI input, and Linux and macOS builds.
