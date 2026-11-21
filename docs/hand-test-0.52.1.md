# Hand test: 0.52.1

- Before you publish: open the draft release on GitHub. The release text must NOT start with a "No ASIO in this build" note. If it does, ASIO failed again; do not publish, tell me.
- Install it (Windows). Edit > Settings > Audio: the ASIO option is available, not greyed out. Pick it and see whether your interfaces' ASIO drivers are listed.
- With an ASIO interface, if you have one: choose its ASIO driver, play a clip and record a short take. Report any crackle, a wrong channel list or a driver that will not open, with the buffer size you used.
- Help > User Guide: the contents list shows "Multi-mic shows" on one line and does not touch "Recording a remote guest". Searching the guide for "double-ender" finds that section.
- Start the app, then Help > Copy Bug Report Details: the "Startup timing:" list is there and complete.
- Open a session, play it, edit a clip, save, close and reopen: nothing different from 0.52.0.
- Still wanted from 0.52.0: the startup timing list from a cold start, plus a stopwatch time and whether the wait is before or after the window appears.
