# Hand test 0.38.0, part 3 (menu completion and guide)

This release finishes the menu items that used to say "Not built yet" and adds guide sections. Test on a session with at least three tracks and a few clips.

## Track visibility presets
1. Hide two tracks (Multitrack > Track Visibility > Hide Selected Track).
2. Choose Save Visibility Preset, type a name, press OK. A notice should say it was saved.
3. Choose Show All Tracks, then Load Visibility Preset and pick the preset. The same two tracks should hide again, as one undo step.
4. Load Visibility Preset should be greyed out, with a reason, until you have saved one.
5. Save nine presets. Only the newest eight should remain.
6. Open another session with tracks of the same names and load a preset. It should match by name.

## Meter Input Signal (Alt+I)
1. Do not arm any track. Press Alt+I. The menu item should show a check, and every track with an input should show an input level when you speak.
2. Press Alt+I again. The meters should go back to playback levels.
3. Arm a track and start a recording with the toggle on. Recording should work as before.
4. Turn on monitoring for a track while it is not armed. You should not hear it until you arm the track.

## Groups
1. Select two clips on different tracks with Ctrl+click and choose Clip > Group > Group Clips.
2. Click one of them. Both should be selected.
3. Press Ctrl+Shift+G (Suspend Groups). A notice appears and the menu item shows a check. Click one clip. Only that clip should be selected. The group should still exist, shown by the group colour.
4. Press Ctrl+Shift+G again. Clicking one clip should select the whole group again.
5. Click one clip of a group, then choose Remove Focus Clip from Group. Only that clip should leave the group. Undo should put it back.
6. With nothing clicked yet, Remove Focus Clip from Group should be greyed out with a reason.

## Guest Track
1. Choose Multitrack > Add > Guest Track. A mono track named Guest should appear below the selected track.
2. Open its Voice Chain. It should hold the Remote Guest chain, and Multitrack > Host Profile should show Remote Guest.
3. Undo should remove the track and its chain in one step.
4. Add a second Guest Track. It should be named Guest 2 or similar, not a duplicate name.

## Podcast Mode and Voiceover Mode
1. View > Voiceover Mode should switch to the Voiceover workspace (takes and markers on the left) and show a check.
2. View > Podcast Mode should switch to the Default workspace.
3. Turn on Music Mode, then choose Podcast Mode. Music Mode should switch off and the metronome should stop.

## Menus and guide
1. Open each menu and check that nothing says "Not built yet". The only item that still explains why it is unavailable is the VST2 line in the Plug-ins menu.
2. Open Help > User Guide. Check the new sections: Track Visibility, Groups, Meter Input Signal, Host profiles and Guest Track, Panels and workspaces, Everyday tools.

Report anything wrong with the menu path, what you did and what happened.
