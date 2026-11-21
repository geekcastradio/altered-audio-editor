## Altered Audio Editor 0.52.1

### Fixed
- **ASIO is back in the Windows build.** The 0.51.0 and 0.52.0 installers were built without ASIO, so ASIO showed as unavailable (issue 167). The build now fetches the ASIO SDK itself and the release says so plainly if ASIO ever fails to build again. ASIO has still not been tested with real hardware, so please report what you see.
- **The User Guide contents list no longer runs two entries together.** "Multi-mic shows and double-enders" is now "Multi-mic shows" (the section still mentions double-enders, so search finds it).
- **The startup timing log can no longer be read half written**, for example while Help > Copy Bug Report Details copies it at launch.

### Changed
- Behind the scenes, more of the interface's rules now have a copy in the Rust core, checked against the interface's own with shared test files: typed dial values and times, In and Out points, the dial and fader maths, the audiobook delivery check, the analysis colors and the Dynamics graph. Nothing looks different.
