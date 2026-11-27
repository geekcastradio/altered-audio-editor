# Hand test for 0.70.4

Things to try on Windows. Each line says what to do and what should happen.

## Spectrum playhead
1. Open a file in the Waveform editor, show the Spectrum view, draw a box and press Space. A line moves across the selection for the whole play.
2. Press Space to stop, then play again five or six times in a row, once by clicking Play and once with Space. The line shows every time.
3. Press Space twice quickly: only one play starts and the line stays.

## Lasso
1. Switch to Lasso, draw a wobbly outline with many points, then click Play. The sound starts about as fast as it does with Box.
2. Remove, -12 dB and Repair on a Lasso selection still change only the area inside the outline.

## Box and edges
1. Drag a box all the way to the bottom. Low reads 0 Hz. Type 0 into Low: it stays 0.
2. Remove a box that reaches the bottom: sound under 30 Hz inside it goes too. A box that starts at 200 Hz leaves sound under 30 Hz alone.
3. Drag a box to the bottom right corner. Its dashed border shows on both of those sides.
4. Scroll the Spectrum view up and down. The top frequency mark never sits on the "Hz" heading.

## Video decoder switch
1. Edit > Settings > Playback and Recording: the Video box "Use the built-in video decoder first" is unticked.
2. Open an ordinary H.264 MP4 with the box off, then on, and press OK each time. The picture shows both ways and reloads when you press OK.
3. With the box on, open a file the built-in decoder cannot read (an AV1 or HEVC file). It falls back and still shows, or stays blank as before.
4. Press Reset on that page: the box goes back to unticked.
