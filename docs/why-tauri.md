# Why Tauri, and what would make us reconsider

Altered Audio Editor uses Tauri 2 with a Rust backend and a Svelte 5 / TypeScript front end. People ask whether a native Rust GUI such as egui would be better. This note records why we chose what we did and what evidence would change our mind.

## Where the work happens

- Audio decoding, playback, DSP, effects, file handling and the session engine are Rust.
- The webview draws the interface and sends commands to Rust.
- So the GUI toolkit does not set audio latency, playback reliability or processing speed. The audio engine, buffering and scheduling do.

## Why Tauri

- **UI breadth.** Multitrack editing, floating and docked panels, dialogs, menus, context menus, a remappable keymap and a long guide are a lot of interface. The web platform gives text input, selection, IME, accessibility, scrolling and layout for free.
- **Speed of iteration.** One person builds and tests this. Svelte lets us change a panel quickly.
- **Small, no-network build.** Tauri apps are small, and we can ban network crates and check it in the privacy gate.
- **Cross-platform.** Windows, macOS and Linux from one code base.

## What it costs

- A webview draws waveforms and spectrograms, so very dense drawing at high zoom can lag compared with a native renderer.
- Two languages and a bridge between them. Large data has to cross it carefully.
- Webview differences between platforms can cause small rendering and input differences.

## What we do about the costs

- Heavy drawing uses canvas, and peak data is prepared in Rust.
- Large audio stays in Rust. The front end gets peaks and results, not raw samples.

## What would make us reconsider

We would look at a native GUI (egui or another) if measurements on a realistic project show a problem we cannot fix inside the current design:

1. Waveform or spectrogram redraw that cannot hold a smooth frame rate at normal zoom on a mid-range PC, after canvas and peak caching work.
2. Spectral selection or scrubbing that feels laggy because of the webview, not because of Rust.
3. Memory or CPU use that is clearly worse than a native equivalent under the same workload.
4. A platform webview bug we cannot work around.
5. A concrete benefit from an ecosystem or distribution partner that needs a native toolkit, weighed against our license goals and the rewrite cost.

If we try it, the first step is a small spike (one panel, such as the waveform view, in egui next to the current one) with numbers on redraw time, CPU and memory. It is not a migration.

## What would not change our mind

- That native should theoretically be faster, with no measurement.
- Fashion in GUI toolkits.

Measurements and reports from real use are welcome in an issue.
