# Altered Audio Editor 0.25.0

Cleaner speech, steadier sessions, and more files that open.

## What's new

- **Speech Denoiser.** A new effect under Restoration. It takes steady background noise out of a voice (fans, hiss, room hum, air conditioning) using the RNNoise model. One control, Amount, from 0 to 100 percent, starts at 80. Use it on a track, a clip, or a selection in the Waveform editor. It works at any sample rate and reports its small delay so the audio stays in place.
- **Snap to Markers in the Waveform editor.** With Snap and its Markers option on, dragging a selection edge near a marker lands exactly on it.
- **Open Recent keeps audio files too.** Audio files you open or save show up beside sessions. A recent file that has moved or been deleted is dropped from the list with a notice.
- **.mkv files open.** The sound comes in through the audio reader. Where the system can show video pictures, the picture opens in the Video panel like any other video.
- **Close All closes everything.** File > Close All and the Files panel's right-click Close All now close every session and every audio file, ask about unsaved work first, and leave the Files list empty.
- **Waveforms stay correct after Save.** Waveform pictures are now built and held in memory (up to 256 MB) and checked against the file's size and time on every use, so a file you saved over redraws from its new contents. Nothing is written next to your audio files. The old waveform cache folder is removed the first time you start this version.

## Fixed

- **Opening several new sessions in a row** no longer shows a red `each_key_duplicate` error in the status bar.

## Good to know

- The Speech Denoiser is for speech. On music it will thin the sound.
- Settings > Media and Cache still lists the waveform entry. It now only shows the old folder until it is cleared.
- Third-party license notices, including the denoiser's, are installed in a `licenses` folder beside the app and are in the portable zip.
- The denoiser model is RNNoise by Xiph.Org and Mozilla, ported to Rust as nnnoiseless, under the BSD 3-Clause license.
