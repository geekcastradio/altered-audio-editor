## Altered Audio Editor 0.50.0

### New
- **Light theme.** View > Light Theme (or Settings > Appearance > Theme) switches between the dark look and a light one. Dark stays the default. Some drawings in the editor may still look dark on the light theme; tell us which and they get fixed.
- **Click a dial's number to type a value.** On any track, bus or master volume, pan, or the click level, click the number, type (for example -2, +3.5, or L30 for pan), and press Enter. Escape cancels. Each typed value is one undo step.
- **Audio probe and speed benchmark for testers.** `cargo run --example audio_probe` opens your sound hardware the way the editor does and says what happened (add `-- --asio` to try ASIO). `cargo run --release --example bench_mix` mixes a dense generated session and reports how many times faster than real time it runs.

### Changed
- **Help > Copy Bug Report Details** now includes the audio driver, output device, channel count and buffer request, so "the audio device did not respond" reports say which device it was. Your user name is still left out of paths.
- **Update and export notices are in the middle of the window.** The update notice and the export progress box (mixdown, stems, bounce, playlist, video) no longer sit in the lower right corner. Neither blocks your work.
- **The message about files that could not be copied into the session folder** now says the session still saves and uses them where they are, and to put a moved file back at the path shown.
- **Sessions keep fields they do not recognize.** A track or clip field written by a newer version now survives opening and saving in this one.
- Behind the scenes, more of the app's logic moved out of the window code into the core (where libraries and working files go, and building waveforms). Nothing looks different.
- The documents were reviewed (README, privacy, security, roadmap and the release steps).
