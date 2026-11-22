# Hand test for 0.63.0

1. Settings > Privacy: turn on the update check and click Help > Check for Updates when a newer version exists (or have a tester's older build do it). The notice is a card with the version, the top changes, and three buttons. Later closes it. All changes opens the full dialog. Install and Restart asks about unsaved work first.
2. The portable zip (no self-update) shows Download in place of Install and Restart.
3. The card does not appear while playing or recording.
4. Export a mixdown (File > Export). The finished notice has Show in Explorer. Click it: Explorer opens with the file selected.
5. Export stems and a playlist: the button shows the last file written.
6. Settings > Effects: untick "Offer to show the file when an export finishes". Export again: no button.
7. Open the User Guide (Help), search for a word, open a few sections. Looks and reads as before.
8. Session Properties and Collect Files: change a sample rate, copy files into the session folder. Same results as 0.62.0.
9. Mixer: send a track to a bus, try to send that bus back. The loop is refused as before. Auto-duck music under a voice track and remove it.
10. Sync Tracks: sync two tracks, check the notice wording and that a locked clip refuses to move.
