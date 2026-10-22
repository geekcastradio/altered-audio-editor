# Contributing to Altered Audio Editor

Thanks for helping. Bug fixes, small features, tests, translations of docs and hardware testing are all welcome. You do not need to be an expert, and you do not need to know Git well to start.

## Start here

1. Open the [Roadmap](ROADMAP.md) or the Issues tab and look for the labels **good first issue** and **help wanted**.
2. Comment on the issue ("I'd like to take this") so two people don't build the same thing. A maintainer will assign it to you.
3. No code? Testing counts. Issues labeled **needs hardware testing** are for people with a specific interface, a Mac or Linux machine.

## How changes get in

1. Fork the repo and make a branch for your change.
2. Keep each pull request to one fix or feature.
3. Run the checks (the same ones CI runs): `npm run check`, `npm test`, `cargo fmt --check`, `cargo clippy`, `cargo test` from the README. They must pass.
4. Open a pull request. The template has a checklist; fill it in and describe how you tested.

The maintainers review every pull request. Nothing reaches `main` without a maintainer merging it. Bigger features: open an issue first so we can agree on the approach before you spend the time.

## Ground rules

- **Original work only.** Don't copy code, icons, artwork, names or layouts from commercial audio software, and don't paste code you don't have the right to share. Describe features by what they do, not by what another product calls them.
- **No other products' names** in text, code, comments, issues or pull requests. A check in CI and `npm run check` looks for these.
- **Privacy stays absolute.** No network calls, telemetry, accounts or tracking. The CI privacy check enforces this.
- **Every source file** starts with the GPL header used in the existing files.
- **Plain UI text:** short sentences. Controls for features that don't exist yet show disabled with the reason.
- **Keyboard shortcuts** are not added or changed without the maintainers' approval.
- **Audio code:** nothing in the audio callback may allocate, lock or wait. Ask in the issue if unsure.

## Contributor agreement

Before your first pull request can be merged, you'll be asked to agree to the [Contributor License Agreement](CLA.md). In short: you keep the copyright on your work, your contribution is released under the project's license, and The GeekCast Radio Network, LLC may also license it under other terms so the project can stay maintained and sustainable. A bot will ask you once, in the pull request.

## Reporting bugs

Use the bug report form in Issues. Include your OS and version, the app version (Help > About), what you did, what you expected and what happened. Security problems go through [SECURITY.md](SECURITY.md), not public issues.

## Name and logo

The code is GPL. The name "Altered Audio Editor" and its logo are not; see [TRADEMARKS.md](TRADEMARKS.md). Forks are welcome under a different name.
