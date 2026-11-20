## Altered Audio Editor 0.43.0

### Fixes
- Playing a selection now stops at its end when playback starts inside it, in both the Waveform Editor and the multitrack view, as in Audition. Edit > Settings > Playback and Recording has a switch to turn it off. (Issue 126, thanks Garretiko.)

### Selections
- New Edit > Select > Adjust Selection: Move Start Left, Move Start Right, Move End Left and Move End Right. They move one pixel at the current zoom. Shift+H and Shift+J move the left side, Shift+K and Shift+L move the right side, and Shift+Left, Shift+Right, Ctrl+Shift+Left and Ctrl+Shift+Right do the same for the start and end. Keys can be changed in Keyboard Shortcuts, and an imported shortcut export maps them. (Issue 125.)
- Transport > Stop at End of Selection (Shift+V) turns the stop-at-selection behavior on and off from the keyboard.

### Updates
- The optional update notice can now look every time the program starts, or at most once a week. When a newer version exists a small notice appears in the bottom right corner with a button to see what is new. It is off by default; with it off the program stays fully offline. A second tick, "Install it for me when the program starts", also off by default, installs the update at start-up for installed copies, after asking about unsaved work. Without it nothing installs without a click. (Thanks Samfox.)

### Under the hood
- The first steps of the move to a Rust interface. Progress and file events now go through one small Rust interface instead of calling the window directly, so a native window can take over later without touching the commands. No change in what you see.
- The selection and playhead rules (Adjust Selection, playhead keys, where Play starts, stop at the end of a selection) now also exist in Rust. Both copies are tested against one shared file of cases, so they cannot drift apart. The program still uses the current copy for now.
