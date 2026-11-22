# Altered Audio Editor 0.62.0

Play the spectral selection from where you are looking, a new Help link for questions and ideas, a fix for the missing Apple silicon Mac build, and three more batches of the move from TypeScript to Rust.

## New

- Spectrum view: a Play button on the bar over a selection, and Play Selected Area as the first item in the right-click menu. The command was already in Effects > Spectrum; now it is where you draw the selection.
- Help > Ask a Question or Share an Idea opens the GitHub Discussions page in your browser.

## Fixed

- macOS (Apple silicon): the release build could fail while making the disk image, which left that download out of 0.61.0. The step now retries and clears stuck disk images first.
- Typing a key name like "constructor" in the shortcut editor no longer reads as a real key.

## Moved to Rust, with tests that compare both sides

The interface still uses its TypeScript code; the Rust copies are tested against the same fixtures so they can take over later.

- Key chords: reading a typed shortcut and building one from a key press.
- The list of libraries that moves between computers.
- The tempo and time signature map: beats and seconds, bar lines, and adding, moving, editing and removing changes.
- MIDI clip and note edits: snapping, note names, moving, resizing, copying and pasting.
- Source Channels: channel names, the standard fold-down, the quick settings and matching a mix to one.
- Waveform editor channel switches: on, off, solo, and which channels an edit or paste reaches.
- Clip numbers: length and end on the timeline, stretch limits, MIDI clip spans, snapping and fade shapes.

## Notes

- No new shortcuts.
- Free, GPLv3, no telemetry.
