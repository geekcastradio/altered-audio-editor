# Hand test for 0.69.0

Things to try on Windows. Each line says what to do and what should happen.

## Audacity projects
1. File > Import Project. Pick a `.aup3` file. A new, unsaved session opens with the same tracks and clips, and a notice lists anything left out.
2. Play it. A linked stereo pair is one stereo track and sounds right. Gain, pan, mute and solo match the original.
3. A folder named "<project> Imported Audio" now sits beside the project file with the WAV files. The original file is unchanged (check its date).
4. Repeat with a `.aup4` file. Track effects show in the notice as left out.
5. Save the session as `.altaudio`, close it, reopen it. Everything is as before.
6. Choose Open and pick a `.aup3` file: it imports the same way.
7. Try a file that is not a project (rename a text file to `.aup3`): a clear error, no crash.

## Workspaces and palette
1. Window > Workspace: Production Desk, Loudness, Video Edit and Broadcast are listed. Open each. Panels fill the window with no overlap. Reset one and it returns to its layout.
2. Switch back to Default: it is as before.
3. Open a Spectrum view: the colors are the Roseus palette. In Spectrogram colors, pick Classic: the old colors return.

## Markers and QuickPass
1. Add markers far apart, zoom in, scroll to the start. Click a marker far to the right in the Markers list: the Editor scrolls to it.
2. Add a range marker. Click it: the whole range is selected. Press Loop Playback and it loops that range.
3. Right-click a marker and choose Go To: the same happens.
4. Run a QuickPass scan that finds results. Click a finding off screen: the view scrolls to it.
5. Right-click a finding and choose Strip Selected. The row shows Removed. Press Ctrl+Z: the row is back, active, and you can click it again.

## Waveform editor
1. Zoom in on part of a file and double-click: only what is in view is selected.
2. Make a selection and choose Zoom to Selection: it fills the view edge to edge.
3. Play it: the view stays put until the end.
4. Zoom slowly into a file with the wheel: the detail gets finer smoothly, with no jump.
5. Toggle Follow in the toolbar. With it off, playback leaves the view alone; with it on, the view follows.

## Save Selection As
1. Open a stereo file in the Waveform editor, select a few seconds, and choose File > Save Selection As (or press Ctrl+Alt+S). A dialog shows the format, sample type and settings, and the length of the selection.
2. Save as WAV. A new file with only that part is in the folder. Play it: it matches the selection. The open file and its selection are unchanged.
3. Repeat as MP3 and FLAC with other settings (bit rate, compression level). Each plays.
4. Put a marker inside the selection and tick the cue points option for WAV. Open the new file: the marker is there at the right spot.
5. Try to save over the open file's name: you are told to pick another name.
6. With no selection, the menu item is greyed out with a reason.

## Track menu
1. Select a track with a stereo clip. Right-click > Track > Swap Stereo Channels. Play a clip with a known left and right: they are swapped. Ctrl+Z puts them back.
2. Split Stereo to Center Mono: both speakers play the same mix. Ctrl+Z puts it back.
3. Select a mono track: both items are greyed out.

## Dialogs and panels
1. Open Settings, Keyboard Shortcuts, Controllers, Plug-in Manager and Analysis: each has a title bar with a close X that closes it.
2. Drag the Files panel down to a thin strip, then back. Same for Libraries.
