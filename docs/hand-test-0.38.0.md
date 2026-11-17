# Hand test 0.38.0

Two changes: the Waveform Editor level knob now previews while you drag, and Effects > Fill Single Click is new.

## Level knob preview

1. Open any audio file with File > Open. Show the level knob if it is hidden (View menu, the HUD item).
2. Press and hold the knob, then drag right or up. The waveform should grow while you drag and the readout should show the dB. Drag left or down and it should shrink.
3. Hold Shift while dragging. Steps should be finer.
4. Release. The audio should change by the amount shown, and the waveform should stay at that height. Undo should put it back.
5. Select part of the file first, then drag. Only the selected part should change height while dragging.
6. Drag a large amount up. The waveform should flatten at the top and bottom edges, not draw outside the lane.
7. Drag back to 0.0 dB before releasing. Nothing should change.
8. Zoom in until you see individual samples and repeat step 2. The line should move as you drag.

## Fill Single Click

1. Open a recording with a mouth click or a pop. Zoom in until you can see it.
2. Drag a selection over just the click, a few milliseconds wide.
3. Choose Effects > Fill Single Click, or press Ctrl+Alt+K. The click should be smoothed over and the surrounding audio left alone. Play it to check.
4. Undo. The click should come back.
5. Select the next click and press Shift+R (Repeat Last Command). It should fill that one too without a dialog.
6. Select more than 25 ms. You should get a message asking you to select less.
7. Select a click within a few samples of the very start of the file. You should get a message that it is too close to the edge.
8. Try it on a stereo file and on a mono file.
9. With nothing selected, Fill Single Click should be greyed out with a reason.

Report anything that looks wrong with the file name, the selection width and what you heard.
