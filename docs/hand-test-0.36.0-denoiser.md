# Hand test 0.36.0: Speech Denoiser (High Quality)

1. Add Speech Denoiser (High Quality) from the Restoration group to a voice track that has background noise (a fan, keyboard, traffic). Compare it with Speech Denoiser on the same clip: the High Quality one should remove more of the hard noise and keep the voice natural. Report clips where it does worse.
2. Lower Amount to 50. Some of the original noise comes back.
3. Play a session with it on two tracks, then on six. Meters and sound should be steady; report dropouts and the number of tracks and your computer.
4. Line up a clip with another recording of the same moment (a double-ender). The effect holds audio back about 40 ms and the session makes up for it: the voice should stay in step with the other track. Report any drift.
5. Export the track with and without the effect: it is the same length and in step.
6. Process a file in the Waveform editor with it (Effects menu), and run it as a step in a Process chain.
7. Music or a laugh track through it thins the sound. That is expected, as with Speech Denoiser.
8. Speed: the first time it plays it may take a moment to load. Report if longer than a few seconds.
