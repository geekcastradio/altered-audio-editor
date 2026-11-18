# Hand test 0.40.0, part 7: zooming while playing keeps following (issue about the playhead leaving the view)

Settings > Editor > "Scroll to follow the playhead" must be on.

1. Waveform Editor: open a file, zoom in so a few seconds fill the window and press Play. At the right edge the view pages and the playhead returns to the left. Good.
2. While it plays, scroll the mouse wheel to zoom out a little. The playhead should stay visible. When it reaches the new right edge the view should page, as before the zoom.
3. Zoom in and out several times while it plays, including with the zoom buttons and View > Zoom. The playhead should never vanish off the edge and stay gone.
4. Now scroll the view by hand while it plays (drag the overview bar or the horizontal scrollbar). The following should let go, as before, so you can look at another part. Stop and press Play again: it follows again.
5. Move the playhead (click on the ruler) while playing: the following comes back.
6. Repeat 1 to 5 in the Multitrack Editor. Same behavior.
7. Zoom with the playhead off screen to the left (you scrolled away, then zoomed): it should still wait for you, not jump back.

If the playhead can still vanish, tell me the zoom method you used (wheel, button, key) and whether the view had been scrolled by hand first.
