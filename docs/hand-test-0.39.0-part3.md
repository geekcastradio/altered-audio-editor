# Hand test 0.39.0, part 3: Fill Single Click rebuilt

Use Garretiko's two clip files (Clicks_short.WAV and Clicks_(somewhat)longer.WAV) or your own recordings with clicks.

1. Open Clicks_short.WAV in the Waveform Editor. Zoom in on a click, select it (about 2 to 10 ms) and press Ctrl+Alt+U. The click should be gone with no bump or arch where it was. Play it.
2. Undo, then do the same on a wider burst (20 to 25 ms). Look at the waveform: the filled stretch should be about as tall as the sound around it, not a big hump and not a flat line.
3. In the longer clip, select the two clicks that sit close together as one selection (about 40 ms) and press Ctrl+Alt+U. It should work in one go. Play it and listen for a dropout.
4. Select 100 ms or more. You should get a message asking for 100 ms or less. Select a little under 100 ms: it should work.
5. Select the same click and fill it twice (Undo in between). The result should look the same both times.
6. Press Shift+R after filling one click, then select another click and press it again: it fills that one with no dialog.
7. Try a click in the middle of speech (a mouth click between words). Listen for a hole, a buzz or a pitch glitch. If it sounds worse than the click did, send me the file and the time.
8. Try it on a stereo file. Try a click very near the start or end of the file (it should say it is too close or fall back to a simple fill).
9. Select a click in the Spectrum view and press Ctrl+Alt+U. It should act on the same time span as the waveform view.
10. Check Help > User Guide, Waveform Editor: the text says 100 ms and Ctrl+Alt+U.

Report anything that sounds wrong with the file name, where the click is and what you heard.
