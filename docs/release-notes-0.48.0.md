## Altered Audio Editor 0.48.0

### New
- Drag the edges of a time selection with the mouse. Press on the left or right border of a selection and drag: that border moves and the other stays put. The pointer turns into a resize arrow when you are over a border. It works in the Waveform Editor, and in the multitrack view with the Time Selection tool. Snapping follows your Snap settings. Shift-click still picks the nearest border. (Issue 125, thanks Garretiko.)

Export results, export cancellations and the "session upgraded" message now appear in the middle of the window instead of the status bar. Click the message to close it; it also clears itself after 12 seconds and never blocks work.

### Changed
- Opening two or more audio files (File > Open, a recent file, from your file manager) no longer builds a multitrack. They go into the Files panel. To build a multitrack, drop the files on the Editor together, or drag them from the Files panel onto tracks. One file still opens on its own in the Waveform editor.
- The session tabs above the Multitrack stand out more: a darker bar behind them, space around them, rounded tabs with a border, and an accent line and bolder name on the tab in front.
