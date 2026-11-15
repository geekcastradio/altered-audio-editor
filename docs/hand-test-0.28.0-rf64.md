# Hand test: RF64, BW64 and Wave64 files

You need two small test files. With ffmpeg on your PATH, from any short WAV:

    ffmpeg -i short.wav -rf64 always test-rf64.wav
    ffmpeg -i short.wav test-w64.w64

A field recorder's own long recording (a file over 4 GB) is the best test if you have one.

## Reading

1. Drag `test-rf64.wav` and `test-w64.w64` into the Files panel. Both show a length, sample rate and channels. The Format column says RF64 and W64.
2. Add each to a track and play. Scrub and jump around: playback starts at the right place and nothing clicks.
3. Mix down a session that holds both files to WAV and compare by ear with the originals. They should match.
4. A recorder's BW64 or RF64 file over 4 GB: it imports, the length matches the recorder's own, and playback near the end of the file works.

## Writing

5. Export a mixdown as WAV (16, 24 and 32-bit float, mono and stereo) and open each file in another player. Nothing sounds different from 0.27.0, and the files play everywhere.
6. Export a 3D speaker layout mixdown (12 channels). It plays in another editor with the channel layout intact.
7. A mixdown longer than 4 GB (about 8 hours of stereo 24-bit, or 30 minutes of 12 channels) is written as RF64 and opens in other software. If you do not have the patience, trust the automated tests: they write the header for a 6 GB file and read it back.

## Known limits

- Wave64 is read but not written. RF64 is written automatically when a file passes 4 GB.
- Chapter markers cannot be written into RF64 files yet (the app says so when you try).
