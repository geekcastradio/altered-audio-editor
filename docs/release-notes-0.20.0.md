## Altered Audio Editor 0.20.0

Mark the good and the bad as you record, review the marks in their own panel, set In and Out points from the keyboard, hear the audio while you scrub, and hide the on-canvas level control when you do not want it.

### Marking while you record
- **Two new quick marks beside Retake.** Add Cut This Marker (Shift+F9) and Add Great Bit Marker (Ctrl+F9) drop a numbered marker at the playhead, in the Multitrack and in the Waveform editor, the same way Add Retake Marker (F9) does. Every key can be changed in Keyboard Shortcuts.
- **A Review panel.** Window > Review lists your Cut this, Retake and Great bit marks in time order. Previous and Next step through them from the playhead, a filter shows one kind, and clicking a row jumps there.

### Scrubbing and the Waveform editor
- **Play audio while scrubbing.** Turn it on in Settings > Playback and Recording and drag the playhead along the ruler to hear it, in the Multitrack and the Waveform editor. It is approximate: playback restarts at the pointer every moment rather than following it smoothly. It is off by default.
- **Show or hide the HUD.** View > Show HUD (Shift+U) turns the on-canvas level control in the Waveform editor on and off.
- **A marker menu on the Waveform editor's ruler.** Right-click a marker's flag for Go to Marker, Show in Markers Panel, Rename, Convert to Range or Point, Change Marker Type and Delete. Clicking a flag picks that marker in the Markers panel.

### Editing
- **In and Out points from the keyboard.** I sets the start of the time selection at the playhead, O sets the end, Shift+I clears the start and Shift+O clears the end, in the Multitrack and in the Waveform editor. With no Out set the selection runs to the end of the content, and with no In it starts at zero. They are also in Edit > Select. Every key can be changed in Keyboard Shortcuts.
- **Repeat Last Command repeats Amplify with the same amount.** Edit > Repeat Last Command Dialog (Ctrl+R) opens the last command's dialog again, with your last choice filled in. Amplify now starts from the amount you used last time.
