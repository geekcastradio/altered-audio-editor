# Hand test 0.40.0, part 2: video details, codec messages, hardware decoding

Use a 1080p H.264 MP4, and if you have them a 4K file and an HEVC file.

1. Open a video (File > Open). In the Video panel, a details line should appear under the file path: codec, size and frame rate, then "software decoding".
2. Play the session for a few seconds. The line should add "ms per picture (slowest ...)" and "N of M slower than one frame". Note the numbers for the 1080p file.
3. Tick "Try hardware decoding for smoother playback". The picture should reopen and keep working. Play again and compare the numbers. The line should say "hardware decoder in use" if the system accepted it, or stay on "software decoding" if it did not.
4. Look closely at the picture with hardware on: it should have the right colors, the right way up, and no black or green frames. If it is wrong, untick the box and tell me the GPU, the file's codec and what you saw.
5. Close the app and reopen it with a video in the session. The checkbox should keep its last setting and the video should open with it.
6. Scrub quickly along the Video Reference bar with hardware on and off. Note whether either one feels different.
7. Try an HEVC or AV1 file on a PC without the Store extension. You should get a message that Windows has no decoder and what to install, not a raw system error.
8. Try a file that is not a video renamed to .mp4. You should get a readable message (it may be the "can't read this kind of file" one).
9. Open Help > User Guide, Video: the new paragraph about the details line and the checkbox should be there.

Report the numbers from step 2 and 3 with your CPU and GPU. They tell me whether hardware decoding is worth turning on by default.
