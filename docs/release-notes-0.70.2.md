# Altered Audio Editor 0.70.2

A small bug-fix release after 0.70.1: Space at the end of the audio starts again from the top, and HE-AAC files no longer play at double speed. No new keyboard shortcuts.

## Fixes

- **Space at the end of the audio** now starts from the beginning. With the playhead at the end and nothing selected, it used to play on into silence while the timer kept counting.
- **HE-AAC files play at the right speed.** A 48 kHz AAC file that is really HE-AAC played at double speed. It now imports at its half-rate core (24 kHz for a 48 kHz file) with the true length, and the Files panel shows "AAC (HE, without high band)".

## Known limitations

- **HE-AAC files lose their high band.** Sound above about 12 kHz is missing, because the part of HE-AAC that rebuilds it is not decoded yet. Speed and length are right. For full quality, convert the file to plain AAC first.
- **Not yet reproduced:** a blank Video panel with some files. It needs a sample file from the reporter.
- **Old Cool Edit and Cool Edit Pro `.ses` files** are refused with a message. Audition 3 `.ses` files open.
- **Convert Sample Type** writes a new file. It does not change the file open in the tab.
