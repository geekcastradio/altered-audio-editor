# Document review checklist

Rule (Steve, 2026-11-20): the legal, security, readme and how-to documents drift as the app grows, so they are reviewed on a schedule.

- **Every 5th release** (0.50, 0.55, 0.60 and so on): a light review. Read each item below against what shipped since the last review and fix what is out of date.
- **Every 10th release** (0.50, 0.60, 0.70 and so on): the full review. Everything in the light review, plus the legal and licence checks marked (full).
- The review is the first item of that release, logged in `docs/decisions.md`, and the release notes say it happened. Nothing else in that release is allowed to push it out.
- First due: **0.50.0** (full). It covers everything since 0.40. Done 2026-11-20, see `docs/doc-review-0.50.0.md`. Next: 0.55 (light), 0.60 (full).

## Light review (every 5th release)
- [ ] `README.md`: what the app does, install steps, platforms, download links, screenshots.
- [ ] `ROADMAP.md` and `docs/backlog.md`: built items marked built, dropped items removed, the 1.0 line still says what Steve decided.
- [ ] `PRIVACY.md`: matches the app. Every network use listed (update check, Help links), nothing new left out. Settings > Privacy wording says the same.
- [ ] `SECURITY.md`: supported versions, how to report, scope line, the plug-in process note, the update signing note.
- [ ] In-app guide (`src/app/GuideDialog.svelte`, `src/app/guide/`), the Welcome dialog and the About dialog: steps and names match the menus, new features have a how-to, removed ones are gone. `docs/guide-screenshots-wanted.md` is current.
- [ ] `docs/RELEASING.md`: the release steps match how releases are really made now.
- [ ] `docs/codecs.md`: formats and the optional features match the build.
- [ ] Help menu links (`src-tauri/src/commands/links.rs` allow list): every link still works and is still allowed.
- [ ] The keyboard shortcut table in `CLAUDE.md` and `src/keymap/default.json` agree.

## Full review (every 10th release)
- [ ] (full) `ATTRIBUTION.md` and `docs/THIRD-PARTY-*.md`: every ported algorithm and every added dependency since the last review is credited, with its licence text where one is required.
- [ ] (full) Dependency licences: run the licence audit over the Rust crates and the npm packages. Nothing GPL-2.0-only, no unknown licence, vendored copies under `src-tauri/vendor` carry their licence files.
- [ ] (full) Reference sources: anything studied from Audacity, SoundCraft or ArtCraft, or Kdenlive is logged, and any port was cleared with Steve first.
- [ ] (full) `docs/clean-room.md`: the rules still match what is done with `.sesx` and other formats.
- [ ] (full) `TRADEMARKS.md`, `CLA.md`, `CONTRIBUTING.md`, `CONTRIBUTORS.md` and `THANKS.md`: names, rules and contributors are current. Contributors since the last review are listed.
- [ ] (full) Other products' names: search the UI copy, docs and comments for names that must not appear (see the naming map). Interoperability mentions of file formats are the only exception.
- [ ] (full) Colorado student data privacy and "Schools and districts" page, once it exists: still true.
- [ ] (full) Pages outside the repo that Steve keeps (thegcrn.com app pages, the download page, the GitHub repository description): Steve checks them, Claude lists what changed.
