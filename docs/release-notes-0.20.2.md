## Altered Audio Editor 0.20.2

Recording fixes, reported on a Mac with a USB interface.

### Recording
- **Record starts right away.** The armed inputs (and input monitoring) now stay open when you press Record and again when you press Stop, instead of being closed and reopened each time. That reopen was slow on some audio interfaces and could clip the first moments of a take or delay the start by a second or two. If the setup changed since the inputs were opened, Record falls back to opening them fresh, as before.
- **The playhead stays at the end of the take.** After you stop a multitrack recording, the playhead rests where the recording ended instead of jumping back to where it started, even with Return to Start on Stop turned on. In the Waveform editor the new take stays selected with the playhead at its end.
