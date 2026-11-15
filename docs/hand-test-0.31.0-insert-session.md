# Hand test 0.31.0: Insert Session as Clip and controller feedback

## Insert Session as Clip

1. Open a session that has some audio (an intro, a theme). Save it.
2. Start or open a second session and save it. Put the playhead somewhere.
3. Multitrack > Insert Session as Clip... and pick the first session.
4. A new track appears with one clip at the playhead named "<name> Mix". Play it: it sounds like the first session's whole mix, with its own volumes, effects and Master.
5. A folder "Inserted Sessions" now sits beside the second session, holding the mixed WAV.
6. Undo removes the clip. Insert the same session again: a second file with a new number, nothing overwritten.
7. A session whose audio files were moved: the notice says how many files were missing.

## Controller feedback (untested on real hardware)

This was built from the Mackie Control message layout and has not been tried on a real surface. If you have a Mackie-style controller with motorized faders:

8. Controllers dialog: tick the device, press Use Mackie Control Layout, tick Send feedback to the devices.
9. Move a track fader with the mouse: the matching motor fader follows. Mute, solo and arm buttons light with the track. Pan rings follow the pan.
10. Touch a motor fader while playing: it should not fight your hand.
11. Report what happened (device, mode, what worked) through Help > Report a Problem.
