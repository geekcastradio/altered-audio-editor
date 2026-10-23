# Release 0.1.10

A bug-fix release for 0.1.9, plus the markers and shortcuts that came out of the same tester round. Not run on real hardware yet. Bump with `npm run bump -- 0.1.10`.

## Changes

- Fixed: right-clicking anywhere in the app opened the web view's own menu (Back, Refresh, Print, Share, in the system language). It is blocked now, except in text fields where Cut, Copy and Paste still work.
- Fixed: right-click on empty space in the multitrack did nothing when no session was open. It now opens the session menu (items that need a session are disabled with a reason).
- Export Markers and Chapters now works in the Waveform editor, using the open file's markers. It saves beside the audio file by default.
- Right-click menu for empty multitrack space follows the standard layout: Ripple Delete, Select, Groups, Track, Insert, Marker, Mixdown, Export Mixdown, Bounce, Session Properties. Entries for features not built yet are disabled with a reason.
- New markers: Subclip, CD Track and Cart Timer (Edit > Markers, right-click menu, Markers panel, Waveform editor). The CD and broadcast tools that use them come later.
- Shortcuts: Alt+A, Shift+A, Shift+E, Ctrl+Alt+Backspace (tracks); Ctrl+G (group); Shift+Backspace and Ctrl+Shift+Backspace (ripple delete); Ctrl+Shift+A, Period, Ctrl+Alt+T, G (select); Shift+M (CD Track marker), C (Cart Timer marker). Alt+B, Ctrl+Shift+G and Alt+Backspace are held by items that are not built yet.

Not in 0.1.10 (see docs/backlog.md): CD layout and disc export, cart timer behavior, bus tracks, Suspend Groups, FCPXML export, MCP control.

## Checklist

- [ ] `npm run bump -- 0.1.10`, then `npm run check`, `npm test`, `cd src-tauri; cargo fmt --check; cargo clippy -- -D warnings; cargo test`
- [ ] Right-click anywhere: no browser menu; text fields still paste
- [ ] Right-click on the empty multitrack with nothing open: session menu appears
- [ ] Waveform editor with markers: File > Export > Markers and Chapters saves a list
- [ ] Add CD Track (Shift+M), Cart Timer (C) and Subclip markers; they show in the Markers panel; save and reopen a session
- [ ] Alt+A adds a stereo track; Shift+E deletes empty tracks; Ctrl+G groups two clips; Ctrl+Shift+A deselects
- [ ] CI green on all platforms; tag v0.1.10, publish the draft, mark pre-release
