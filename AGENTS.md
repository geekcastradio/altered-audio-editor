# Instructions for AI coding assistants

These rules apply to GitHub Copilot, Gemini and any other assistant that works in this repository. People can read them too. `CLAUDE.md` has the full project rules; this file is the short version that every assistant must follow.

## What this is

Altered Audio Editor is a free, open-source audio editor for podcasters and voiceover artists. Tauri 2, Svelte 5, TypeScript (strict) and Rust. Licensed GPLv3. Windows is the tested platform; Linux and macOS are experimental. The project is moving its logic, and later its whole interface, from TypeScript to Rust. See `docs/decisions.md` for what was decided and why.

## Rules that are never broken

1. **Original names and looks.** Never use another commercial product's feature names, panel names, icons, colors, menu trees or wording in code, comments, docs or UI text. When a name is in doubt, use a plain descriptive one (Voice Chain, Batch Convert, QuickPass, Noise Reducer). See `docs/clean-room.md`.
2. **Describe behavior, not other products.** Never copy code from other products. Open-source projects listed in `CLAUDE.md` may be read for ideas; write your own version and credit any ported algorithm in `ATTRIBUTION.md`.
3. **Privacy.** No network requests, telemetry, accounts or analytics. The only network use is the update check described in `PRIVACY.md`. A change that adds any other network use will not be accepted.
4. **New dependencies** need a licence check first (it must work with GPLv3) and a credit in `ATTRIBUTION.md`.
5. **No new keyboard shortcuts** without the project lead's approval.
6. **Every edit is undoable** and goes through the command stack. The interface never touches audio; the Rust core does.
7. **Never break old files.** All file formats are versioned JSON with migrations.
8. **Every source file starts with the GPLv3 header** naming The GeekCast Radio Network, LLC. Copy it from a neighbouring file. `npm run check` verifies it.
9. **Writing style** for UI text, docs and comments: plain, short sentences, no em dashes.
10. **Do not publish or tag releases**, and do not edit release notes of published releases.
11. **Private files stay private.** If `CLAUDE.local.md` or a `private/` folder exists on your machine, never copy anything from them into tracked files, commits, pull requests or issues.
12. **Branch names start with your tool:** `claude/`, `gemini/` or `copilot/`. Do not push to another tool's branch.
13. **Keep the history clean.** When a good run or commit replaces a failed one on your branch, say so; the project lead has asked that failed runs are deleted once a green run replaces them.

## How to work

- One feature or fix per branch and pull request, built from `main`. Small commits, conventional messages.
- Run before you open the pull request, from the repository root: `npm run check`, `npx vitest run`. From `src-tauri`: `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`.
- Add a dated entry to `docs/decisions.md` for anything that changes behavior or architecture, with the reason.
- If a requirement is unclear, ask in the issue. Do not guess about audio behavior, file formats or what the project lead wants.
- Anything that touches audio devices, plug-ins or channel layouts needs a hands-on test by a person; say so in the pull request.

## Porting TypeScript logic to Rust (the common task)

The pattern is in every `src-tauri/tests/*_fixture.rs` file:

1. Pick a TypeScript module with pure logic (no DOM, no stores).
2. Write a throwaway test that runs the real TypeScript functions and writes inputs and outputs to `shared-fixtures/<name>.json` (keep it under about 300 KB; store NaN and Infinity as strings).
3. Write the Rust module `src-tauri/src/<name>.rs` (register it alphabetically in `lib.rs`).
4. Add `src-tauri/tests/<name>_fixture.rs` that reads the fixture and compares with a small tolerance, and a Vitest test beside the TypeScript module that reads the same JSON (import it directly).
5. The TypeScript copy stays in use. Both must pass the same fixture so they cannot drift.

## Where things are

- Rust core: `src-tauri/src` (commands are thin wrappers in `src-tauri/src/commands`).
- Interface: `src/app`, `src/panels`, `src/stores`; menus are data in `src/app/menus/model.ts`.
- Docs: `docs/`. Hand-test lists: `docs/hand-test-<version>.md`.
- Work queue and specs for the next features: `docs/next-work.md`.
- How the assistants split the work and avoid collisions: `docs/ai-workflow.md`.
