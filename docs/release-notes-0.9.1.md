# Altered Audio Editor 0.9.1

This replaces 0.9.0, which was withdrawn: its window could freeze on start-up ("Connecting to core"). That is fixed here, and everything from 0.9.0 is included.

Pitch correction and bending, a Metadata panel with MP3 cover art, a frequency band splitter, and a snap-to-zero mode.

## Added
- **Pitch Tune and Pitch Curve (Effects > Pitch and Time, Waveform editor).** Pitch Tune pulls a voice toward the notes of a key and scale, with strength, speed and a limit past which a note is left alone. Pitch Curve bends the pitch along a line you set across the selection (start, optional middle, end, up to 7 semitones either way). Both keep the voice's character, since each small slice of sound keeps its own tone color, and leave breaths, consonants and silence as they are. They work on the selection or the whole file and are one undo step. **Experimental:** made for one voice at a time and for small moves. Test it on a copy before using it on a finished episode.
- **Metadata panel (Window menu).** Add audio files and search them by name, title, artist, album or genre. Edit one file's tags with checks on year and track. Keep named tag sets, such as a show's artist, album and genre, and fill a file from a set or write a set to every file in the list. MP3 files can also take cover art (JPEG or PNG, up to 8 MB). Writing text tags keeps the picture.
- **Split into Frequency Bands (Clip menu).** Cuts a clip into two or three tracks (Low, Mid, High) at frequencies you type, each with Precision Filters so the bands add back to the original level. Process each band on its own, such as taming the lows without touching the highs. With three bands the two frequencies must be at least six times apart.
- **Snap Selections to Zero Crossings (Always)** (Edit menu). A dragged selection in the Waveform editor lands on the nearest zero crossings when you let go. Off by default, remembered between runs. The one-time Snap Selection action stays.

## Known limits
- Pitch Tune and Pitch Curve have been checked with measurements, not yet by ear on real speech. Large moves, several voices at once and noisy recordings will sound worse.
- Cover art is MP3 only. FLAC and M4A cover art wait for a later version.
- Clearing a field in the tag editors does not remove it from the file yet.
- Still untested on real hardware: buses and sends on real interfaces, VST3 instruments and effects, MIDI input, and Linux and macOS builds.

## Files panel

- Two right-click menus, laid out like Audition's: one for empty space, one for a file.
- New: Close Selected Files, Close All, Link Media..., Dismiss Error Messages..., Cancel Running Operations...
- Middle-click a file or session row to close it.
- Context menus are light with a blue highlight so they stand out from the dark panels.
- Browser-style keys: Ctrl+W closes the session or file in front; Ctrl+Tab / Ctrl+Shift+Tab (or Ctrl+PageDown / Ctrl+PageUp) step through the Files panel entries.
- Clips: no solid title bar; light readout box under the pointer for the volume line (+0.0 dB).
- Fade handles show the fade curve in a small two-tone box.
- Dials start at 12 o'clock; End works in the Waveform editor; middle-drag shows the hand; QuickPass shows scan percent; Voice Chain presets are a grouped pop-out menu with descriptions.
- QuickPass: Delete strips highlighted findings, right-click menu, Tracks button to skip tracks, Gaps choice shown on the Scan tab.
- Razor tool: razor-blade icon and cursor.
- Razor: clicking a selected clip cuts every selected clip under that point.
- Grouped clips share a group color.
- Volume line on by default, arrow cursor over it, aligned check marks in menus.
- Ripple Delete in the clip menu and on Shift+Delete; blocked edit shortcuts and menu errors now say why.
- Effect settings windows open centered with a light title bar.
- Speech Leveler window like Audition's (amount, dynamic range, boost low signals, compressor, noise gate); panel minimum sizes; loudness scan shows percent.

- New Dynamics effect: Gate, Compressor, Expander and Limiter in one window, each with its own On switch (Effects > Dynamics).
