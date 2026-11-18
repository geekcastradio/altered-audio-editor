# Altered Audio Editor 0.39.0 (draft, grows with each part)

Part 1 brings the keyboard in line with Audition. Where we and Audition used the same key for different things, Audition's meaning now wins and ours moved to a new key.

## What's new

- **Audition's panel keys.** Alt+1 Editor, Alt+2 Mixer, Alt+3 Properties, Alt+5 Loudness Match, Alt+6 Time Info, Alt+7 Levels, Alt+8 Markers, Alt+9 Files, Ctrl+P Metadata and Alt+X Phase Wheel.
- **Audition's function keys.** F1 Help, F2 and F3 repeat the last command, F4 Settings, F9 spectral view, F10 meter input, F11 Convert Sample Type, F12 swap editors.
- **More marker keys.** Ctrl+0 delete selected markers, Ctrl+Alt+0 delete all markers, "/" rename, Shift+F8 CD track marker, Ctrl+Shift+F8 range marker.
- **More keys.** Ctrl+Shift+A deselect all, Ctrl+Shift+K settings, Alt+Z Frequency Analysis, Ctrl+M minimize, "+" zoom in.
- **Playhead keys.** Left and Right move one pixel, Page Up and Page Down one screen, Q and W go to the start and end of the time selection, and Ctrl+J asks for a time to jump to. They work in the Multitrack and the Waveform Editor, and a focused slider, drop-down or list keeps its own arrow keys.
- **Fill Single Click rebuilt.** It now accepts selections up to 100 ms, so two nearby clicks can go in one selection. The fill is built from the sound on both sides and matched to the room level around the click, which removes the arch that the first version could leave. The same selection always gives the same result.
- **Audition shortcut import** now recognises the panel, marker, help and playhead commands too.

## Keys that moved

- Retake, Cut This and Great Bit markers: F6, Shift+F6, Ctrl+F6 (were F9 variants).
- Fill Single Click: Ctrl+Alt+U (was Ctrl+Alt+K).
- Voice Chain Alt+0, Audio FX Ctrl+Alt+1, Libraries Ctrl+Alt+2, History Ctrl+Alt+3.
- Zoom out is "-" only (Ctrl+- is free for Audition's Move Track).
