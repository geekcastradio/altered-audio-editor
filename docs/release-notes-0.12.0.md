# Altered Audio Editor 0.12.0 (work in progress)

Small fixes on the way to beta.

- **Clearing a tag field removes it.** In File > Edit File Tags and the Metadata panel, a field you empty is now removed from the file (MP3, FLAC, WAV and M4A). Fields you never touch are left alone. Batch Convert and "write a set to every file" still only fill in what is set.
- **Cover art for FLAC and M4A.** The Metadata panel's cover slot now works for FLAC (a PICTURE block) and M4A (the `covr` item) as well as MP3. JPEG or PNG, up to 8 MB. Writing text tags keeps the picture.
- **CI.** Workflow actions moved to their Node 24 versions (upload-artifact v6, download-artifact v7), which clears the Node 20 deprecation warning.
- **Welcome dialog.** It shows on a fresh install and again only after a release that adds a "What's new" page, and then only the new pages, once. Pages can carry a picture (`src/assets/welcome/<name>.png`). Most releases add none.
