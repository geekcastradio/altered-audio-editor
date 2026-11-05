# Altered Audio Editor 0.16.0

Faster marker work, a stronger Loudness Match, music that ducks from track roles, and a track view with room to read.

## Mixing

- **Duck Music Under Voice reads Mix Role.** Right-click a track header, choose Mix Role, and mark your voices Foreground and your music Ducked. With nothing selected, Duck Music Under Voice ducks every Ducked track under the Foreground tracks, in one undo step. Select a track that is not marked Ducked and it ducks just that one, as before. Sessions with no roles set behave exactly as they did. Background is still only a label for now.
- **Loudness Match, more ways to match.** Match by Loudness (LUFS), Peak, True Peak or Total RMS. A new Tolerance box (0.5 dB to start) leaves alone anything already that close, so a clip that is close enough is not touched. Set it to 0 for the old behavior. You can also right-click a clip and choose Loudness Match.
- **QuickPass is in the Effects menu.** It was only a panel before. Its Remove Silence scan is the fast way to tighten an episode: it can cut silences out and close the gap, split at each end, turn them down, or mark them.

## Markers

- **Pick markers in the Markers panel.** Click one to select it and jump to it, Ctrl+click to add or remove, Shift+click to pick a run.
- **New commands under Edit > Markers.** Rename Selected Marker, Delete Selected Markers and Delete All Markers. Each is one undo step.

## Track view

- **New tracks open expanded**, with all five header rows showing. Tracks you already have keep the height you saved. Zoom Reset (All Axes) sets every track to the new height.
- **Sync Lock and Mono moved** beside Invert Polarity on the Mix row, so the track name gets the room.
- **Link Media on offline clips.** When a clip's file has gone missing, right-click it and choose Link Media to point it at the file's new home.

## Under the hood

- More tests around saving a session made by an older version, so the backup-then-save path is checked on every build.
