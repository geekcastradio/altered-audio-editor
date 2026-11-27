# Altered Audio Editor 0.70.4

A small release after 0.70.3. It fixes the problems found in the Spectrum view and adds a Settings switch for the built-in video decoder. No new keyboard shortcuts.

## New

- **Built-in video decoder switch.** Edit > Settings > Playback and Recording > Video has "Use the built-in video decoder first". It is off by default. When it is on, H.264 pictures in MP4 and MOV files are decoded inside the app, so they do not depend on what your system has installed. A file it cannot read opens with the system decoder instead. The open picture reloads when you press OK. On Linux it is the only video decoder.

## Fixes

- **Spectrum playhead.** The moving line during Play Selected Area could disappear or flash at the start, most often when Play was triggered twice. It now stays for the whole play.
- **Lasso selections play at once.** They used to wait a long time before the sound started.
- **Box reaches 0 Hz.** The Low value stopped at 30 Hz. A box dragged to the bottom now reads 0 Hz and also takes everything below 30 Hz.
- **Box borders** at the bottom and right edges of the Spectrum view are drawn again.
- **Frequency scale.** The top mark no longer overlaps the "Hz" heading.

## Known limitations

- **No stacked left and right Spectrum yet.** Use the L and R switches one at a time.
- **Video:** no player controls yet, no rotation, 8-bit 4:2:0 only, no Matroska. The built-in decoder handles H.264 in MP4 and MOV only.
- **HE-AAC files** still lose their high band. Convert to plain AAC for full quality.
- **Old Cool Edit `.ses` files** are refused with a message. Audition 3 `.ses` files open.
- **Not yet reproduced:** a blank Video panel with some files, and blue boxes when selecting several tracks.
