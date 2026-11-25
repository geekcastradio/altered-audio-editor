# Hand test for 0.70.0

Things to try on Windows. Each line says what to do and what should happen.

## Favorites
1. Open the Favorites menu: it is empty. View > Quick Actions Bar is off and no bar shows in the tool strip.
2. Right-click Effects > Pitch and Time > Time Stretch. Choose Add to Favorites. It appears in the Favorites menu.
3. Turn on View > Quick Actions Bar. A bar shows in the tool strip gap with the pinned command. Turn it off: the bar is gone, not just empty.
4. Pin two more commands, one that only works in the Waveform editor and one for the Multitrack editor. Switch editors: the one that does not apply is greyed out.
5. Right-click a pinned button and choose Remove from Favorites.
6. Close the app and reopen it: the pinned list and the bar setting are as you left them.
7. Make the window narrow: the bar does not push the tool strip out of reach.

## Import and Open
1. File > Import: Files, Folder and Session are listed.
2. Session: the dialog's file type list has all session and project files, `.sesx`, `.rpp`, `.aup3` and `.aup4`, and `.otio`. Open one of each you have. It imports as before.
3. File > Open with a `.rpp` or `.aup3`: it imports the same way.
4. Pick a text file renamed to `.rpp`: a clear error, no crash.

## Waveform editor saving
1. Open a WAV, make an edit, choose Save Audio As. The dialog has file name, folder, format and the encoder settings. Save as FLAC at 24-bit: the new file plays and the original is unchanged.
2. Repeat as MP3 with a chosen bit rate and as AAC.
3. Edit > Settings > Autosave: tick "Ask before Save writes over the original audio file". Make an edit and press Ctrl+S: a question appears. Cancel: nothing is written. Press Ctrl+S again and confirm: the file saves.
4. Untick it: Ctrl+S saves with no question.
5. With it ticked, Save Audio As does not ask the extra question (it picks a new file).

## Markers and selection
1. In the Waveform editor, make a selection, then click a marker flag on the ruler. The selection is gone. Press Space: it plays from the marker.
2. Click a marker in the Markers list: same result. Click a range marker: the range is selected.

## Light theme
1. View > Theme: Light. Look at the Multitrack editor: the selected track's name is dark and easy to read, there is no pale blue block, and M, S and R buttons are readable when on.
2. Look at the right-hand track scrollbar: the color strips are softer.
3. Open a file in the Waveform editor in the light theme, make a selection: the selection and the level grid lines are visible.
4. Edit > Settings > Appearance: set a Selected track lane color. The selection color follows it, in both themes.
5. Switch to Dark: it looks as it did in 0.69.0.

## Multitrack
1. Right-click empty space in a lane. The groups are Paste; Ripple Delete; Select; Groups, Stretch and Fit to Length; Track and Track Visibility; Insert; Marker; Mixdown, Export Mixdown and Bounce; Clip Properties and Session Properties.
2. Stretch has Time Stretch, Stretch and Pitch and Remove Stretch. Fit to Length has Fit Music to Length. Clip Properties is greyed out with a reason.
3. In a new session, drag three audio files from File Explorer onto Track 1 together: each is on its own track, all starting at the same point.
4. Put a clip on Track 2 at the start, then drag three files onto Track 1: the files do not land on top of the clip on Track 2. They use the free tracks below, and new tracks are made when needed.
5. Repeat the drag from the Files panel with the files selected.
