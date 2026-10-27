# Altered Audio Editor 0.5.1

## Fixed
- Mix Down and Bounce of selected clips now include MIDI clips, placed by the session's tempo. Before, a selection of only MIDI clips could not be exported, and a mixed selection left the MIDI out.
- Exporting with VST3 effects starts each plug-in once instead of twice, so exports begin sooner.
- Ctrl+Alt shortcuts read the physical key, so they work with AltGr keyboards on Windows.
- Files with more than two channels read their saved peaks back correctly.
- Removing a file from the Files panel, or using Remove Unused Files, closes it in the Waveform editor (asking first if it has unsaved edits).

## Changed
- The Waveform editor fits the file exactly: zoom to fit no longer adds empty time after the end, and scrolling stops at the end of the file.

Several of these fixes come from the Audiform project (GPL-3.0-or-later). Thank you to its developer.
