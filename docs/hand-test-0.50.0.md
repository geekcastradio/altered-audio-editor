# Hand test: 0.50.0

- View > Light Theme. The whole app goes light. Switch back with View > Dark Theme. Do the same from Edit > Settings > Appearance > Theme. Look at the multitrack lanes, the Waveform editor, the spectrogram and the Mixer and tell me what still looks wrong.
- Click the number next to a track's volume dial (like -2.0). Type -6 and press Enter: the dial moves. Type 3 and Enter. Type abc and Enter: nothing changes. Press Escape while typing: nothing changes. Ctrl+Z undoes it.
- Do the same on pan: type L30, then R20, then C.
- Start File > Export > Mixdown. The progress box appears in the middle of the window and you can still click in the editor behind it.
- Help > Copy Bug Report Details, then paste into a note. It lists Audio driver, Output device, Output channels and Buffer request.
- Open a session where a file is missing and import another file from a different folder. The notice in the middle of the window says the session still saves. Then save.
- Save a session, close and reopen it. Nothing is different.
- Optional, on your PC: in `src-tauri`, run `cargo run --release --example bench_mix` and send me what it prints and your CPU model. Run `cargo run --example audio_probe` and send me what it prints.
