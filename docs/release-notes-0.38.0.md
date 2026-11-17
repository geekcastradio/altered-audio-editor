# Altered Audio Editor 0.38.0

A release candidate for 1.0. It follows 0.36.0 and carries the planned 0.37.0 documentation work and the 0.39.0 menu completion in one release. Every menu item that used to say "Not built yet" now works or has been removed, and the User Guide covers the panels and tools it was missing. From here the aim is fixes only.

## What's new

- **Level knob preview.** In the Waveform Editor, the waveform now changes height while you drag the level knob, for the selection or the whole file. You can judge the level by eye. The change still applies when you let go.
- **Fill Single Click.** New under Effects. Select one click or pop (up to 25 ms) and it is smoothed over from the audio on each side. It works on mono and stereo files, is one undo step, has no dialog, and has the key Ctrl+Alt+K. Repeat Last Command (Shift+R) repeats it on the next selection, so you can work through a file click by click.
- **Track Visibility presets.** Save Visibility Preset remembers which tracks are hidden under a name. Load Visibility Preset brings it back in one undo step. Presets match tracks by name, so one made for an episode works in the next. The newest eight are kept.
- **Meter Input Signal (Alt+I).** Shows the input level on every track, not only the armed ones, so you can check a mic before arming it.
- **Suspend Groups (Ctrl+Shift+G) and Remove Focus Clip from Group.** Pick a single clip from a group without breaking it, or take just the clip you clicked last out of its group.
- **Guest Track.** Multitrack > Add > Guest Track adds a mono track named Guest with the Remote Guest profile already on, ready for a guest's recording.
- **Podcast Mode and Voiceover Mode.** In the View menu. They switch to the Default or Voiceover workspace and turn Music Mode off.
- **User Guide.** New sections for Play Selected Area, the Soundboard, the Automation page in Settings, Batch Convert, recording a remote guest with a double-ender, Track Visibility, Groups, Meter Input Signal, Host profiles and Guest Track, Panels and workspaces (Levels, Time Info, History, Metadata, Drive Manager, Playlist, Phase Wheel and 3D Space), and Everyday tools (Generate, Fill with Room Tone, Speaker Balance and the audiobook delivery check).
- **Welcome pages.** A fresh install now also shows pages on comping takes, process chains and batches, the new Speech Denoiser (High Quality), and quicker clean-up with tidier views. After an update you see only the pages you have not seen.

## Removed

- Remove Music, Transcribe Session and Remove Filler Words are no longer in the menus. They need speech and separation models and arrive after 1.0.

## Good to know

- Fill Single Click refuses selections over 25 ms and clicks within a few samples of the file edges.
- Playback speed and the buffer size are still marked experimental.
- Nothing in the program uses the network except what you start yourself, such as Check for Updates.
