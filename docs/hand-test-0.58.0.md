# Hand test: 0.58.0

Nothing should look or act differently. A quick pass:

- Multitrack: zoom in and out with the wheel and with Ctrl+= and Ctrl+-, scroll, and use Zoom Out Full. The ruler labels stay readable at every zoom and the clock and timecode read as before.
- Drag the edges of the overview strip: the lanes zoom and the far edge stays put. Collapse a track and resize another.
- File > Check Loudness on a mix: the card shows the same verdict and advice as in 0.57.0. Pick another target (broadcast, custom) and an export with a loudness target: the predicted result line reads sensibly.
- Recording: with no selection, record from the playhead. With a time selection and punch-in on, recording starts a little before it, stops at its end, and the take lands inside the selection. Try a recording offset of about 25 ms.
- Open, play, edit, save, close and reopen a session: nothing different from 0.57.0.
