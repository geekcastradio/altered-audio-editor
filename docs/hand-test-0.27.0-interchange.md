# Hand test: Export Edit Lists (0.27.0)

1. Open a session with two or more tracks and a few clips, some with fades and one muted. Add two markers.
2. File > Export > Edit Lists... The dialog opens with the format focused. Esc closes it.
3. EDL: pick a folder. One .edl per track appears, named "Session - Track". Open one in a text editor: events are numbered, record times match the clip positions, "Starts at" shifts the record times.
4. Run it again into the same folder. The first files stay, the new ones get " 2" in the name.
5. Markers as an EDL: with no markers the dialog says so and the button is off. With markers, import the file in DaVinci Resolve (File > Import > Timeline Markers from EDL). Report whether markers land at the right times and in the chosen color.
6. OpenTimelineIO: open the .otio in any OTIO viewer or editor that reads it. Tracks, clip names and source files should match.
7. Use a stretched clip, a looped clip and a reversed clip: the result lists a warning for each. The muted clip is counted as left out.
8. Change the frame rate to 29.97 drop frame and enter 01:00:00;00. The dialog accepts it and the EDL says DROP FRAME.
9. Type 1:2:3 in "Starts at": the warning shows and the button is off.
10. Cancel the folder picker: the dialog stays open, nothing is written.
