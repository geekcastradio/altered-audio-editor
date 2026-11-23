# Hand test for 0.67.0

Things to try on Windows. Each line says what to do and what should happen.

## Tool strip
1. Open a session. The tool buttons (Move, Razor, Slip, Time Selection and the selection tools) sit on the left, right after the Waveform and Multitrack switch and the two small icons.
2. The three theme icons (sun, monitor, moon) sit alone at the right end.
3. Pick each theme: the tool buttons do not move.

## Light Mode
1. Click the sun with a session open. The timeline lanes are a soft gray, not white. The track headers are darker than the page.
2. Clip names are dark and easy to read on every clip. The waveform is dark against the tinted clip body.
3. Click a clip: it gets a blue wash and a dark outline you can see at a glance. Select several clips to check.
4. The envelope lines (volume, pan) on a clip are dark enough to see.
5. Blue text (the track volume numbers, the selected Track Effects tab, level readouts) is readable on the gray buttons.
6. Open the Waveform editor in Light Mode: the waveform and ruler are readable.
7. Click the moon: Dark Mode looks as it did in 0.66.0.

## Switching themes
1. With a session open and a few tracks showing, click the sun, then the moon, then the monitor, several times. The lanes, overview strip, ruler and selection highlights change at the same moment as the panels, with no visible lag.
2. Do the same in the Waveform editor and in the Spectrum view.
3. Change the Windows light or dark setting while Sync with System is on: the whole window, timeline included, follows.

## Shortcut import (no change you can see)
1. The import button in Edit > Keyboard Shortcuts still reads a shortcut export, lists commands it could not match, and does not give one key to two actions.
