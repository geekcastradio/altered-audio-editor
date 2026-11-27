# Hand test for 0.70.3

Things to try on Windows. Each line says what to do and what should happen.

## Spectrum
1. Open a file in the Waveform editor, switch to Spectrum, draw a box. Drag its edges, then type exact start, end and frequency values: the box follows.
2. Pick a frequency band: it selects the whole band across the time range.
3. Press Space with a box selected: the area plays and a line moves across the view. Press Space again: it stops.
4. Click Repair with the strength slider at 100, undo, then at 30: the 30 percent fix is clearly gentler.
5. Switch off R, then draw and Repair: only the left side changes. Switch R back on and check the right side is unchanged.
6. With only L on, Play Selected Area plays the left side only.

## Transport bar
1. Click L: the right side goes quiet. Click R instead: the left goes quiet. Click M: both sides play as mono. Switching them off restores normal playback. The file and exports are unchanged.

## Interface size
1. Settings > Appearance: set 150 percent. Panels, text, Time Info and menus all grow. Check no panel overlaps or clips.
2. Try 90 and 200. Press Reset on the page: back to 100.
3. Close and reopen the app: the size is kept.

## Convert Sample Type
1. Open a 44.1 kHz WAV, run Convert Sample Type to 48 kHz. The notice names the new file and it opens. The original file on disk is unchanged.

## Video
1. Open an ordinary H.264 MP4: it plays as before (the built-in decoder is off).
