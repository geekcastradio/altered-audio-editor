## Altered Audio Editor 0.54.0

### Added
- **One-job starter chains in Batch Convert.** The Chain picker now offers a pick for each common single job: peak to -0.1 dB or -3 dB, fade in or out, hard limit, voice-over compressor, de-ess, reduce noise, reduce hiss, reduce room echo, remove 60 Hz hum, remove vocals, fix DC offset, telephone voice, and raise or lower pitch. A new De-Ess Voice preset backs the de-ess pick.

### Fixed
- **QuickPass only offers tracks that have audio.** The Tracks menu no longer lists empty tracks, and the button only shows when two or more tracks hold audio.

### Changed
- Behind the scenes, more of the interface's rules now have a copy in the Rust core, checked against the interface's own with shared test files: track names, the Speaker Balance arithmetic, the file tag helpers and type-to-jump in lists and menus. Nothing looks different.
