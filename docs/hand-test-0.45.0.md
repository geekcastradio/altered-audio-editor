# Hand test 0.45.0: startup timing

1. Close the program completely. Start it from a cold start (right after boot is best) and wait until the window is usable.
2. Open `%APPDATA%\Altered Audio\startup-timing.log` in Notepad. It should list, in order: process started, window and core ready, plug-in list loaded, setup done, ui: main window mounted, ui: first frame painted. Each line has the milliseconds since the process began and the gap since the line before.
3. Note which gap is biggest. If the whole launch takes about a minute, that gap is what to fix. Send the file (or paste it) with your report.
4. Start the program again. The file is replaced by this launch's numbers, not added to.
5. Portable copy: the file is in `data\config` next to the program.
6. Nothing else should look or behave differently.
