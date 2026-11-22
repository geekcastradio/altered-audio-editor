# Hand test: 0.57.0

Nothing should look or act differently. A quick pass:

- Multitrack: show a clip's volume line. Click to add a point, drag it, drag two selected points together, and check the dB label ("+3.5 dB") and the snap to 0 dB near the middle. Switch to pan and check the C, L and R labels.
- Silence Time Selection in Selected Clips: select a range over a clip. A short dip appears in the volume line with small ramps, and one undo removes it.
- Clip > Time Stretch: type a length like 1:30 or 45.5. The percent text reads like "125%".
- Open two sessions: the tab bar shows. Close the front one and the neighbor comes to the front. A session with unsaved changes shows a star.
- Hide a few tracks, save the set as a visibility preset, load it in another session with matching track names.
- Open, play, edit, save, close and reopen a session: nothing different from 0.56.0.
