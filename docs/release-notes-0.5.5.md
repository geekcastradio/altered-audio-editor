# Altered Audio Editor 0.5.5

## Fixed
- Long FLAC exports no longer make files that other programs cannot open. Loud 24-bit audio could make the encoder write one oversized block, which crashed players and editors on files around two hours long. A block that would be larger than the raw audio is now stored as raw audio.
- Adding chapters or tags to a FLAC no longer loads the whole file into memory, so it works on very long files.
- Saving from the Waveform editor as MP3 or FLAC now writes the file's chapter markers into it. Before, only WAV files kept markers.
- Long submenus, such as the Plug-ins list in the Effects menu, stay on screen and scroll with the mouse wheel or the arrow keys.
- Time Info no longer lets the times run into each other in a narrow panel.

## Planned
- Markers in every file format that can hold them, with a check after each export that the file is sound.
