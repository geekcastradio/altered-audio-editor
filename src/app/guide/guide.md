## Start here

Altered Audio Editor is a free, open-source multitrack audio editor for podcasters and voiceover. It works like the editors you may know from Adobe Audition, with extras made for spoken word.

![The Multitrack Editor: 1 the ruler, 2 a track header, 3 a clip and 4 the playhead](editor-overview)

There are two editors. The **Multitrack Editor** is where you build an episode: tracks, clips, effects, mixing and export. The **Waveform Editor** edits one audio file at a time, down to the sample. Switch with View > Switch Between Editors, or pick one from the View menu.

Your first episode in five steps:

1. Choose File > New Session, pick a name and a sample rate (48000 Hz is a good default), and press OK.
2. Bring in audio with File > Import Files, or drag files from your file manager into the window. Use Multitrack > Insert Files as Tracks to put each file on its own track.
3. Press the space bar to play and stop. Click in the ruler to move the playhead.
4. Fix what you hear: split clips at the playhead (Clip > Split at Playhead), trim edges, and add effects in the Voice Chain.
5. When it sounds right, choose File > Export > Session Mixdown.

Save often with File > Save. While there are unsaved changes the program also keeps a recovery copy, and offers to recover it if it did not close normally.

If you open a session made by an older version, nothing changes until you save. Saving upgrades it and keeps the original in the Backups folder, named with the old version, and a notice says so. Older versions of Altered Audio cannot open a session once it is upgraded. If the backup cannot be made, the save is stopped and your file is left as it was.

## Privacy

There is no account, no sign-in, no tracking and no telemetry. Your audio and sessions stay on your computer. The program goes online only when you choose Help > Check for Updates or open a link from the Help menu. There is also a weekly update check that is off until you turn it on in Settings.

## Recording

Add a track with Multitrack > Add > Mono Track or Stereo Track, then arm it with the **R** button in its header. Pick the input in the track header. Press Record (Multitrack > Record) and press Stop when you are done. The take lands on the armed track as a new clip at the playhead.

Recording Options (Multitrack > Recording Options) holds pre-roll, punch-in and the recording offset. With a time selection, Record starts a pre-roll before it, stops by itself at the end and replaces what was there on the recorded tracks (punch-in). The offset trims a few milliseconds from the start of each take so takes line up with what they were played against.

To record a call, each output device is also offered as an input named "System audio" followed by the device. It records what the computer plays, while microphones record on their own tracks.

## Tracks, clips and editing

A **track** holds clips. A **clip** is a piece of an audio file placed on the timeline. Cutting a clip never changes the file on disk.

- Split at the playhead with Clip > Split at Playhead, or all clips at once with Split All Clips at Playhead.
- Drag a clip to move it. Lock Position keeps a clip where it is.
- Drag the small square handles in a clip's title bar to set its fade in and fade out. New fades start as an S curve, which sounds smooth on speech.
- Ripple editing (Edit > Ripple Delete) closes the gap when you delete, so the rest of the episode slides left.
- Snap (Edit > Snap) pulls edges to clips, the playhead, markers or ruler ticks.
- Undo and Redo cover every edit. A fader drag or a recorded automation take is one step.

**Closing gaps.** Click an empty space between two clips and it lights up. Press Delete (or use Edit > Ripple Delete > Gap Between Clips) and the clips after it slide left to close it. Undo puts everything back.

![Click the gap between clips and press Delete: everything after it slides left](ripple-gap)

**Ripple Delete a time selection.** Drag across a stretch of time, then choose one of the Ripple Delete commands. Time Selection in Selected Clips (Alt+Backspace) cuts that stretch out of only the clips you selected and slides the later clips on their tracks left. Time Selection in Selected Track and in All Tracks cut whole tracks.

**Sync Lock** keeps tracks in step. The chain-link button in a track's header turns it on or off. When you ripple on a track with Sync Lock on, every other track with Sync Lock on closes the same stretch, so music and voices do not drift apart. A track with it off is left alone. New sessions start with Sync Lock on for the first tracks; tracks you add later start with it off.

![With Sync Lock on, a ripple delete on Voice closes the same time on Music; with it off, Music is left behind](sync-lock)

If a Sync Lock track has audio in the space you are closing, the program stops and says so, because closing it would cut or misalign that track. Turn Sync Lock off for that track to close the gap on one track only.

Markers (Edit > Add Marker) can mark cues, chapters, ad slots and social clips. Chapters and ad slots can be exported from File > Export > Markers and Chapters.

In the Markers panel, click a marker to pick it and jump to it. Ctrl+click adds or removes one, and Shift+click picks a run. Edit > Markers > Rename Selected Marker, Delete Selected Markers and Delete All Markers work on the pick, and Undo brings them back.

## Voice Chain and effects

The **Voice Chain** panel (Window > Voice Chain) is a row of effects that every track, bus and the master can have. Add effects from the Effects menu. Reorder effects by dragging, and switch one off to compare. The chain also has input gain, output gain and a Mix control.

Presets are in the Voice Chain's Presets menu. Presets you save appear under Yours.

The built-in effects cover the usual spoken-word jobs: noise reduction and noise print, de-essing, hum cut, compression, limiting, EQ, click and clip repair, and speech leveling. For more, plug-ins are described below.

A plain effect only changes what you hear in the session. To make an effect permanent on part of a file, open the file in the Waveform Editor and use Apply to Selection or Apply to Whole File.

## The Mixer, buses and routing

Open the Mixer from the Window menu if it is not showing. Every track and bus has a fader, a pan control, Mute, Solo and a level meter. The master strip is on the right.

![Host and Guest feed a Voices bus, a send adds a shared reverb bus, and everything reaches the master](bus-routing)

A **bus** is a track that other tracks feed. Add one with Multitrack > Add > Bus Track. A typical use is a Voices bus: send all voice tracks to it and put the compressor and limiter on the bus once.

In a track's header (taller headers show more) you can choose where its sound goes after its fader:

- **Master**: the normal mix.
- **A bus**: the track joins that bus.
- **None**: the track plays to nothing. It still shows its meter and its sends still work, but it is left out of the mix and out of exports. Use it for a track that only feeds a bus through a send.

**Sends** copy a little of a track to a bus at their own level, for example to a shared reverb. A send can be before or after the fader.

When tracks go through buses or the master has effects, the program mixes a short moment ahead. A fader move can then take about half a second to be heard. In a session with no routing, faders act at once.

## Automation

Automation moves a setting over time with points on a lane under the track. Show the lanes with the track's lanes button. A lane can move Volume, Pan, or any control of a built-in effect on that track or bus.

The mode box in the track header decides what happens:

- **Off** plays no lanes. Everything holds still.
- **Read** plays the lanes.
- **Write** records your moves from the moment you press Play to the moment you stop.
- **Latch** starts recording at your first touch of a control and keeps going until you stop.
- **Touch** records only while you hold a control. When you let go, the lane takes over again.

To record, set the mode, press Play, and move a fader, a pan control, or a slider in the Voice Chain. When you stop, the moves become points on the lane, and the controls go back to where they were. The whole take is one undo step named Record Automation.

Recorded moves are thinned so lanes are not flooded with points. Settings > Playback and Recording has a slider for how much. Zero keeps every point.

Buses have the same mode box. Plug-in controls and the master strip cannot be recorded yet.

## Hardware controllers

A MIDI fader box or control surface can run the faders, pans, mute, solo, arm and transport. Connect it, then open Edit > Controllers.

1. Tick your device in the list. Press Refresh if it is not there. The line "Last message" shows what the program hears, so you can check the device is working.
2. For a Behringer BCF2000 or any Mackie Control, set the device to its Mackie Control mode and press Use Mackie Control Layout. Eight faders run tracks 1 to 8, the ninth runs the master, the knobs set pan, and the buttons arm, solo and mute. The bank buttons move to the next eight tracks, and the transport buttons play, stop and record.
3. For anything else, pick what you want to run (for example Track 3 Volume), press Learn and move the control. Check Endless knob for a knob that sends how far it turned.

Faders recorded in Write, Latch or Touch mode work with a real fader. A Mackie fader with a touch sensor grabs the moment you touch it. Other faders count as let go a moment after they stop moving.

Motorized faders do not follow the screen yet, so after you change tracks the faders sit where your hands left them. Connecting a device sends nothing to it.

## Plug-ins

The program loads VST3 plug-ins. Open Effects > Plug-in Manager to see what was found and to add folders to scan. Plug-ins appear in the effect menus next to the built-in ones.

A plug-in's own window opens from its slot in the Voice Chain. What you change there is saved with the session. Heavy plug-ins can be slow to start. If one stops responding, the program switches it off and tells you instead of freezing the sound.

## Waveform Editor

Open a file with File > Open, or send a clip over with Clip > Open in Waveform Editor. Select a stretch of audio and use the Effects menu to Amplify, Normalize, Fade In, Fade Out, Invert, Reverse or Silence it. The Spectrum view (View > Show Spectrum) shows frequency over time, with selection tools to reduce, remove or repair a marked area.

Edits here change the file when you save it. Changes to a file used by a session show up in the session once saved.

New audio files you make (File > New Audio File, Paste to New File, Mix Down to New File) appear at the top of the Files panel until you save them.

## Exporting

File > Export has these choices:

- **Session Mixdown** renders the whole session to one file (WAV, MP3, FLAC and more, depending on your options).
- **Time Selection** exports just the selected stretch.
- **Track Stems** writes one file per track.
- **Markers and Chapters** writes your chapters and cues.
- **Multitrack Session (.sesx)** writes an Audition session file. Routing and some effects may be left out, and the export tells you what.

Check Loudness (File > Check Loudness) measures your episode against a target. Loudness Match (Effects > Loudness Match, or right-click a clip and choose Match Loudness) sets clip gain so each selected clip, or each track when none are selected, reaches a goal. Match by Loudness (LUFS), Peak, True Peak or Total RMS. The Tolerance box leaves alone anything already within that many dB of the goal, so a clip that is close enough is not touched. Rows that would peak over full scale start unticked, and applying is one undo step.

Each session can have its own loudness target. Set it in Multitrack > Session Properties under Loudness target, and Check Loudness uses it for that session instead of the choice in Settings. It is saved with the session, so a session template made from it (File > Save as Session Template) starts every new episode with the same target. Leave it on "Use my Settings choice" to keep one target for everything.

## Files, folders and settings

Open Settings with Edit > Settings. Pages cover audio hardware, playback and recording, appearance, colors and where files go. The cache, recordings and other working files each have a place you can change.

The Files panel lists the media in your session. Collect Session Media (File > Collect Session Media) copies everything a session uses into one folder, which is the safe way to move a session to another computer.

Keyboard shortcuts are listed in Help > Keyboard Shortcuts, and you can change them in Edit > Keyboard Shortcuts. They follow Audition's wherever the same command exists.

## When something goes wrong

- **Help > Report a Problem** opens the issue page on GitHub.
- **Help > Copy Bug Report Details** copies the version, your system, which editor is showing, the plug-ins in use and the last errors. Paste it into your report. Your user name is hidden, and nothing is sent anywhere by the program itself.
- If a plug-in misbehaves, remove it from the chain, or save, restart and try again without it.
- If the program closed unexpectedly, it offers to recover your unsaved work the next time it starts.

Help > Altered Audio Editor Github has the source code. Help > Support Altered Audio Editor shows ways to support the project.
