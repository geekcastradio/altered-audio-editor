# Altered Audio Editor 0.6.0

The first release built around the finish line of an episode: markers that survive every format, a check on the file you just made, and a package ready for your podcast host.

## Added
- **Markers in every format that can hold them.** Chapters are written into exported WAV, FLAC, MP3 and M4A files, and read back when you open one of those files in the Waveform editor. Chapters can carry a link (MP3 and the chapter lists). Batch Convert carries markers and chapters across formats too.
- **File check after export.** After saving, the app reopens the file and checks that it opens, has the sample rate and length you asked for, decodes all the way through, is not silent, and holds the chapters you wrote. A failed check says which one. It is on by default (Export dialog: "Check the file after saving"). CI also runs `flac -t` and `ffprobe` on every format the app writes.
- **Publish Package (File menu).** Fill in title, description, season, episode, type and explicit flag, and the app exports the MP3 and writes a folder beside it: `episode.json` (field names match the RSS.com API), `show-notes.txt`, `chapters.json`, `soundbites.json` and a README. It checks the title and description against RSS.com's limits first. Nothing is uploaded; the app still makes no network calls.
- **Loudness scan before export.** The Export and Publish dialogs show the mix's loudness, true peak and the gain a normalize would apply, with a plain-language warning if the result would clip or miss the target.
- **Chapter polish.** A link field per chapter, and tips on titles that are long, wordy or repeat a word.
- **Convolution Reverb**, **Vox Strip** (a vocal chain in one effect), **Reverse** and **Invert Polarity** on clips (non-destructive), and **fade curves you bend** by dragging the fade handle.
- Effect lists sorted alphabetically with search; VST3 plug-ins grouped by type then maker; menus jump to an item as you type its first letters.
- Per-effect presets; a tidier clip menu; a **Join the GCRN Community on Discord** item in the Help menu; QuickPass fits narrow panels; a flat look for the new effects.

## Changed
- Saving from the Waveform editor as WAV, FLAC, MP3 or M4A keeps chapter markers (before: MP3, FLAC and WAV only).

## Credits
Several features here come from the Audiform project (GPL-3.0-or-later), adapted to our code and look: Convolution Reverb, Vox Strip, Reverse and Invert Polarity, fade bending, sorted and searchable effect lists, and menu type-ahead. Thank you to its developer. See ATTRIBUTION.md.

## Known limits
- M4A export and its check run on Windows only.
- WAV chapters need a plain RIFF file; RF64 and files that already hold cue points are left alone with a message.
- Ogg and Opus are read but not exported.
