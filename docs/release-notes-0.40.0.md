## Altered Audio Editor 0.40.0 (in progress)

### Video
- A details line in the Video panel shows the codec, size and frame rate, how long each picture takes, and how many took longer than one frame. It is there so a report of "the video lags" comes with numbers.
- New checkbox, "Try hardware decoding for smoother playback." Off by default. It asks the system for the graphics hardware to decode the picture, and if that fails the video opens the usual way. The choice is remembered.
- When Windows has no decoder for a file's codec, the message now says so and names what to install (the HEVC and AV1 extensions from the Microsoft Store) instead of showing a raw system error.

### Docs
- New docs/why-tauri.md: why the app uses Tauri, what that costs, and what measurements would make us reconsider a native interface.
- User Guide, Video: how to read the details line and when to try hardware decoding.

(Guide layout work, new diagrams and the remaining Audition hotkeys are still to come in this release.)

### User Guide (added)
- Long paragraphs are now bullet lists, one feature per bullet, in 30 places across the guide.
- Six new diagrams: Fill Single Click before and after, a double-ender, comping takes, a process chain, panel keys, and multi-mic bleed.
- The contents list is wider and wraps long titles with proper spacing (fixes the crowded "Multi-mic shows and double-enders" title).
- docs/guide-screenshots-wanted.md lists the real screenshots to capture next.

### Project docs (added)
- New docs/clean-room.md: how the project keeps Adobe code, assets and text out, for contributors and anyone who asks.

### Welcome and guide pictures (added)
- Welcome pages now have pictures: automation, controllers, takes, chains and Fill Single Click (they named pictures that were never in the program).
- New Welcome page for upgrades: "Shortcuts now follow Audition", with the panel and playhead keys.
- User Guide: diagrams for Automation and Hardware controllers.

### Audition transport keys (added)
- K stops and L plays, as in Audition. Ctrl+Shift+Space plays and pauses like Space. Ctrl+L turns Loop Playback on and off in the Waveform Editor. All four import from an Audition shortcut export.
- Not built: J (play backward). L plays but does not speed up on repeated presses.
- New menu items: Multitrack > Play and Multitrack > Loop Playback.

### Fixed
- Zooming while a file plays no longer makes the playhead vanish. Follow Playhead now lets go only when you scroll the view by hand, not when you zoom (Waveform Editor and Multitrack). Reported by Garretiko.

### Project documents (refreshed)
- ROADMAP.md rewritten: where the project is, the way to 1.0, what comes after, what is not planned and how to help. The old starter-issue list, which referred to 0.1.8, is gone; open issues carry the labels.
- SECURITY.md: supported versions and a fuller scope (file parsing, installer and updater, the plug-in helper).
- PRIVACY.md: a note on built-in speech models, local video decoding and Report a Bug.
- .gitattributes: more file types marked binary.
