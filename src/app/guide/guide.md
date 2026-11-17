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

Opening files: one audio file opens in the Waveform Editor as itself, and two or more build a Multitrack session. A video file opens as a picture to cut against (see Video). With two or more sessions open, a tab for each runs along the top of the Multitrack. File > Open Recent keeps sessions and audio files. Sessions saved by other editors open too: File > Import Session (.sesx) reads Audition's session files, and File > Import Session (.rpp) reads projects from another editor, bringing tracks, clips, fades, markers and regions (plug-ins, sends and automation are left out, and the notice says what was left behind).

New sessions can start from a template. File > New Session offers Two-Host Show, Solo Voiceover and Audiobook, and File > Save as Session Template makes your own. Templates are ordinary: rename, change or delete them.

If you open a session made by an older version, nothing changes until you save. Saving upgrades it and keeps the original in the Backups folder, named with the old version, and a notice says so. Older versions of Altered Audio cannot open a session once it is upgraded. If the backup cannot be made, the save is stopped and your file is left as it was.

## Privacy

There is no account, no sign-in, no tracking and no telemetry. Your audio and sessions stay on your computer. The program goes online only when you choose Help > Check for Updates or open a link from the Help menu. There is also a weekly update check that is off until you turn it on in Settings.

## Recording

Add a track with Multitrack > Add > Mono Track or Stereo Track, then arm it with the **R** button in its header. Pick the input in the track header. Press Record (Multitrack > Record) and press Stop when you are done. The take lands on the armed track as a new clip at the playhead.

Recording Options (Multitrack > Recording Options) holds pre-roll, punch-in and the recording offset. With a time selection, Record starts a pre-roll before it, stops by itself at the end and replaces what was there on the recorded tracks (punch-in). The offset trims a few milliseconds from the start of each take so takes line up with what they were played against.

To record a call, each output device is also offered as an input named "System audio" followed by the device. It records what the computer plays, while microphones record on their own tracks.

**Marking while you record.** Three quick marks drop a numbered marker at the playhead: Add Retake Marker (F9), Add Cut This Marker (Shift+F9) and Add Great Bit Marker (Ctrl+F9). They work in the Multitrack and in the Waveform Editor, and every key can be changed in Keyboard Shortcuts. Window > Review lists your Cut this, Retake and Great bit marks in time order, with Previous and Next to step through them and a filter to show one kind.

**Scrubbing.** Settings > Playback and Recording has Play audio while scrubbing. With it on, dragging the playhead along the ruler plays what is under it. It is approximate: playback restarts at the pointer every moment. It is off by default.

**Playback speed.** Settings > Playback and Recording has Playback speed (experimental). With it on, Transport > Playback Speed plays the mix from 0.5x to 2x with the pitch kept, which is handy for reviewing speech. It is for listening only: recording, exports and saved files are not affected. At 0.5x and 2x the sound is smeared, the metronome is silent whenever the speed is not Normal, and it does not apply to 3D Space sessions heard on speakers. Recording puts it back to Normal, and so does starting the program again.

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

Press **I** to set the start of the time selection at the playhead and **O** to set the end. Shift+I and Shift+O clear them. With no end set the selection runs to the end of the content. They work in both editors.

Markers (Edit > Add Marker) can mark cues, chapters, ad slots and social clips. Chapters and ad slots can be exported from File > Export > Markers and Chapters.

In the Markers panel, click a marker to pick it and jump to it. Right-click a marker flag on the ruler, or a row in the panel, for Go to Marker, Rename, Convert to Range or Point, Change Marker Type and Delete. The panel has End and Description columns, a search box and resizable columns. Ctrl+click adds or removes one, and Shift+click picks a run. Edit > Markers > Rename Selected Marker, Delete Selected Markers and Delete All Markers work on the pick, and Undo brings them back.

**Track Visibility.** Multitrack > Track Visibility hides tracks from the Editor and Mixer lists without stopping them from playing. Save Visibility Preset remembers which tracks are hidden under a name, and Load Visibility Preset brings it back. Presets match tracks by name, so a preset made for one episode works in the next one. The newest eight are kept.

**Groups.** Group Clips (Clip > Group) ties clips together so clicking one picks them all and they move as one. Suspend Groups (Ctrl+Shift+G) turns that off for a while so you can pick a single clip without breaking the group. Remove Focus Clip from Group takes just the clip you clicked last out of its group, and Ungroup Selected Clips takes out every clip you have picked.

**Meter Input Signal** (Multitrack > Meter Input Signal, Alt+I) shows the input level on every track, not only the armed ones, so you can check a mic before you arm it.

**Host profiles and Guest Track.** Multitrack > Host Profile gives a voice track a starting Voice Chain set for how that person sounds: Close Mic, Standard Mic or Remote Guest. It replaces the track's chain in one undo step, and Clear Profile takes it away. Remote Guest uses stronger noise and room cleanup, a gentler gate and a low-pass that keeps call fizz out above 7 kHz, and marks the track as a guest track. Multitrack > Add > Guest Track adds a mono track named Guest with that profile already on, ready for a guest's recording.

## Takes and comping

When you record a line more than once, put the takes on one track over the same stretch, select those clips and choose Clip > Make Takes from Selected Clips. They become a set of takes: only one plays, and the others stay on the track, drawn dim. The **Takes** panel (Window > Takes) lists each set.

To comp, make a time selection over the part you like and press **Use over selection** beside that take. The editor cuts the takes at the edges of the selection and switches which one plays, as one undo step. With no selection, **Use** plays that take over the whole stretch. A take is cut back to the audio it actually has, so nothing goes silent by accident. Each piece is an ordinary clip, so you can still move, trim, fade or split it.

**Flatten** removes the takes that are not playing and turns the rest into ordinary clips. **Delete** removes one take. Settings > Multitrack turns the take commands off if you never use them; sessions that already have takes still play the chosen ones. Mixdowns and .sesx exports leave out takes that are not playing. A session that has takes plays every take at once in older versions of the program.

## Multi-mic shows and double-enders

These tools are for a conversation recorded with one mic per person, or a call recorded twice. Each works on the whole session and is one undo step.

- **Reduce Bleed** (Multitrack > Reduce Bleed) cleans one mic's track against another's, taking out the other voice that leaked in while keeping the voice on this mic, even when both talk at once. Each clip is cleaned into a copy in a Bleed Reduced folder beside your session. Your originals are not changed. Run it before Auto Mic Mix, and use Sync Tracks first if the tracks are not in step.
- **Auto Mic Mix and Talk Time** (Multitrack > Auto Mic Mix and Talk Time) finds who is talking and writes a volume lane on each mic that turns it down while someone else speaks. When nobody talks every mic stays open, so the room sound stays even. The same measurement reports how long each voice spoke. The lanes are ordinary automation you can edit.
- **Sync Tracks** (Multitrack > Sync Tracks) lines up two recordings of the same conversation, such as a call recording against a local mic. It reports how far the second track trails the first and how fast their clocks drift apart, then moves the second track into line. It needs at least two minutes of shared audio. It measures drift but does not correct it yet.
- **Close Dead Space** (Multitrack > Close Dead Space) shortens every stretch where no track has audio to a pause you pick. Everything after moves left together, so tracks stay in step.
- **Find Isolated Short Clips** (Multitrack > Find Isolated Short Clips) lists stray bumps, coughs and breaths left behind as tiny clips. You tick the ones to remove.

## Recording a remote guest

There is no built-in call feature, and nothing in the program sends audio over the network. For a remote guest the most reliable method is a double-ender: you and your guest talk over any call tool you like, and each of you also records your own microphone locally.

1. Each person records their own voice on their own computer, at the same sample rate, ideally as WAV. Record in this program or any recorder you trust.
2. Start both recordings before the call begins, and clap once at the start so the two files share an obvious sound.
3. After the call your guest sends you their file. Import it on a second track under your own.
4. Run Multitrack > Sync Tracks to line the guest up with your track. It needs at least two minutes of shared audio. Check the reported drift: if the two computers' clocks disagree, a long episode can slip by a few milliseconds, which Sync Tracks reports but does not correct yet.
5. If the call recording or a room mic picked up the other voice, run Reduce Bleed, then Auto Mic Mix and Talk Time. Close Dead Space tidies the pauses.

Your guest's local file is clean, so it sounds better than the call itself. Keep the call recording as a backup, and as the guide track if the guest forgets to record. To capture the call on your side, use the "System audio" input described under Recording.


## Process chains

A process chain is a small text file that lists steps to run on a session in order. Multitrack > Run Process Chain runs one as a single undo step. A chain is checked before anything changes: a step that is unknown, out of range or names a preset or track that does not exist stops it with a message saying which. A step that fails takes back what already ran, unless the chain marks it to be skipped.

The steps are Auto Mic Mix, Close Dead Space, Loudness Target (sets the Master so the mix reaches a target, and stops short rather than push peaks past its limit), Voice Chain Preset (gives a track or every track a preset by name) and QuickPass (runs a preset that removes or turns down what it finds). Example chains are in the docs/chains folder that comes with the program.

## Music, stretching and fitting

Select a clip and open the **Properties** panel. Stretch changes a clip's speed and pitch, with a mode of Off, Real-time or Rendered (HQ), and can fit a clip to the time selection. Remix is for music: set a target length and it finds up to three ways to cut or repeat the music where it lines up, shows the length you would get, and puts the one you pick on the track with short crossfades as one undo. Source Channels sets how much of each channel of the file goes left and right. Clip > Time Stretch and Clip > Fit Music to Length open these sections. Remix applies once: change the length later and you find joins again.

## Soundboard

The Soundboard panel (Window > Soundboard) has nine pads, each holding one sound such as a jingle, a stinger or an ad read. Press an empty pad to choose a file for it. Press a pad that has a sound and it goes on the session at the playhead, on the selected track (or the first track), as one undo step. Change and Clear sit under each pad.

A pad puts the sound on the session. It does not play live over the transport, so press Play to hear it. Each pad is also an action under Multitrack > Soundboard, so you can give it a key in Edit > Keyboard Shortcuts. The pads are saved with the program, not with a session. If a pad's file has moved, pressing it says the file could not be read.

## Voice Chain and effects

The **Voice Chain** panel (Window > Voice Chain) is a row of effects that every track, bus and the master can have. Add effects from the Effects menu. Reorder effects by dragging, and switch one off to compare. The chain also has input gain, output gain and a Mix control.

Presets are in the Voice Chain's Presets menu. Presets you save appear under Yours.

The built-in effects cover the usual spoken-word jobs: noise reduction and noise print, de-essing, hum cut, compression, limiting, EQ, click and clip repair, and speech leveling.

The **Speech Denoiser** (under Restoration) takes steady background noise out of a voice, such as fans, hiss and air conditioning. It has one control, Amount, from 0 to 100 percent, starting at 80. Use it on speech only: on music it thins the sound. **Speech Denoiser (High Quality)** is a larger network for harder noise, such as keyboards, traffic and a faraway second voice. It uses more of the processor and adds about 40 ms of delay, which the session makes up for, and its Amount starts at 100. The **Limiter** has a True peak switch that holds the peaks between samples under the ceiling, which matters for MP3 and AAC encoders and phone speakers. **Match EQ** can match another track or a neutral voice target, which builds a gentle EQ that moves no band more than 6 dB. For more, plug-ins are described below.

**QuickPass** (Effects > QuickPass) scans your clips for silences, short noises such as coughs and bumps, clicks and clipping, and lists what it found before changing anything. Its Remove Silence scan is the quick way to tighten an episode: pick a preset, set how quiet is silent and how long it must last, and choose what to do with each one. Remove cuts it out and can close the gap on that track or on every track, Split cuts the clip in two at each end, Turn Down only lowers it, and Mark adds a range marker. Remove keeps a short pad at each end so words do not run together, and Undo puts everything back in one step.

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

**Mix Role** is a label on a track: right-click a track header and choose Mix Role. Foreground is for voices, Ducked is for music that should drop while a Foreground track talks, and Background is a label for sound that sits low under everything (nothing reads it yet). Multitrack > Duck Music Under Voice uses the labels: with nothing selected, or a Ducked track selected, it ducks every Ducked track under the Foreground tracks. If a track that is not marked Ducked is selected, it ducks just that one. With no roles set it works on the selected track and listens to the voice tracks, as before.

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

Settings > Automation says where scripting stands. Process chains work now (Multitrack > Run Process Chain). Scripting and a local endpoint are planned after 1.0, and nothing listens on your computer.

Buses have the same mode box. Plug-in controls and the master strip cannot be recorded yet.

## Hardware controllers

A MIDI fader box or control surface can run the faders, pans, mute, solo, arm and transport. Connect it, then open Edit > Controllers.

1. Tick your device in the list. Press Refresh if it is not there. The line "Last message" shows what the program hears, so you can check the device is working.
2. For a Behringer BCF2000 or any Mackie Control, set the device to its Mackie Control mode and press Use Mackie Control Layout. Eight faders run tracks 1 to 8, the ninth runs the master, the knobs set pan, and the buttons arm, solo and mute. The bank buttons move to the next eight tracks, and the transport buttons play, stop and record.
3. For anything else, pick what you want to run (for example Track 3 Volume), press Learn and move the control. Check Endless knob for a knob that sends how far it turned.

Faders recorded in Write, Latch or Touch mode work with a real fader. A Mackie fader with a touch sensor grabs the moment you touch it. Other faders count as let go a moment after they stop moving.

Motorized faders can follow the screen. Tick **Send feedback to the devices** for a control surface in its Mackie layout and the faders move when you move one with the mouse, undo, or change banks. Mute, solo and arm lights follow the tracks and the pan rings show each pan. A fader you are touching is left alone. This is experimental: it was built from the Mackie Control message layout and has not been tried on every surface, so reports are welcome. Nothing is sent to a device unless the option is on and the device is ticked. Faders do not follow automation while a session plays.

## Plug-ins

The program loads VST3 plug-ins. Open Effects > Plug-in Manager to see what was found and to add folders to scan. Plug-ins appear in the effect menus next to the built-in ones.

A plug-in's own window opens from its slot in the Voice Chain. What you change there is saved with the session. Heavy plug-ins can be slow to start. If one stops responding, the program switches it off and tells you instead of freezing the sound.

## Waveform Editor

Open a file with File > Open, or send a clip over with Clip > Open in Waveform Editor. Select a stretch of audio and use the Effects menu to Amplify, Normalize, Fade In, Fade Out, Invert, Reverse or Silence it. The Spectrum view (View > Show Spectrum) shows frequency over time, with selection tools to reduce, remove or repair a marked area. Effects > Spectrum > Play Selected Area plays only the sound inside the selection, so you can hear what you are about to remove, without changing the file. The selection can be up to five minutes long.

Drag the level knob in the corner of the view to turn the selection, or the whole file, up or down. The waveform redraws at the new height while you drag, so you can judge the level by eye, and the change applies when you let go. Effects > Fill Single Click smooths over one selected click, up to 25 ms, from the audio on each side. Select the next click and press Shift+R to repeat it, or use Ctrl+Alt+K.

Edits here change the file when you save it. Changes to a file used by a session show up in the session once saved.

New audio files you make (File > New Audio File, Paste to New File, Mix Down to New File) appear at the top of the Files panel until you save them.

## Video

File > Open takes video files (MP4, M4V, MOV, WMV, AVI, and MKV where the system can show them). The video becomes the session's reference: the **Video** panel (Window > Video) shows the picture at the playhead with a timecode, and a Video Reference bar under the ruler follows the zoom. Drag the bar to change when the picture starts. The picture is a reference to cut against, not a frame-accurate player. The session remembers which video it uses and where it starts. If the video had sound and the session is empty, the sound goes onto the first track.

Pictures need Windows for now. On Mac and Linux the Video panel says so, and a video's sound still imports. The video file itself is not changed or mixed in.

**Mixdown Into Video** (File > Export > Mixdown Into Video) mixes your session from the video's first picture on and writes a copy of the video with that mix as its sound. The picture is copied without re-encoding, so it is fast and loses nothing. It works for H.264 files in MP4, M4V and MOV, on Windows.

## Exporting

File > Export has these choices:

- **Session Mixdown** renders the whole session to one file (WAV, MP3, FLAC and more, depending on your options).
- **Time Selection** exports just the selected stretch.
- **Track Stems** writes one file per track.
- **Markers and Chapters** writes your chapters and cues.
- **Edit Lists** writes your cut for other editors: EDLs (one file per track), a marker list, or an OpenTimelineIO file. Choose the frame rate and starting timecode. Fades, automation and effects stay behind, and the dialog lists what could not be carried.
- **Multitrack Session (.sesx)** writes an Audition session file. Routing and some effects may be left out, and the export tells you what.

Very long recordings are fine: WAV exports and multichannel exports switch to RF64 by themselves past 4 GB, and RF64, BW64 and Wave64 files open.

Check Loudness (File > Check Loudness) measures your episode against a target. Loudness Match (Effects > Loudness Match, or right-click a clip and choose Loudness Match) sets clip gain so each selected clip, or each track when none are selected, reaches a goal. If a match would push a track over its peak limit, it can add a True Peak limiter at that limit instead of leaving the row unticked. Match by Loudness (LUFS), Peak, True Peak or Total RMS. The Tolerance box leaves alone anything already within that many dB of the goal, so a clip that is close enough is not touched. Rows that would peak over full scale start unticked, and applying is one undo step.

Each session can have its own loudness target. Set it in Multitrack > Session Properties under Loudness target, and Check Loudness uses it for that session instead of the choice in Settings. It is saved with the session, so a session template made from it (File > Save as Session Template) starts every new episode with the same target. Leave it on "Use my Settings choice" to keep one target for everything.

**Batch Convert.** The Batch Convert panel runs each file in its list through a chain of steps and converts it to the format you choose. The steps are a Voice Chain preset, trim, loudness and peak limit, and fades. The Loudness step takes each file to a LUFS target with a peak ceiling. Pick a starter chain, or save your own.

## Panels and workspaces

Every panel is in the Window menu. A **workspace** is a saved arrangement of panels. Window lists Default, Focused Edit, Mixing, Voiceover, Music and Review and Export, plus any you save, with commands to save changes, save the arrangement under a new name, reset the current one and delete it. Built-in workspaces cannot be deleted. Drag a panel's tab to move it, and Reset puts the current workspace back the way it was.

View > Podcast Mode switches to the Default workspace and turns Music Mode off. View > Voiceover Mode switches to the Voiceover workspace, with takes and markers on the left, and also turns Music Mode off. Music Mode (View) shows the MIDI and bars tools, and the Music workspace turns it on.

**Levels** shows the output level, left and right, on a scale from -60 to 0 dB. It follows whatever is playing: the session, the Waveform Editor's file, or a preview from Files, Drive Manager or Libraries. Click a red over light to clear it.

**Time Info** shows where the playhead is, the time selection (or the span of the selected clips), the visible range, and the session's length, format and mixdown size. The playhead, selection and view times can be dragged or typed. In the Waveform Editor it shows the open file instead.

**History** lists every edit since the session opened. Click a row to return to that point. Rows after it dim until a new edit replaces them.

**Metadata** edits the tags inside WAV, FLAC, MP3 and M4A files. Add files, search the list, edit one file's fields, set cover art, and keep named tag sets (a show's artist, album and genre) to put on many files at once. Only filled-in fields are written, so chapters and the audio stay as they are. A file open in the Waveform Editor is refused.

**Drive Manager** browses drives and folders and previews audio. Bring files into the session by double-click, with Import, or by dragging rows onto the Files panel.

**Playlist** keeps range markers in an order you choose. Play runs them back to back so you can hear an arrangement, such as a highlight reel or a set of promos, without moving any clips. Export writes each one as its own file.

**Phase Wheel** shows how two signals of what is playing line up, frequency by frequency. Each dot is a band of frequencies. A dot straight up means the two move together, straight down means they are opposite and would cancel in mono. The bar under the wheel is the overall correlation: +1 the same, 0 unrelated, -1 opposite.

**3D Space** shows the room from above with the layout's speakers and a dot for each track and bus. Drag a dot to move it, and use the wheel over a dot to raise or lower it. The bar on top picks the speaker layout and whether you hear it on speakers or headphones.

## Everyday tools

**Generate** (Effects > Generate) has Tone, Noise and Silence. Tone and Noise make a new audio file and place it as a clip at the playhead on the selected track. Silence makes room at the playhead: everything after it moves later, on every track or only the selected one, and a clip under the playhead is split.

**Fill with Room Tone** (Effects, Waveform Editor) replaces the selected stretch with room tone built from the quietest second elsewhere in the file. It needs a quiet stretch to copy, quieter than -40 dBFS, and Undo takes it back.

**Speaker Balance** (Multitrack > Speaker Balance) brings every voice track to the same loudness in one click, before the Voice Chain, so gates and compressors see each speaker at a like level. The quietest voice is the reference and the others come down to it, so nobody is pushed into clipping. It is applied as clip gain in one undo step, and music, sound effect and ambience tracks are left alone.

**Audiobook delivery check.** Check Loudness has an Audiobook (ACX) choice. It checks the three numbers narration is judged on: average level between -23 and -18 dBFS RMS, peaks no higher than -3 dBFS, and a noise floor no louder than -60 dBFS RMS. Confirm the figures against the retailer's own page before you submit. A pass here is a good sign, not a guarantee.

## Files, folders and settings

Open Settings with Edit > Settings. Pages cover audio hardware, playback and recording, appearance, colors and where files go. Every setting you see does something. Channel Routing has a Mono option that plays both sides together on one output, for a single speaker. The cache, recordings and other working files each have a place you can change.

The **Libraries** panel (Window > Libraries) holds a show's reusable sounds, such as intros, ads, promos, beds and legal reads, sorted into colored bins. Drop files on it to copy them in, drag items onto a bin to file them, or drag them out onto tracks. You can tag items, several at once. Click the show badge to set its color and cover picture. Closing a library only hides it, and Remove Library from List never deletes files. Find Libraries in Folder adds every library it finds, and Export and Import Library List moves your list to another computer (the audio is copied separately).

The Files panel lists the media in your session. Multitrack > Insert Session as Clip mixes another saved session down and places it at the playhead as one clip on a new track, handy for a theme or a reused segment. The clip is a copy as the session was saved: later changes to that session do not reach it.

Collect Session Media (File > Collect Session Media) copies everything a session uses into one folder, which is the safe way to move a session to another computer.

Keyboard shortcuts are listed in Help > Keyboard Shortcuts, and you can change them in Edit > Keyboard Shortcuts. They follow Audition's wherever the same command exists.

## When something goes wrong

- **Help > Report a Problem** opens the issue page on GitHub.
- **Help > Copy Bug Report Details** copies the version, your system, which editor is showing, the plug-ins in use and the last errors. Paste it into your report. Your user name is hidden, and nothing is sent anywhere by the program itself.
- If a plug-in misbehaves, remove it from the chain, or save, restart and try again without it.
- If the program closed unexpectedly, it offers to recover your unsaved work the next time it starts.

Help > Altered Audio Editor Github has the source code. Help > Support Altered Audio Editor shows ways to support the project.
