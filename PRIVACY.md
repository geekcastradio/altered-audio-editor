# Privacy

Altered Audio Editor has no accounts, no sign-in, no telemetry, no analytics, no tracking and no ads. It works fully offline, and nobody has to do anything for you to use every feature.

## What the app does not do

- It never contacts the internet on its own. There is no update check, crash reporter, usage counter or license check.
- It does not collect, send or sell anything about you, your computer, your audio or how you use the app.
- It needs no account and never asks you to sign in.

## What stays on your computer

Everything the app saves stays in folders on your own computer:

- Sessions (`.altaudio`) and libraries wherever you save them.
- Settings, keyboard shortcuts, workspaces, autosaves and the waveform cache in the app's own config, data and cache folders.

## The only time the app touches the web

The Help menu has links (The GCRN website, the book, and an optional PayPal page to support the project). They open in your own web browser only when you pick them. The app itself only opens https links on a short allow list, kept in `src-tauri/src/commands/links.rs`.

## For contributors

Changes that add network access, telemetry, accounts or tracking of any kind will not be accepted. If a future feature truly needs the network (for example, an optional sync with a podcast host you choose), it must be off by default, clearly labeled, and never required to use the app.
