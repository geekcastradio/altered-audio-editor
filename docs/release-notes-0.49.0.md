## Altered Audio Editor 0.49.0

### New
- **Import OpenTimelineIO files (issue 120).** File > Import Session (.otio) opens a timeline another editor saved, such as Kdenlive, as a new unsaved session. Audio tracks, clips (their place, length and where in the file they start), gaps and markers come across. Fades, volume, pan, effects, transitions and speed changes are not in the format in a form every editor agrees on, so they are left out, and the notice after import says what was left out. A clip whose file cannot be found is still placed so you can relink it. It reads timelines AAE writes (File > Export > Edit Lists) and back.

### Changed
- The Voice Chain, Batch Convert and QuickPass group at the bottom left can now be dragged down to just its tab strip when you are not using it. Its panels scroll when the group is short.
- When a file cannot be copied into the session folder because the original is gone, the message names the file and where it was expected.
- Spectrogram color schemes (issue 141). Right-click the spectrogram and choose Spectrogram Colors to pick a scheme. **Default** is now the purple, red, orange and yellow look. **AAE** is the earlier teal and gold look. Ember, Ocean, Violet, Forest and Gray are also there, and you can still set your own quiet and loud colors. If you never changed the colors, your spectrogram now shows the new Default; pick AAE to get the old look back.
