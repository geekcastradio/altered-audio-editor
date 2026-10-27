# Altered Audio Editor 0.4.0

Three tools for working with recordings: split a multichannel clip onto its own tracks, capture the noise in a recording with one command, and export a file at a podcast loudness.

## New
- **Split Channels to Tracks** (Clip menu). Select a clip from a file with two or more channels, and each channel gets its own track right under the original. Every new track plays only its channel, the way the clip was already playing it, so with all of them on the mix sounds the same. The LFE track starts muted because the standard fold-down leaves it out. The original clip is muted, not deleted, and the whole thing is one Undo.
- **Capture Noise Print** (Effects menu, Shift+P). Select a stretch that holds only the noise, such as a pause or the room before anyone speaks, and choose Capture Noise Print. The noise is learned and set on a Noise Print Reducer on the selected clip or track. If one is already there, it is updated. One Undo.
- **Export at a loudness** (File > Export > Mixdown). A new Loudness choice brings the exported file to a target: podcast stereo (-16 LUFS), podcast mono (-19 LUFS), louder platforms (-14 LUFS), broadcast, or your own custom target. The mix is measured first, then the whole file is raised or lowered by one fixed amount, with true peaks kept under the target's limit. Your session is not changed.

## Good to know
- If a mix has very high peaks, the export can end up a little quieter than the target so it never goes over the peak limit. Limit the voice tracks first for a louder file.
- Mono targets are measured when the session mixes to mono.
- Effects and automation on the original clip are not copied to the split tracks.
- 3D Space speaker-bed exports are written as mixed.

## Test checklist
- Split a 5.1 file: six tracks appear, the original is muted, the LFE track is muted, and playing all of them sounds like the original
- Undo brings back the single clip, and Redo splits it again
- Select a pause, press Shift+P, and confirm a Noise Print Reducer appears with settings and reduces the noise on playback
- Press Shift+P again on another pause and confirm the same effect is updated, not duplicated
- Export the same session three times: as mixed, to -16 LUFS, and to a target with a very low peak limit, then run Check Loudness on each file
- Export a normal stereo session as WAV and as MP3 with chapters on and confirm the chapters are still written
