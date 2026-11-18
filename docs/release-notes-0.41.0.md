## Altered Audio Editor 0.41.0

### Keyboard (Audition keys)
- Ctrl+1 to Ctrl+5 pick the current clipboard, as in Audition. Edit > Set Current Clipboard shows the same choice.
- A turns Follow Playhead on and off. New: View > Follow Playhead, with a tick that matches the toolbar button.
- Alt+Shift+comma and Alt+Shift+period nudge the selected clips five steps left or right. New: Clip > Nudge > Nudge Left 5 Steps and Nudge Right 5 Steps.
- All of these import from an Audition shortcut export.

### Build and release
- The Linux build and the release's Linux installer job no longer stop on a slow package mirror: the install is tried up to three times, with a different mirror on the retries.

### Not in this release
- J (play backward). Playing a whole mix backward needs engine work that is not safe to ship untested; the options are written up in the project notes.
