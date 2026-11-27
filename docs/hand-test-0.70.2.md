# Hand test for 0.70.2

Things to try on Windows. Each line says what to do and what should happen.

## Space at the end of the audio
1. Open a session with a clip. Click the ruler past the end of the clip, with nothing selected. Press Space: playback starts from the beginning, not into silence.
2. Press Space to stop, then click the ruler in the middle and press Space: it plays from where you clicked and stops at the end.
3. Press Shift+Space with the playhead at the end: recording starts where the playhead is. Stop it.
4. Make a time selection and press Space: it plays the selection as before.

## HE-AAC files
1. Import an HE-AAC `.m4a` that is labelled 48 kHz (the file from issue 321 works). It shows 24,000 Hz and its real length in the Files panel. Hover it: the codec reads "AAC (HE, without high band)".
2. Play it: normal speed, not double. It sounds duller than the original.
3. Import an ordinary AAC `.m4a` (not HE): rate, length and codec label are unchanged.
4. Drag the HE-AAC file onto a track next to a WAV: they run in sync for the whole file.
