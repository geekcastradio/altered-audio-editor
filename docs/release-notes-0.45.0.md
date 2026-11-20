## Altered Audio Editor 0.45.0

### Under the hood
- Startup timing. Each launch now writes `startup-timing.log` in the Altered Audio settings folder (`%APPDATA%\Altered Audio` on Windows), replacing the one from the last launch. It lists each stage of starting the program with the time since the process began: window and core ready, plug-in list loaded, setup done, main window mounted, first frame painted. The file stays on your computer and nothing is sent anywhere. It is there so a slow start can be read stage by stage. If you see a slow launch, send that file with your report.
