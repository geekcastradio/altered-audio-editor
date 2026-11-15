# Altered Audio Editor 0.28.0

Open and write very large WAV files, bring in other editors' projects, put your mix into a video, and run saved editing steps in one go.

## What's new

- **Mixdown Into Video.** File > Export > Mixdown Into Video... mixes your session from the video's first picture onward and writes a copy of the video with that mix as its sound. The picture is copied as it is, with no re-encoding, so it is fast and loses nothing. Any sound the video had is left out of the copy. This works for H.264 files in .mp4, .m4v and .mov, and it needs Windows for now.
- **Run Process Chain.** Multitrack > Run Process Chain... runs a saved list of steps on the open session as one undo. A chain is a small text file. Two steps exist today: Auto Mic Mix and Close Dead Space, with the same settings as their menu commands. A bad chain is refused before anything changes, and a step that fails stops the chain and takes back what already ran, unless you mark it to be skipped. An example sits in docs/chains.
- **Import Session (.rpp).** File > Import Session (.rpp)... brings in a project from another editor as a new, unsaved session: tracks with their names, volume, pan, mute and solo, clips with their positions, start offsets, fades and speed changes, plus markers and regions. Media is found next to the project first. A clip whose file cannot be found is still placed, so you can relink it.
- **Very large WAV files.** The editor now opens RF64, BW64 and Wave64 files, which is what recorders and DAWs write when a take passes 4 GB. WAV exports and multichannel exports switch to RF64 by themselves past that size, so a long multitrack recording is no longer cut off or corrupted. Smaller files stay ordinary WAV.
- **Settings only lists what works.** Rows that were greyed out are gone, and so are two empty pages. Every setting you see does something.

## Good to know

- Project import brings in the editing, not the processing. Plug-ins and effect chains, sends, folders, automation and MIDI are left out, and the notice after import names what the project had.
- Mixdown Into Video starts the sound with the picture. If the session starts partway into the video, move the video first. Other picture formats (HEVC, VP9, .mkv) are refused with a message, and no half-written file is left behind.
- The first process chain steps are a start. More steps arrive in later releases, and the chain file format is the same one they will use.
- Check the first RF64 or Wave64 file you open against what your recorder reports for its length.
