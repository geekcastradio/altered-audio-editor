# Altered Audio Editor 0.10.0

Automation modes, the way you know them from Audition.

- **Write, Latch and Touch.** The Automation box in a track's header (and now in a bus's header too) offers Off, Read, Write, Latch and Touch. While playing, moves of the fader and pan (Mixer, track header or bus header) are recorded into the track's lanes. Write records from the moment you press Play, Latch from your first touch until you stop, Touch only while you hold the control, after which the lane takes over again. Everything recorded is one undo step ("Record Automation") when playback stops, and the controls return to where they were.
- **You hear it as you move it.** The control wins over the lane while you record. A point lands at the playhead, which is where you hear it.
- **Fewer points.** Recorded moves are thinned so a lane is not flooded with points. Settings > Playback and Recording > "Recorded automation: thin points" sets how much (zero keeps everything).
- **Effect settings too.** The sliders in the Voice Chain record like the fader does, into that effect's lane (made for you if it does not exist), for tracks and buses. Plug-in (VST3) controls have no lanes yet, so they are not recorded.
- **Output: None.** A track or bus can be set to play to nothing (the output picker, the track menu Output To, and the Mixer). It still meters and its sends still work, but it is left out of the mix and exports. Hardware outputs by channel come in 0.11.0.
- **User Guide.** Help > User Guide opens a guide inside the program (works offline, searchable). The first time the program runs, a short welcome points to it. People who already had settings saved are not shown the welcome.
- Existing sessions open as before.
