# Hand test 0.48.0: drag the edges of a selection (issue 125)

1. Waveform Editor: open a file, drag a selection. Move the pointer onto its left border: the pointer becomes a left-right arrow. Drag the border: only that border moves, the other stays. Do the same on the right border.
2. Drag a border past the other one: the selection flips around the fixed border and stays valid (never empty).
3. Press on a border and let go without moving: nothing changes and the playhead does not jump.
4. Click in the middle of the selection or outside it: that still starts a new selection, as before. Shift-click still moves the nearest border.
5. Multitrack view, Time Selection tool (T): make a selection across clips. Hover and drag either border, over empty space and over clips. The border follows the pointer and the timeline scrolls if you drag to the edge.
6. With Snap on, the dragged border lands on the snap targets. Hold Shift while dragging in the multitrack view to turn snapping off for that drag.
7. Move tool and other tools: nothing about them changes.
8. Edit > Select > Adjust Selection keys (Shift+H, J, K, L) still work after a drag.

- Start with no session. File > Open, pick three audio files. They appear in the Files panel; no "Untitled Session" tab or tracks with clips appear.
- Drag the same three files from the Files panel onto the Editor together. A multitrack builds with one clip per file.
