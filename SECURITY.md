# Security policy

## Reporting a problem

Please don't open a public issue for a security problem. Use **Security → Report a vulnerability** on this repository (GitHub's private reporting). Include what you found, how to reproduce it, and which version.

You'll get a reply within a week. Fixes ship in the next release, and you'll be credited unless you'd rather not be. Please give us a reasonable time to fix a problem before you share it publicly.

## Supported versions

Altered Audio Editor is not at 1.0 yet, so only the latest release gets fixes. When 1.0 ships, this page will say which releases are supported.

## Scope

The app has no accounts and no telemetry. Its only network use is the update check (Help > Check for Updates, or the optional weekly check), which reads one small file from this project's GitHub releases, and an update is checked against the project's signing key before it installs.

Reports are most useful when they are about:

- Opening crafted audio, video, session (`.altaudio`, `.sesx`, `.rpp`) or zip files.
- The installer and the update check.
- The helper process that runs VST3 plug-ins, and how the app talks to it.
- How the app opens links and files.

Out of scope: bugs in plug-ins made by other companies, in the decoders that come with your operating system, or in GitHub itself, and anything that needs access to an unlocked computer. Please don't test against systems you don't own.
