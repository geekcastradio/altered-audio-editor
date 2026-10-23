# Release 0.1.9

Not run on real hardware yet. Test before publishing. Contents (bump with `npm run bump -- 0.1.9`):

## Changes

- Space bar: plays the multitrack (or the open Waveform file); stops when playing or recording. The Pause/Stop setting is gone.
- Shortcuts from the approved table are assigned: Home, End, Ctrl+Left and Right (previous or next marker, clip or selection edge), Ctrl+Alt+Left and Right (markers only), M or * adds a cue marker, = and - zoom, backslash zooms to the session, 1 switches editors (0 Multitrack, 9 Waveform), Shift+D spectrum, Alt+Up and Down move a clip to the track above or below. Shortcuts for features that do not exist yet are listed as disabled menu items.
- Timeline: dragging the ruler slides the timeline (a click places the playhead); middle mouse drag slides the view and the tracks; dragging a clip near the edge scrolls the timeline with it; ruler labels are finer at every zoom; the timeline zooms out to 4 hours, or further only for longer content.
- Alt+wheel over the lanes makes the track taller or shorter (Alt+Shift+wheel zooms the waveform height).
- Clicking the lanes while playing no longer moves the playhead; only the ruler does. The ruler marker lines up with the playhead line. The bottom scrollbar is gone (the overview strip does it).
- Clips: waveform drawn much brighter than the clip body; fade handles sit under the title bar and are bigger.
- QuickPass: a Fix by menu on the Scan tab; Strip leaves the gap open by default so tracks stay in step ("Close gaps after fixing" is an option in Settings). Columns fit the panel.
- Panels: no float or maximize buttons; the panel menu (dots on the active tab, or right-click) has Close Panel, Undock Panel, Close Other Panels in Group, Panel Group items and Small Tabs. Left panels no longer scroll sideways. The Mixing workspace keeps Editor as its first tab.
- Voice Chain presets: Godlike, Demonic, Evil Voice, Ethereal, Angelic (starting points, tune by ear; reverb stage arrives with the reverb effect).
- Round 5: ASIO playback falls back to the system output; device lists keep their last good result; wheel rules; start screen for an empty session.

Not in 0.1.9 (see docs/backlog.md): fade bending, Libraries groupings, track visibility, Voice Chain row layout, effect editor windows with presets, new effects (reverb, delay, modulation, stereo), clip menu audit.

## Checklist

- [ ] `npm run bump -- 0.1.9`, then `npm run check`, `npm test`, `cd src-tauri; cargo fmt --check; cargo clippy -- -D warnings; cargo test`
- [ ] Space starts and stops playback in both editors and stops a recording
- [ ] Home, End, Ctrl+Left, Ctrl+Right, M, 1, 0, 9 do what the notes say
- [ ] Drag the ruler: view slides, no selection; click the ruler: playhead moves
- [ ] While playing, click in the lanes: playback keeps going
- [ ] Middle mouse drag slides the view; no autoscroll circle appears
- [ ] Drag a clip to the right edge: the timeline scrolls and the clip follows
- [ ] Alt+wheel over a track resizes it; Alt+Shift+wheel changes waveform height
- [ ] Fade handles are under the title bar; drag them
- [ ] Waveforms stand out from the clip body
- [ ] QuickPass: Fix by Strip leaves gaps; other tracks do not shift
- [ ] Panel tabs: no float or maximize buttons; the dots menu works; Small Tabs toggles
- [ ] Voice Chain: pick Godlike, Demonic, Evil Voice, Ethereal, Angelic on a track
- [ ] CI green on all platforms
- [ ] Tag, publish the draft, mark pre-release

## Draft release notes

**New**: Space bar plays and stops; the approved shortcut set; drag the ruler or the middle mouse button to slide the timeline; clips drag past the screen edge; finer ruler; Alt+wheel track height; QuickPass Fix by menu that leaves gaps open; simpler panel tabs with a panel menu; five character voice chains.
**Fixes**: playhead no longer jumps when you click while playing; ruler marker lines up with the playhead; panels no longer scroll sideways; clearer waveforms and fade handles; Settings > Devices and ASIO playback fallback.
**Known issues**: builds are unsigned (Windows) or ad hoc signed (macOS); audio, ASIO, macOS and Linux behavior is lightly tested on real hardware.
