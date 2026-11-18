# Hand test 0.39.0, part 2: playhead keys

Open a session with some audio, then repeat in the Waveform Editor with a single file.

1. Press Left and Right. The playhead moves one pixel at the current zoom. Hold the key: it keeps moving. Zoom in and out and check the step gets smaller and larger.
2. Press Page Down and Page Up. The playhead moves one screen width.
3. Drag a time selection, put the playhead elsewhere, then press Q: the playhead goes to the start of the selection. W goes to the end. With no selection, neither key does anything.
4. Press Ctrl+J, type 1:05.250 and press Enter: the playhead lands at 1 minute 5.25 seconds. Type "soon": a notice says it isn't a time. Type a time past the end: it stops at the end.
5. Multitrack > Go To has Step, Page, Selection Start, Selection End and Time... items showing the same keys.
6. Click a slider (a fader or a knob row), then press Left and Right: the slider moves and the playhead does not. Click a drop-down or a list and use its arrow keys the same way.
7. While audio plays, press Left, Right and Q. Nothing should stutter or stop playback.

Anything that misbehaves: send me the key, which editor you were in and what happened.
