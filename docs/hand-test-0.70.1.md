# Hand test for 0.70.1

Things to try on Windows. Each line says what to do and what should happen.

## Dropping files
1. Drag two FLAC files from Explorer into the Files panel. Both show Ready and are copied into the session folder.
2. Drop the same two files again straight away, before the first finishes if you can. No red error, no freeze that lasts, and no "(2)" copies in the Files panel.
3. Drop them a third time after a pause: they are not listed twice.
4. Drag one of them onto a track twice in a row: two clips, one file in the Files panel.
5. Select several tracks and drag a clip while a drop is going on. If a blue box stays behind on the lanes after you let go, note what you did and tell us.

## Audition 3 `.ses` import
1. File > Open and pick an Audition 3 `.ses` file. Its tracks, names and clips show, and clips sit on the tracks they were on.
2. Open it again through File > Import > Session and by dragging it into the window: same result.
3. Try an old Cool Edit `.ses`: a clear message that the layout is not read yet, no crash.
4. Try a text file renamed to `.ses`: a clear error, no crash.

## Music Mode and the status bar
1. Window > Workspace > Music: Music Mode turns on. Switch to Default: View > Music Mode is unticked and the metronome track is gone.
2. Start the app: no red ResizeObserver message in the status bar.

## Favorites
1. Right-click a menu item and choose Add to Favorites: the menu closes and no box is left hanging.
2. Right-click a Quick Actions Bar button: same.

## Convert Sample Type
1. Open a WAV in the Waveform editor. Edit > Convert Sample Type, pick a new rate and type. A notice says which file was saved ("name (converted).wav").
2. Do the same again: it saves "name (converted) 2.wav" instead of failing.
3. Open the new file: its rate and type are the ones you picked.

## Menus
1. Edit: Undo, Redo, Repeat Last Command together. Tool, the clipboard block, Delete, Ripple Delete and Crop, then Select, Markers, Zero Crossings, Snapping and Find. Keyboard Shortcuts, Controllers and Settings last.
2. Multitrack: Tracks and Playback and Recording are submenus. Add Track is inside Tracks. Mix Down and Bounce sit together near the bottom.
3. Clip: Open in Waveform Editor is first. Time Stretch, Stretch and Pitch, Fit Music to Length and Remove Stretch are under Stretch.
4. View: the zoom items are under Zoom. Activate Left and Right Channel are under Waveform Channels.
5. Window: panels are in groups with separators: left column, then Editor, Mixer and Levels, then the right column, then the rest by name.
6. Try a few shortcuts from the moved items (Ctrl+K, zoom in and out, Space, Shift+Space): they still work.
