# Altered Audio Editor 0.70.3

A feature and fix release after 0.70.2, driven by issues 321 and 331. The Spectrum view gets a playhead, a strength setting for Repair and a left and right channel switch. The transport bar gets listen buttons, and Settings gets an interface size. No new keyboard shortcuts.

## New

- **Spectrum playhead.** Play Selected Area shows a moving line in the Spectrum view, and Space plays or stops it.
- **Spectrum selection.** Adjustable box, typed values and frequency band selection.
- **Repair strength.** A slider after the Repair button (5 to 100 percent) sets how much of the fix is blended in. It is not saved between runs.
- **Left and right Spectrum.** The Spectrum view, its edits and Play Selected Area follow the L and R channel switches.
- **L, R and M buttons on the transport bar.** Listen to one side only, or fold to mono. This changes what you hear, not the audio.
- **Interface size.** Settings > Appearance, 90 to 200 percent. It scales the whole window, so Time Info grows too. Reset puts it back to 100.
- **Convert Sample Type opens the copy.** The converted file is saved next to the original and opened in the Waveform editor. The original is untouched.
- **Built-in H.264 video decoder (off by default).** Pure Rust, from ArtCraft's `craft-video`. MP4 and MOV only. Anything it cannot read falls back to the system decoder. Nothing turns it on yet.

## Known limitations

- **Interface size scales everything**, not only text.
- **HE-AAC files still lose their high band.** Convert to plain AAC for full quality.
- **No stacked left and right Spectrum yet.** Use the L and R switches one at a time.
- **Video:** no player controls yet, no rotation, 8-bit 4:2:0 only, no Matroska. The video decoder is not enabled in this build.
- **Old Cool Edit `.ses` files** are refused with a message. Audition 3 `.ses` files open.
- **Not yet reproduced:** a blank Video panel with some files, and blue boxes when selecting several tracks.
