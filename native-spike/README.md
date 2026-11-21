# Native interface spike (Phase 2)

A small program that tests a Rust-native interface with egui and egui_dock. It is not the app, is not part of any release, and plays no sound. It exists so the toolkit choice rests on numbers from a real machine. The plan is in `docs/decisions.md` and the phased plan in the project docs.

What it has:

- A dock layout (Files, Editor, Levels, Settings) that can be dragged, split and tabbed.
- A transport bar with a timecode and a frame time readout (average and worst 1%).
- A three-hour synthetic waveform (2 million peak blocks) drawn each frame. Drag to scroll, mouse wheel to zoom.
- Thirty-two animated meters.
- Text fields: a search box, a multi-line notes box, a slider and a drag value. Try typing, pasting, selecting, an input method editor and right to left text.

It writes `spike-timing.log` in the folder it runs from: when the window and renderer were ready, and when the first frame drew.

Build and run (Windows, from this folder):

```powershell
cargo run --release
```

Or download the built `.exe` from the "Native spike" workflow run on GitHub (Actions, Native spike, the run, Artifacts).

What to send back: the contents of `spike-timing.log`, the frame time shown in the top bar while the waveform is zoomed out fully and zoomed in, how it behaves when you type in the Notes box, and your CPU and graphics card.

The spike has its own `Cargo.toml` and workspace, so the app's checks and builds never include it.
