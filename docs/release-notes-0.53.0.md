## Altered Audio Editor 0.53.0

### Added
- **Matroska and WebM audio files open.** `.mka` (audio-only Matroska) and `.webm` files now import like any other audio file, from the Open dialog, a drop or a folder scan (issue 174). Opus, Vorbis, FLAC, AAC and MP3 inside them all play. `.mkv` already opened.

### Fixed
- **Matroska audio no longer imports with a length of zero.** Opus, Vorbis, FLAC and AAC inside Matroska reported no length, so a file could import as an empty clip. The length is now worked out from the packet times. It can be off by up to about one packet (a few hundredths of a second) and errs long, so no audio is cut off.
- **The Waveform editor follows a recording again** (issue 178). While you record into a file, the view scrolls to keep the end of the new audio in sight. This had stopped working since 0.42.0, and recording into an empty file left the view stuck at the start. Scrolling by hand still holds the view until the next take.

### Changed
- Behind the scenes, more of the interface's rules now have a copy in the Rust core, checked against the interface's own with shared test files: the time display, the clip selection rules (Edit > Select), the Report a Problem link, the number formats and scales in the Analysis panel, and the arithmetic of Loudness Match. Nothing looks different.
