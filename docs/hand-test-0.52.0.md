# Hand test: 0.52.0

- Start the app. Then Help > Copy Bug Report Details and paste it into a note. After the audio lines there is a "Startup timing:" list with lines like "process started" and "ui: first frame painted", and also "settings loaded", "libraries restored", "recovery checked". Send me that list. For the slow-start question, also time a cold start with a stopwatch, from the double-click to a window you can use, and say whether the wait is before the window appears or after.
- File > Export > Mixdown: the progress box is in the middle and finishes. Cancel one half way: it stops.
- File > Check Loudness and the analysis (Frequency Analysis, Amplitude Statistics): the progress shows and the result appears.
- Edit > Settings > Plug-ins: Scan. The progress counts up and the list fills. Stop a scan half way.
- Close the app and reopen it. Settings, shortcuts, libraries and the last layout are all as you left them.
- Open Edit > Settings > Media and Cache and File > Open Backup: both show the same folders as before.
- Optional, on your PC: download the `.exe` from the "Native spike" workflow run (Actions, Native spike, the latest run, Artifacts), try it, and send me `spike-timing.log` plus the frame time shown at the top while the waveform is zoomed out and in.
