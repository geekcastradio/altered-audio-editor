# Hand test: Mixdown Into Video (0.28.0)

Needs Windows (the AAC encoder is the system one) and an H.264 .mp4 or .mov.

1. Open the video (File > Open) so the session has a picture. Put a voice or music on a track.
2. File > Export > Mixdown Into Video... Pick a new .mp4 name. Progress shows, then a notice says the video was written.
3. Play the new file in a player. The picture is the original, the sound is your mix, and they start together. Check a clap or a hard cut near the start for sync.
4. A source video that had its own sound: the new file carries only your mix.
5. A video with B frames (most phone and camera files): the first frame still lines up with the first sound.
6. A .mkv or HEVC video: a clear message says only H.264 can be copied, and no file is left behind.
7. A session with no video: the menu item is dim and says why.
8. Cancel during export: no .mp4 and no leftover `.audio-tmp.m4a` in the folder.
9. Move the video later in the session (offset above zero): sound still starts with the first picture.
