# Altered Audio Editor 0.30.0

Take one mic out of another, for recordings where every microphone also hears the other speakers.

## What's new

- **Reduce Bleed.** Multitrack > Reduce Bleed... cleans one mic's track against another's. It learns how the other mic's sound reaches this one and takes that out, while keeping the voice on this mic, even when both people talk at once. Each clip is cleaned into a copy in a "Bleed Reduced" folder beside your session. Your original files are never changed, and one undo puts every clip back. It uses ordinary signal processing, so there is no model to download and nothing leaves your computer.

## Good to know

- Run Reduce Bleed before Auto Mic Mix. Auto Mic Mix turns tracks down and the cleaner works best against the untouched recording of the other mic.
- The two tracks should be in step. If they are not, use Sync Tracks first. A small offset is fine.
- Locked clips, and clips that are looped or stretched, are left alone, and the notice says how many.
- A mic that picked up almost none of the other one is left as it is.
- The settings behind it are tuned on test audio so far. Tell us how it sounds on your real recordings.
