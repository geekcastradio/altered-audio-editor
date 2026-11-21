## Altered Audio Editor 0.51.0

This one is mostly behind the scenes. Nothing in the editor should look or sound different. If something does, that is a bug, so please tell us.

### Changed
- **More of the app's logic moved out of the window code into the core.** The Drive Manager's folder list, where autosave and new sessions go, the Waveform editor's working files and progress, QuickPass and Batch Convert progress, spectral edits, recording to an unsaved session, and the Settings > Media and Cache page now run through the core. This is groundwork for the native interface.
- **The README was checked claim by claim.** The Speech Denoiser line now names both models, OTIO import is listed, the update line says the install-at-start option exists (off by default), the file formats table includes the preferences file, and a feature that did not exist was removed. The line about audio on the CPU now covers live audio only.
- Unused interface code was removed.

### For testers
- **A native interface spike** lives in `native-spike/` in the source. It is not part of the app and is not in these downloads. The "Native spike" workflow on GitHub builds a Windows test program you can try; see `native-spike/README.md`.
