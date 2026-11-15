# Altered Audio Editor 0.34.0

Three small additions: play just a selected area of the spectrum, a place in Settings that says where scripting stands, and a soundboard.

## What's new

- **Play Selected Area.** In the Spectrum view, draw a selection and choose Effects > Spectrum > Play Selected Area. You hear only the sound inside the selection, without changing the file. Selections up to five minutes long.
- **Soundboard.** A new panel (View > Panels) with nine pads. Press a pad and its sound goes on the session at the playhead, on the selected track, as one undoable step. An empty pad asks you to choose a file. Each pad is also an action, Multitrack > Soundboard > Pad 1 to Pad 9, so you can give it a key in Edit > Keyboard Shortcuts. Pads are saved with the program, not with a session.
- **User Guide sidebar.** Long section titles, such as Multi-mic shows and double-enders, now wrap cleanly instead of overlapping the next title.
- **Settings > Automation.** A page that says what is available now (process chains, under Multitrack > Run Process Chain) and what is planned after 1.0. Nothing listens on your computer.

## Good to know

- A pad puts the sound on the session; it is not a live player, so it does not play over the transport on its own. Press Play to hear it.
- A pad remembers the file's location. If you move or delete the file, you get a notice that the file could not be read.
