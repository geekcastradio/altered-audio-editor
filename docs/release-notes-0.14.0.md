# Altered Audio Editor 0.14.0

Built for voiceover and podcast work: fixes for the things that cost you takes, checks before you deliver, and a few editing gaps closed.

## New for voice work

- **Delivery Check.** Check Loudness has a new target, "Audiobook (RMS, peak, noise floor)". It measures the three numbers audiobook retailers ask for (average level, highest peak and noise floor), shows pass, close or fix for each, and offers a one-click master gain when gain alone would fix it. Confirm the exact limits against your retailer's current spec before you submit.
- **Fill with Room Tone** (Effects menu, Waveform editor). Select a gap, a cough or a cut, and it fills the selection with room tone taken from the quietest stretch elsewhere in the same file. The edges blend in, and one undo takes it back. It refuses to run when the file has no quiet stretch to borrow from.
- **Plosive Tamer** (Effects > Dynamics). Pulls down the low thump of "p" and "b" sounds only when they happen, and leaves the rest of the voice alone. Presets for close mic, standard mic and a light touch.
- **Match EQ** (Effects menu). Pick a source track and a reference track. It compares their tone, fits a Parametric EQ that moves the source toward the reference, and adds that EQ to the source track as one undo step. It matches tone only, not volume.
- **Retake markers.** Press F9 while recording to drop a "Retake" marker at that spot, so the bad take is easy to find when you edit. Also under Edit > Markers.
- **Batch chains.** Batch Convert can now trim silence from both ends, bring every file to a loudness (LUFS) without letting peaks pass a ceiling you set, and save the whole set of steps under a name. Starter chains: Podcast at -16 LUFS, Voiceover, Audiobook, and a quick peak match. Names, tags and the output folder stay per job.

## Editing

- **Auto Heal Selection** (Ctrl+U). Repairs the selected stretch from the audio around it in one step.
- **Merge Clips in the right-click menu.** Right-click the selected clips and choose Merge Clips. The merged file is saved with the project files.
- **Show Only Bus Tracks** (track right-click menu). Hides the audio tracks so only the buses show.
- **Return to Start on Stop** (a toggle in the Multitrack menu). On by default. Turn it off and Stop leaves the playhead where playback ended.
- **Pan law.** New sessions start with a -3 dB centre. Older sessions keep their current balance behaviour, and Session Properties lets you change it.
- **Convert Sample Type** (File menu). Converts the open file to the sample settings you choose, saved as a new file beside the original.

## Presets

- **Preset packs.** Export all your presets to one file and import a pack someone sends you. Importing never overwrites: a name that is already taken gets a number.

## Fixes

- The Audio FX panel title now updates in layouts saved by older versions.
