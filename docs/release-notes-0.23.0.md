# Altered Audio Editor 0.23.0

Video reference. Line your edit up against a picture without a web video player in the app.

## What's new

- **Open a video like any file.** File > Open takes video files (MP4, M4V, MOV, WMV, AVI) as well as sessions and audio. The video becomes the session's reference, the Video panel opens, and its sound goes onto the first track when the session has no audio yet. A file with no picture, such as an audio-only MP4, opens as audio like before.
- **Video panel.** Window > Video, or File > Open. The picture follows the playhead and a timecode sits over it. It is drawn from the decoder built into the operating system, so the app does not carry a second browser engine's video player and memory use stays small.
- **Saved with the session.** The session remembers which video it is lined up against and where the video starts (Starts at, in seconds, can be negative). Undo and redo work on opening, moving and removing the video. Older sessions open as before.
- **Windows.** Pictures from MP4, M4V, MOV and WMV files that Windows can decode.
- **Phone clips.** Vertical videos that carry a rotation stand upright.
- **Updates on Mac and Linux.** Help > Check for Updates can now install an update on macOS (from the app) and on Linux (from the AppImage), as it already did on Windows. The .deb and a Mac app run from outside an app bundle still get a link to the download page. This starts with this version, so installs of 0.22.0 and older need one manual download of 0.23.0 first.
- **Intel Macs.** An experimental Intel build is attached to the release.
- **macOS and Linux.** Pictures are not available there yet. The panel says so, and the sound of a video still imports as before.

## Good to know

- The picture follows the playhead to the nearest frame. It is a reference to cut against, not a frame-accurate player, and it can lag a little during playback on long-GOP files.
- The video is not mixed or exported. Its file stays where it is; if you move it, the panel says so.
- Files with an unusual codec may open without a picture if Windows has no decoder for them.
