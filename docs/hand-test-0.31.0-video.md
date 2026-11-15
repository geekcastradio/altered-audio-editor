# Hand test 0.31.0: video fixes

Use a video with a picture (an .mp4 from a phone or a screen capture works).

1. In the Files panel, double-click the video. The Video panel opens and shows a picture, and the Waveform editor opens on its sound.
2. Drag a video from Explorer onto the Files panel. It is added to the Files list and the Video panel shows its picture.
3. File > Open on a video still opens the Video panel (and puts the sound on Track 1 when the session is empty).
4. Switch to the Multitrack. A "Video Reference" bar sits under the ruler with a film icon and the file name, as long as the video. Zoom in and out: the bar follows.
5. Drag the bar left or right. The Video panel's "Starts at (s)" changes to match, and Undo puts it back.
6. Play with the playhead over the picture. The picture follows the playhead.
7. Waveform editor: play from the middle of a file, press Stop. The cursor stays where playback stopped. Multitrack: Stop still goes back to where playback started.
8. If the Video panel shows a red line, write down its exact words and send them.

## Session tabs

9. Open two sessions. A tab bar appears along the top of the Multitrack, one tab per session, the front one marked. With one session open there is no bar.
10. Click the other tab: that session comes forward. A session with unsaved changes shows a star.
11. Click the x on a tab. It asks about unsaved changes like closing from the Files panel, then closes.

## MSI installer (after the release is built)

12. The DRAFT release has Altered-Audio-Editor-v0.31.0-x64.msi next to the setup.exe. If it is missing, the optional MSI step failed in CI; the release itself is unaffected. Install the MSI on a spare Windows machine or VM and check that it starts and uninstalls from Settings > Apps.
