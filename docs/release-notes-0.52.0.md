## Altered Audio Editor 0.52.0

### New
- **Help > Copy Bug Report Details now includes the startup timing.** It lists how long each stage of starting the app took (process start, core ready, interface mounted, first frame, and now settings loaded, libraries restored, recovery checked and files opened at launch). Send it with a "the app starts slowly" report. The same log is still in `startup-timing.log` in the settings folder.

### Changed
- Behind the scenes, export progress (mixdown, merge, loudness check, analysis, video mix), the plug-in scan progress, and the places where settings, backups and Documents are found now run through the core instead of the window code. Nothing looks different.
