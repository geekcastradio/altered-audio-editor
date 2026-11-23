# Hand test for 0.66.0

Things to try on Windows. Each line says what to do and what should happen.

## Light Mode and the theme switcher
1. At the right of the tool strip there are three icons: sun, monitor, moon. Hover each: a description appears.
2. Click the sun. The whole window turns light at once, including the waveform lanes, the meters and open panels. Close and reopen the app: it is still light.
3. Click the moon: dark again. View > Theme lists the same three choices and shows which is on.
4. Click the monitor. Change the Windows light or dark setting: the app follows without a restart.
5. Settings > Appearance shows the same choice.

## Matching window headers
1. Open Settings, Keyboard Shortcuts, Controllers and Plug-in Manager. Each has the same title bar, in both looks.
2. Open an effect window from the Voice Chain (double-click a slot). The title bar matches. The power, star, help and close buttons are readable in both looks.
3. Open a Piano Roll (Music Mode on, double-click a MIDI clip): the title bar matches, with the controls in a row under it.
4. In the Waveform editor, open a color popover (Settings > Appearance colors or the waveform color control): it has a small title bar.

## Level meter range (issue 269)
1. Settings > Meter range lists 24, 48, 60, 72, 96 and 120 dB. Pick 96 and play a quiet file: the Levels panel and the Voice Chain meters show the lower range, and the tick labels change.
2. Right-click the Levels panel: the same choices are in the menu, with the current one ticked. Pick 24: the scale shortens.
3. Close and reopen the app: the range is kept. Reset to defaults in Settings sets it back to 60.
4. The yellow and red marks still sit at their dB values (Settings > meter levels) at every range.

## Panels fold to their title
1. Drag the divider between Voice Chain and Properties all the way down. Properties shows only its tab strip (Properties, Takes). Drag it back up: the content returns.
2. Do the same for Files and for Libraries and Audio FX on the right. Each can fold to its title.
3. The Editor, Mixer, Levels and Time Info still stop at a usable size.
4. Window > Reset to Default Layout puts everything back.

## AAC and M4A on Linux and macOS (only if you have a tester)
1. Export a short session as M4A on Linux or macOS. The file plays in the system player and shows the right length.
2. On Windows nothing changes: it still uses the system encoder.

## Regression checks
1. Open a real .sesx, save it as .altaudio, close, reopen: layout, libraries and clip positions are intact.
2. Record a few seconds, play it back, undo and redo an edit.
3. Settings, Edit > Keyboard Shortcuts: change and reset one shortcut.
