# Next work (queue and specs)

Written 2026-11-23 after 0.66.0. A1, A3 and A5 turned out to be built already (checked against the code the same day); what is left to build is A4, A2, A6 and section B. Order is by what the project lead asked for. Each item is written as behavior, so any assistant (Claude, Copilot, Gemini) can pick one up. Read `AGENTS.md` first. A person hand-tests anything that touches audio, plug-ins or devices.

Status words: **ready** means it can be built now with no samples. **needs samples** means see `docs/samples-needed.md`.

## A. Features requested for soon

### A1. True-peak limiter (already built; checked 2026-11-23)
The Limiter has a True peak switch (`src-tauri/src/dsp/effects/true_peak.rs`, 8x oversampled look-ahead, with a test for peaks between samples). Nothing to build. Hand test: a file with inter-sample peaks, Limiter ceiling -1 with True peak on, then Check Loudness should read at or below -1 dBTP.

Original spec, kept for reference:
- Behavior: the Voice Chain Limiter gets a "True peak" switch. When on, it detects peaks on a 4x oversampled copy of the signal, so the output does not go above the ceiling when it is converted to a lossy format or played through a converter. Default ceiling stays -6 dB; the delivery target is -1 dBTP.
- Done when: a test signal with inter-sample peaks measures at or below the ceiling with the `ebur128` true-peak reading; the switch is off for old sessions and presets so they sound the same; the Rust DSP has unit tests; settings window shows the switch; decisions.md has an entry.
- Size: medium. Pure Rust DSP, so it carries into the native interface.

### A2. Better `.sesx` import (needs samples: item 5)
- Behavior: opening a session from Audition also brings in drawn clip volume lines (as clip envelopes), fades, and the effects we have an equivalent for (EQ, dynamics, limiter, noise reduction). Effects we do not have are listed in the import notice as skipped. The original effect data is kept unread and unedited so a later export can write it back (agreed 2026-11-23).
- Done when: the real sample sessions import at the right level (within 1 dB of the original mix); skipped items are listed; round trip through `.altaudio` loses nothing; tests use small hand-made `.sesx` fixtures.
- Size: medium to large. One feature at a time: envelopes first, then effects.

### A3. Single instance (already built; checked 2026-11-23)
`tauri-plugin-single-instance` is in `Cargo.toml` and wired in `src-tauri/src/lib.rs` (since 0.42.0). Nothing to build. Hand test: open an audio file from File Explorer while the app is running; it should open in the running window. Check that the plugin is credited in `ATTRIBUTION.md`.

Original spec, kept for reference:
- Behavior: opening an audio or session file from File Explorer while the app is running opens it in the running app and brings its window to the front, instead of starting a second copy.
- Notes: the Tauri single-instance plugin is MIT or Apache licensed; check it and credit it in `ATTRIBUTION.md` before adding. It must not use the network. Windows only needs to be tested; keep Linux and macOS working.
- Size: small.

### A4. FLAC compression levels (ready)
- Behavior: the FLAC choice in the export dialog and Batch Convert gets a compression level from 0 to 8 (default 5). All levels are lossless; higher is smaller and slower.
- Check first: the notes on the earlier 24-bit FLAC encoder bug (`docs/codecs.md`) so a level never writes a frame larger than the raw samples.
- Done when: files at every level decode to identical samples; settings are remembered with the other export settings.
- Size: small.

### A5. Settings backup and restore (already built; checked 2026-11-23)
Settings > Backup and Restore (`BackupPage.svelte`, `src-tauri/src/backup.rs`): a `.aaebackup` file, Export Everything, Import with Merge or Replace, automatic snapshots. See decisions.md, 2026-11 entries on backup. Nothing to build.

Original spec, kept for reference:
- Behavior: Edit > Settings has "Back up settings" and "Restore settings". Backup writes one file with settings, keyboard shortcuts, workspaces and Voice Chain presets (never audio, sessions or paths to the person's files outside what the settings already hold). Restore reads it, shows what it will change, and is one undoable step where possible.
- Done when: a restore on a clean install reproduces the same settings; a damaged file is refused with a plain message; the file has a `schema_version`.
- Size: small to medium.

### A6. `.aup3` import (needs samples: item 6)
- Behavior: File > Import can open an Audacity project and make a new session: one track per Audacity track, clips at their positions, label tracks as markers. Effects and unsupported edits are listed as skipped.
- Notes: an `.aup3` is a SQLite database with the project structure and audio blocks inside. Prefer reading it with a library whose licence works with GPLv3 (SQLite itself is public domain); check and credit any crate. Write our own reader from the file format; do not copy another program's importer.
- Size: large. Parked before; un-parked 2026-11-23.

## B. Unbuilt menu items to finish "soon"
Each is small to medium. For each, build it or remove the menu item before 1.0.
1. Multitrack > Load and Save Visibility Presets (which tracks are shown).
2. Clip > Groups > Suspend Groups and Remove Focus Clip from Group.
3. Voice Chain Pre and Post fader placement and Pre-render Track.
4. Chain-level envelope targets (automation on a whole chain, not one effect).
5. Video: finish what is in the 1.0 scope (check the Video panel and the gap list for what is missing).

## C. Docs soon
- Guide screenshots: waiting on your PNGs (see `docs/guide-screenshots-wanted.md`, now with how-to). The guide needs a small change to show PNG images.
- Schools page: written (`docs/schools.md`). Review it against `PRIVACY.md` at the next document review.
- SECURITY.md: exists; review at the next document review.

## D. Held until the native interface exists
Insert Session as Clip and sessions as tabs, and other interface-heavy features. Reason: building them twice (Svelte, then native) is waste.

## E. Rust conversion target
Project lead's target: 75% converted by spring 2027. See the 2026-11-23 entry in `docs/decisions.md` for how "75%" is counted and what has to happen.
