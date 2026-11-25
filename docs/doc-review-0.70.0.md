# Document review for 0.70.0 (full)

Done 2026-11-25 against `docs/doc-review-checklist.md`. No review record exists for 0.55 or 0.60, so this one covers everything since 0.50.

## Fixed in this review

- `src/app/guide/guide.md`: the import line described the old separate Import Session items. It now describes File > Import > Session with the file type list, and no longer names another product. New how-to lines for Favorites and the Quick Actions bar, Save Audio As options and the ask-before-Save setting.
- `SECURITY.md`: the scope list now names every session type the app opens (`.aup3`, `.aup4`, `.otio` added).
- `THANKS.md`: reporters since 0.43 are listed (Dean-Corso, Reaper10, Garretiko, eugenialoli, diego_au).
- `src-tauri/vendor/opus-decoder`: the crates.io package ships no licence files, so the folder had none. `LICENSE-MIT` (upstream text) is added and `ALTERED-AUDIO.md` says where it came from.

## Checked, no change needed

- Rust crate licences (667 packages, read with `cargo metadata`): MIT or Apache-2.0 families, MPL-2.0 (20), Unicode-3.0, BSD, ISC, Zlib, CC0, CDLA-Permissive-2.0. LGPL-3.0 appears only on `mp3lame-sys` and `mp3lame-encoder` (credited in `ATTRIBUTION.md` and `docs/codecs.md`) and as one option of a dual licence on `r-efi`. Nothing GPL-2.0-only, AGPL or unknown.
- npm licences (from `package-lock.json`): MIT 71, Apache-2.0 OR MIT 13, MPL-2.0 12, Apache-2.0 5, OFL-1.1, ISC, BSD-3-Clause one each. Nothing GPL or unknown.
- Direct dependencies against `ATTRIBUTION.md`: the ones not named there are plain MIT or Apache-2.0 crates covered by the lock file line at the top of that section.
- `PRIVACY.md` matches the Help links allow list (GitHub, Discord, the book, PayPal, thegcrn.com) and the update check wording. No new network use was added since 0.50.
- Keyboard table: spot check of 13 chords from the `CLAUDE.md` tables against `src/keymap/default.json` found no missing chord.
- `CONTRIBUTORS.md`: unchanged. Git history shows only the project's own commits since the last review.

## Needs Steve or a follow-up

1. **Another editor's name in tracked files.** Same finding as the 0.50 review, still open. The name appears in published release notes and hand-test pages (0.9 to 0.43), `docs/audition-shortcuts.md` (file name too), `docs/samples-needed.md`, `docs/audiform-intake.md`, shared fixtures for the shortcut and rack importers, and about 20 source files and comments. Some are legitimate interoperability mentions (importers, file formats). The release notes for 0.9.x and 0.39 to 0.41 say things like "laid out like Audition's", which the naming rules do not allow. The private-terms check cannot run here because `private/banned-terms.txt` is not in this workspace. Proposal: rename `docs/audition-shortcuts.md`, reword the old release notes, and run `npm run check` with the real list on Steve's machine. Old release notes are public history, so Steve decides whether to edit them.
2. **Not reviewed here (outside the repo or needs the app open):** `docs/guide-screenshots-wanted.md`, every Help link actually loading, the Welcome and About dialogs against the menus, thegcrn.com app pages, the download page, the GitHub repository description.
3. **For the web pages, changes since 0.50:** Import submenu and one Session dialog (`.sesx`, `.rpp`, `.aup3`, `.aup4`, `.otio`), Save Audio As codec options, Favorites menu and Quick Actions bar, ask-before-Save setting, multitrack right-click menu groups, calmer light theme.
4. **Reviews skipped:** 0.55 and 0.60 have no record. The next ones are 0.75 (light) and 0.80 (full).
