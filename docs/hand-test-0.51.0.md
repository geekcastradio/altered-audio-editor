# Hand test: 0.51.0

Nothing should look different. This release moved logic into the core, so test the places that use it.

- Window > Drive Manager (or the Drive Manager panel): the list shows your drives, then Home, Desktop, Documents, Music and Downloads. Open a folder and preview a file.
- File > New Session: the suggested folder is Documents\Altered Audio\Sessions. Create one.
- Make an edit in a saved session, wait for the autosave (or close without saving), reopen the app: the recovery offer appears and recovers it.
- Open a file in the Waveform editor. Apply a Voice Chain to a selection (the progress shows, the result plays). Save it in place. File > New Audio File and Paste to New File both open an untitled file.
- In the Spectrum view, box a sound and Reduce it, then Play Selection. Both work.
- QuickPass: scan a clip for clicks and for levels. The progress runs and results appear.
- Batch Convert: convert two or three files with a Voice Chain preset. Per-file progress shows.
- In a new, unsaved session, arm a track and record a few seconds. The take appears and plays (it goes to the cache folder's recordings).
- Edit > Settings > Media and Cache: the page shows the folder and sizes. Clear the waveform drawings (they rebuild). Pick another working folder, then switch back to the default.
- Save a session, close and reopen it. Nothing is different.
- Optional: Help > Check for Updates still works.
