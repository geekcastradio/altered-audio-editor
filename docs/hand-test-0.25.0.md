# 0.25.0 hand test

Run on the installer build from the draft release, on a clean Windows machine if you can. Tick each line.

## Speech Denoiser
- [ ] Open a voice recording with steady noise. Effects > Restoration > Speech Denoiser at 60, 80 and 100. Noise drops; at 100 the voice may sound thin.
- [ ] Amount at 0 sounds the same as the original.
- [ ] Apply to a selection only. Edges are clean, nothing shifts in time.
- [ ] Use it on a track's Voice Chain. Play, stop, play again. No dropouts.
- [ ] Try a 44.1 kHz and a 48 kHz file.

## Waveform and Save
- [ ] Edit a WAV, Save over it. The waveform redraws from the new audio, not blank and not the old picture.
- [ ] No `.peaks` files appear next to any audio file.

## Files panel
- [ ] Open three new sessions in a row. No red error in the status bar.
- [ ] Right-click empty space in the Files panel, Close All. Everything closes, the list is empty.
- [ ] Same with File > Close All, with one session unsaved: it asks, then closes.

## Snap to Markers
- [ ] Add markers. With Snap and Markers on, drag a selection edge close to one. It lands on it.
- [ ] Turn Markers off. The edge moves freely.

## Open Recent and .mkv
- [ ] Open a WAV from File > Open. It appears under File > Open Recent. Click it, it reopens.
- [ ] Move that file, click its recent entry. A notice appears and the entry is removed.
- [ ] Open the sample .mkv. Sound is on Track 1 and the picture shows in the Video panel.

## Install
- [ ] Installed folder has `licenses\ATTRIBUTION.md` and `licenses\THIRD-PARTY-nnnoiseless.md`.
- [ ] Portable zip has the same files.
- [ ] Help > Check for Updates from 0.24.0 offers 0.25.0.

## Mac (ask the Mac tester)
- [ ] Recording latency and the freeze fix hold up on a real Mac.
