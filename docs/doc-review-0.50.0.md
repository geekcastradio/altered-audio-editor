# Document review for 0.50.0 (full)

Done 2026-11-20 against `docs/doc-review-checklist.md`. It covers what shipped since 0.40.

## Fixed in this review

- `README.md`: the download list now includes the MSI; the macOS note covers Intel as well as Apple silicon; "Early versions are pre-releases" now says early releases (releases are normal releases, not pre-releases).
- `ROADMAP.md`: no longer names another product; the 1.0 line now says what Steve decided on 2026-11-20 (Rust interface, video, MIDI tracks, soundboard, localization in; AI and outside-call features after 1.0); the bug report path names the real menu items.
- `PRIVACY.md`: the Help links list now matches the allow list (GitHub, Discord, the book); "Report a Bug" is "Report a Problem", and Copy Bug Report Details is described, including the new audio driver and device lines and that user names are scrubbed from paths.
- `SECURITY.md`: the update check wording matches Settings (at start or weekly, off by default).
- `docs/RELEASING.md`: rewritten to the real flow (bump PR, run the Release workflow on `main`, draft with all platforms, hand test, publish, the tag push builds nothing).

## Checked, no change needed

- `docs/codecs.md`, `ATTRIBUTION.md`, `docs/THIRD-PARTY-*.md`: match the dependency list in `src-tauri/Cargo.toml` (DeepFilterNet and RNNoise credited with notices, ASIO, LAME, Symphonia, opus-decoder vendor copy).
- npm licences (read from `package-lock.json`): MIT 71, Apache-2.0 OR MIT 13, MPL-2.0 12, Apache-2.0 5, OFL-1.1, ISC, BSD-3-Clause one each. Nothing GPL-only or unknown.
- Keyboard table spot check against `src/keymap/default.json`: Ctrl+K split and Shift+Space record agree.
- Reference sources: the AudiForm 2 ideas-only rule and the Kdenlive rules are in `CLAUDE.md`; nothing was ported from either.

## Needs Steve or a follow-up

1. **Rust crate licence audit not run.** Claude's workspace has no cargo, and the npm check covers only the interface. Proposal: add `cargo deny check licenses` (a `deny.toml` with an allow list) to CI so the full-review item is repeatable on every release. Needs a third-party tool in CI, so Steve decides.
2. **Other products' names.** `docs/clean-room.md`, `TRADEMARKS.md` and the shortcut and `.sesx` import code name another editor on purpose (compatibility statements, file formats, and a shortcut import). Beyond those, the word still appears in about 25 other source files and in `CLAUDE.md` and `docs/audition-shortcuts.md` (the file name too). The private-term check is skipped here because `private/banned-terms.txt` is not in this workspace, so Steve should run `npm run check` on his machine for the real list. Proposal: rename `docs/audition-shortcuts.md` and review the UI strings and comments in a later release.
3. **Not reviewed here (outside the repo or needs the app open):** the in-app guide and Welcome and About dialogs against the menus, `docs/guide-screenshots-wanted.md`, every Help link actually loading, thegcrn.com app pages, the download page, the GitHub repository description, the Colorado schools page (does not exist yet). Claude can do the guide pass next; the web pages are Steve's to check.
4. **Changes since 0.40 Steve should know for the web pages:** MSI download, Intel Mac build, centered update notice, OTIO import, spectrogram schemes, Copy Bug Report Details audio lines.
