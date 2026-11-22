# Hand test: 0.55.0

This version changes no screens. Check that nothing moved:

- Waveform editor: play with Follow Playhead on and the view scrolls along; scroll by hand and it holds; zoom and it carries on. Record into a file and the view follows the new audio.
- Drag the border of a time selection (both edges, and a very narrow selection): the closer edge is grabbed.
- Multitrack: Silence Time Selection in Selected Clips puts a short dip in the volume line with small ramps, and one undo removes it.
- Clip level handle (bottom-left corner of a clip): drag up and right to raise, down and left to lower; hold Shift for fine steps. The dB label reads like "+3.5 dB" and disappears at 0.
- Time Info: edit Start, End and Duration, and the View row (start, end, length): values apply as before.
- Reduce Bleed on a track with some locked and some looped or stretched clips: the notice names how many were left alone. The mixdown size in Session Properties matches the file you get.
- Open a session, play it, edit a clip, save, close and reopen: nothing different from 0.54.0.
