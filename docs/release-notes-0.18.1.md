## Altered Audio Editor 0.18.1

Sessions saved by other editors open with their markers and at a sensible track height, dropped files get their own tracks, and the playhead keeps following when you drag it past the edge.

### Sessions
- **Markers stored in a session's own metadata now come across.** Sessions saved by other editors keep their markers in a metadata block rather than in a plain marker list. Both kinds are read now, and the import notice says how many came across.
- **Sessions saved with short tracks no longer open squashed.** Track heights under the standard height open at the standard height.

### Opening files
- **Dropping several files gives each its own track by default.** If you prefer them one after another on a single track, that is still a choice in Settings, General.

### Editing
- **Dragging the playhead along the ruler scrolls the timeline.** Hold it at or past the edge of the lanes and the view follows, so you can scrub through a long session in one drag.

### Files panel
- **Closing a session from the Files panel takes its files out of the panel too.** Files another open session still uses stay.
