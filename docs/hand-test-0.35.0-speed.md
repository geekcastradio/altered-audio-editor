# Hand test 0.35.0: playback speed (experimental)

Off by default. This changes the live playback engine, so test with care and report anything odd.

1. Settings > Playback and Recording: turn on Playback speed. Transport > Playback Speed now has choices. Off: they are unavailable.
2. Play a session with a voice track at Normal. Nothing changes from 0.34 (same sound, meters, position).
3. While playing, choose 1.5x. Playback restarts from where it was; the voice is faster and the pitch is the same. Try 0.75x, 1.25x, 0.5x and 2x. 0.5x and 2x sound smeared: that is expected, but report if it is worse than smeared.
4. At a non-Normal speed: the playhead, the time display and the meters keep in step with what you hear. Click in the ruler and play again; Pause and Space work.
5. Loop Playback and Skip Selection at 1.5x: the loop and the skip land where the selection is.
6. With the metronome on: it is silent while the speed is not Normal and clicks again at Normal.
7. A big session: play at 0.75x and 1.5x. Report any dropouts or crackle (Help > Report a Problem, with the number of tracks and effects).
8. A session with buses, sends or automation at 1.5x: faders move the sound after about half a second.
9. Open a Waveform file and repeat steps 3 to 5.
10. Press Record at 1.5x: speed goes back to Normal first and recording is normal.
11. Turn the setting off while at 1.5x: speed goes back to Normal. Restart the program: speed starts at Normal.
12. A 3D Space session on speakers plays at Normal whatever the choice.
