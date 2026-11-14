# Altered Audio Editor 0.27.0

Move your cut to other editors, a gentler voice EQ, ready-made show layouts, and one look for every dialog.

## What's new

- **Export Edit Lists.** File > Export > Edit Lists writes your cut so another editor can open it. Pick EDLs (one file per track, the usual CMX3600 type), a marker list for DaVinci Resolve, or an OpenTimelineIO file. Choose the frame rate, including 29.97 drop frame, and the timecode the session starts at. Existing files are never overwritten. The dialog lists what these formats cannot carry: fades, automation and effects stay behind, and muted, looped, reversed and stretched clips are reported.
- **Match EQ to a neutral voice.** Effects > Match EQ has a new "Match to" choice: another track, or a neutral voice target. The voice target builds a gentle Parametric EQ that moves no band by more than 6 dB, so you hear a small nudge and not a different voice. Pick a time selection of plain speech for the best result.
- **Starter session templates.** File > New Session now offers Two-Host Show, Solo Voiceover and Audiobook. Each sets the sample rate, names and colors the tracks, and marks the music track to duck under the voice. They are ordinary templates: rename, change or delete them, and a deleted one stays deleted.
- **One look for every dialog.** Time Stretch, Fit Music to Length, Add Tracks, Generate, Recording Options, File Tags, New Audio File, Pitch, Match EQ, Auto Mic Mix, Sync Tracks, Export Markers, the clip color picker, Check Loudness, Loudness Match, Publish Package, Export Mixdown, New Session, Session Properties, About, Update, Welcome and the User Guide now share one design: Cancel on the left, the main action last in the accent color, the cursor in the first field, Enter to apply, Escape to close, and a dim button says why it is dim. In Match EQ, Auto Mic Mix and Sync Tracks the colored button is now the action, not Close.

## Good to know

- The marker list for DaVinci Resolve follows Resolve's own marker export. Check the first import before relying on it.
- Edit lists carry clip positions, names and source files. An EDL holds one clip at a time, so a track whose clips overlap is reported.
- The neutral voice target is a starting point. Trust your ears over the numbers.
- A few large windows still keep their own design and move over in a later release: Analysis, Plug-in Manager, Controllers and Keyboard Shortcuts.
