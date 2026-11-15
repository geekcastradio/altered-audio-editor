# Hand test 0.33.0: buffer size (experimental)

Settings > Audio Devices > Buffer size asks the audio driver for a buffer size. It is a request: drivers may ignore it.

1. Leave it on Driver default. Play and record as usual: nothing should change from 0.32.
2. Choose 256 frames, press OK, then play. Sound plays. Record a short take on an armed track: it records and plays back.
3. Try 64 and 2048. A device may keep its own size, which is fine. Note whether the delay when monitoring a mic changes, and whether 64 crackles.
4. With ASIO: set the buffer in the interface's own control panel first, then try the setting here. Report which one wins.
5. If a size stops playback or recording, an error appears that ends with how to set it back. Set Driver default and confirm sound returns.
6. Close and reopen the program: the choice is remembered.
7. Report device, driver (system or ASIO), the sizes tried and what happened through Help > Report a Problem.
