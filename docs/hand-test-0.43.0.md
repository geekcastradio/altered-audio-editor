# Hand test 0.43.0: selection playback, Adjust Selection, update notice

## Playing a selection stops at its end (issue 126)
1. Waveform Editor: open a file, drag a short selection, press Space. Playback should stop at the end of the selection and the playhead should rest there (or return to the start if Return Playhead on Stop is on).
2. Click in the waveform outside the selection, press Space. Playback starts at the start of the selection and still stops at its end.
3. Multitrack: make a time selection over a few seconds, press Space. Same result.
4. Edit > Settings > Playback and Recording: untick "When playback starts within a selection, stop at the end of the selection". Press Space again: it now plays on past the selection.
5. Loop Playback still loops the selection, Skip Selection still skips it. Recording is unaffected.

## Adjust Selection (issue 125)
1. Make a selection. Edit > Select > Adjust Selection: Move Start Left, Move Start Right, Move End Left, Move End Right. Each moves that border by one pixel at the current zoom. Zoom in and the step gets finer.
2. The borders never cross: Move Start Right on a very short selection stops one sample before the end.
3. With no selection the four commands are greyed out with a reason.
4. Keys: Shift+H and Shift+J move the left side, Shift+K and Shift+L move the right side. Shift+Left and Shift+Right move the start, Ctrl+Shift+Left and Ctrl+Shift+Right move the end. Check all eight, in both the Waveform Editor and the multitrack view. They are listed in Edit > Keyboard Shortcuts and can be changed. Typing in a text box must not trigger them.
5. Shift+click still changes the borders with the mouse.
Not built: dragging the edge of a selection with the mouse. Tell me if you want it.

## Update notice (Samfox's request)
1. Edit > Settings > Privacy: "Tell me when a new version is out" is off by default. Turn it on: pick "Every time the program starts" or "At most once a week". The two choices are greyed out while the box is off.
2. With the box on and a newer release on GitHub, restart the program: a small notice appears bottom right: "Version X is available" with See what's new and Later. See what's new opens the usual update dialog. Later closes it.
3. It does not appear while audio is playing at launch, and it never installs anything by itself.
4. With the box off, the program makes no web request at start. Help > Check for Updates still works.
5. Tick "Install it for me when the program starts" (installed copy only) and restart while a newer release exists: it should show "Installing version X and restarting", ask about unsaved work if there is any, download, install and restart. Decline the unsaved-work question: the corner notice appears instead. On the portable zip or the Linux package it just shows the notice.

## Stop at End of Selection key
1. Transport > Stop at End of Selection has a tick that matches Settings. Shift+V toggles it, in both views.

## Rust steps (nothing should look different)
1. Export a mixdown, run a batch, and open a file by dragging it onto the program. Progress bars and opening behave exactly as in 0.42.0.
2. Run the checks: `npm test` and `cargo test` both pass. `shared-fixtures/selection.json` is read by both.
