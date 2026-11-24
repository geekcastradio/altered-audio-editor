# Hand test for 0.68.0

Things to try on Windows. Each line says what to do and what should happen.

## Skipped effects are kept (.sesx)
1. Open one of your real `.sesx` sessions. The import notice lists the skipped effects by name and says their original settings are kept.
2. Save it as `.altaudio`, close it, reopen it. The notice is gone, the tracks and clips are as before.
3. Choose File > Export > Multitrack Session and save a copy. Open the copy in your other editor (or open it here again): the effects that were skipped are in the same tracks and slots, switched on or off as they were.
4. Open a plain session with no skipped effects and export it: nothing about the export changes.

## FLAC compression
1. Open the export dialog, choose FLAC. A Compression list from 0 to 8 shows, with 5 marked default.
2. Export the same voice file at 0, 5 and 8. The files get smaller as the level rises (the gap on speech is modest) and take longer to write.
3. Play all three: they sound identical. Open them in another player to be sure they all play.
4. Open Batch Convert > Output and choose FLAC: the same Compression choice shows. Run a small batch at level 8 and play one result.
5. Quit and restart the app: the level you picked is remembered with the other export settings.

## Import labels
1. Edit > Keyboard Shortcuts: the import button reads "Import shortcuts...".
2. Voice Chain > Presets: the item reads "Import chains from a .xml file...". Pick a file with no chains in it: the notice describes the file without naming another product.
