# Decisions log

Newest first. Each entry: date, decision, reason.

## 2026-10-14: Formats in and out, Batch Convert, Export Mixdown dialog, per-speaker tracks, system audio, bulk trim, track meters

Decision:
- Opening: Opus (`.opus`, Ogg Opus) now decodes through the pure-Rust `opus-decoder` crate (MIT or Apache-2.0), wrapped as a decoder in our codec registry. Checked against every format a common call recorder exports: FLAC, WAV (16-bit, 8-bit, MS and IMA ADPCM), AAC (M4A), ALAC, Ogg Vorbis, Ogg FLAC and Opus open. HE-AAC does not: the decoder handles AAC-LC only. Audacity 3 projects (`.aup3`) are a possible later import.
- Writing: one encoder module writes WAV (16, 24, 32-bit float), FLAC (16, 24; pure-Rust `flacenc`, Apache-2.0, streamed frame by frame so long episodes never sit in memory) and MP3 (LAME 3.100 through `mp3lame-encoder`, LGPL, built from source; constant 32 to 320 kbps or variable quality 0 to 9). It converts channels and sample rate on the way, keeping the exact length. Mixdowns, Batch Convert and Waveform saves all use it.
- Waveform editor: FLAC and MP3 files save in place in their own format (no conversion needed). Saving an MP3 in place, or Save As to MP3, warns that MP3 re-encoding loses quality. Formats we can read but not write (M4A, Ogg, Opus) go through Save As. Save As offers WAV, FLAC and MP3.
- File > Export > Session Mixdown opens our Export Mixdown dialog: what to mix (whole session, time selection, selected clips), name and folder, format, sample rate, channels, bit depth or MP3 bit rate, a summary of what the file will be with its rough size, and "Add the file to the Files panel". It remembers the last format and folder and asks before replacing a file. Include markers and metadata is shown disabled until metadata export exists.
- Batch Convert panel (the left column tab): a list of files (Add Files, Add Folder, add the Files panel's selection, or drop files, folders or zips), naming with text before and after the name, output next to each original or into one folder, Replace existing files, the same format controls, and Remove finished files from the list. Converts one file at a time on a worker; each row shows progress, Done, or why it failed, and the rest carry on. Stop discards only the file in progress. Every file is written under a temporary name and moved into place when complete, so a stopped or failed file never leaves a partial file and converting a file onto itself is safe. Rows that would write the same file, or replace their own source without Replace on, are flagged and skipped.
- Multitrack > Insert Files as Tracks: each file goes on its own new track at the session start, named after the file without a leading speaker number ("1-name.flac" becomes "name"). Zips (a call recorder's per-speaker download) are extracted into a folder beside the zip first; only audio is extracted, every file lands directly in that folder, entries with paths that climb out are skipped. Importing or dropping a zip anywhere does the same. Insert Selected Files as Tracks does it for the Files panel's selection.
- Recording a call: there is no direct link to any chat service (that would need an account and the network, which the privacy rules rule out). Instead, on Windows each output device is offered as an input named "System audio: device", recording what the computer plays (the call) while mics record on their own tracks. Windows sends nothing while the output is silent, so the recorder pads silence by the clock to keep the take in sync.
- Edge trims on a selected clip trim every selected clip by the same amount (the grabbed clip snaps; each clip stops at its own limits), as one undo step. Trim cursors are a bracket with an arrow.
- Track meters: left and right bars, green to -12 dB, yellow to -3 dB, red above, with a held peak line and a clip light per channel. Clip lights (track and Levels panel) stay lit after stopping until clicked.
- CI uses actions/checkout and actions/setup-node v5 (Node 24).

Reason: the project lead's shows record calls as per-speaker FLAC, edit in WAV and publish MP3; this removes the conversion steps around that and makes multi-speaker imports one action. The dialogs and panels follow the common behaviors of these tools; their layout, wording and names are our own.

## 2026-10-14: Track layout pass: collapse, resize, track scrollbar, meters, box select, live recording

Decision:
- Each track header has a collapse arrow (folds the track to its name row; saved as `collapsed`, left out of files when off) and a resize edge at the bottom (drag; one undoable step).
- A track scrollbar on the right of the lanes shows each track's color. Drag the thumb to scroll; drag either end of the thumb to make every track taller or shorter.
- A Zoom Out Full button at the end of the overview bar fits the whole session across and every track into the lanes' height. The overview window has a clearer border and grips on both ends.
- With the Move tool, dragging empty space draws a dashed box that selects the clips it touches (Shift adds); a click still moves the playhead. The Time Selection tool keeps the full-height time selection.
- A slim vertical meter sits at the right edge of each header: the input level while the track is armed (inputs open for metering as soon as a track is armed, without recording), playback level otherwise. While recording, each take draws its waveform on its track as it arrives.
- Razor tool: a razor blade cursor, and the cut line also marked in the ruler. Clips get a subtle raised edge; alternate lane shading is stronger.
- Volume and pan dials are larger with small symbols. The automation menu lists Off, Read, Write, Latch and Touch; only Read can be chosen until automation arrives (P5).
- The Editor tab names what it shows: "Editor - session name" (with a star when unsaved), or the file open in the Waveform editor.

Reason: long sessions with many tracks need folding, sizing and a way to see and reach every track; recording needs a level before and while you record. The look follows our own design; only the behaviors are common practice.

## 2026-10-14: Recording v1, spectral tools, pitch view, double-click to edit

Decision:
- Recording (moved up from P4 at the project lead's request): Record in the transport bar or the Multitrack menu. Every armed track records the input picked on its header into its own 32-bit float WAV, starting at the playhead, while the rest of the session plays. With no track armed, the selected track is armed. Takes go in a Recordings folder next to the saved session, or the app's cache folder until the session is saved. On stop, each take becomes a clip on its track and shows in the Files panel, as one undoable step. In the Waveform editor, Record inserts the take into the file at the playhead.
- Recording engine: one thread per input device. The device callback only copies samples into a lock-free ring buffer (two seconds deep); the same thread writes each track's channels to disk every 10 ms. Input strings from the picker ("Mono: device 1", "Stereo: device 1-2") route channels; a missing device gives a plain error.
- Not yet: input monitoring through the app, latency compensation, punch-in, metronome. The Monitor button saves its setting now.
- Audio Hardware... in the input picker opens the system's own sound settings (Windows Sound settings; pavucontrol or the desktop's sound panel on Linux; System Settings on macOS). It starts a local settings app only.
- Tool strip: Spectrum and Pitch view toggles, then Move, Razor, Slip, Time Selection, then four spectral tools: Box Select, Lasso Select, Brush Select (with a size slider) and Spot Repair. Spectral tools work in the Waveform editor; choosing one turns the spectrum on. Shift adds to a selection. The Slip icon now shows a clip with arrows so it reads clearly.
- Spectral selections are shapes in time and frequency. For an edit, the interface turns them into a grid of weights and the core applies Reduce, Remove or Repair inside it with softened edges. Repair fills each frequency band from the level just before and after the area, keeping the original phase; Spot Repair runs it as soon as the stroke ends.
- Pitch view draws the fundamental frequency (60 Hz to 1 kHz, difference-function method on audio low-passed at 1.1 kHz and thinned to about 11 kHz) over the spectrum; with the spectrum off, the spectrum stays as a dim backdrop. The line breaks at octave jumps.
- The waveform and spectrum share the Waveform editor with a draggable divider.
- Double-clicking a clip in the Multitrack (Move or Slip tool) opens its file in the Waveform editor.

Reason: recording a single host or a voiceover directly is basic to the job, and the spectral tools cover the repairs editors reach for most (clicks, coughs, whistles, phone buzz) without a separate app. Names are our own and describe what each tool does.

## 2026-10-14: Spectrum view v1, host profile names

Decision:
- Waveform editor: a Spectrum button (and View > Show Spectrum) shows frequencies over time under the waveform. Log frequency axis from 30 Hz to Nyquist, 2048-point frames, colors from near black through indigo, teal and amber to warm white over 100 dB. It renders the edit list, so it shows exactly what plays. The view covers up to 20 minutes at once; zoom in for longer files.
- Drag a rectangle on the spectrum to select an area of time and frequency. Effects > Spectrum (and a small bar over the selection) offers Reduce Selected Area 12 dB, Reduce Selected Area 24 dB and Remove Selected Area. The core cuts the stretch into overlapping frames, scales the bins inside the rectangle with soft edges, and rebuilds it; outside the rectangle the audio comes back unchanged. The changed stretch is written to a new file in the app's cache folder and swapped into the edit list as one undoable step. Saving renders it into the file as usual.
- Next for spectral editing: lasso and brush selections, and healing that fills a selection from the audio around it.
- Built-in host profiles are named for what they are: Close Mic (-14 dB), Standard Mic (-20 dB), Remote Guest (-30 dB, telephone EQ, heavier denoise).

Reason: removing a single sound (a whistle, a phone buzz, a chair squeak) without touching the voice under it is a common repair. A first testable version lets people try it early. Profile names should say what each one is for.

## 2026-10-14: Track header controls, timeline toggles, interface font, public repo cleanup

Decision:
- Track headers are their own raised panel, a step lighter than the lanes, with a dark gutter between the header column and the lanes and under each track. Headers are 214 px wide.
- Each header: drag grip, name, Mute, Solo, Arm and Monitor Input; round Volume and Pan dials (drag up or right, Shift for fine, double-click to reset); Invert Polarity, which flips the track's gain sign in playback and mixdown (stored as `polarity_inverted`, left out of files when off, so old files are unchanged); an input picker listing Default, None, every mono channel and stereo pair of the computer's audio inputs (names only, nothing is opened); and an automation mode menu shown disabled until track automation (P5). Rows hide on short tracks.
- Timeline toggles beside the header view strip: Metronome (disabled until P4), Follow Playhead (pages the lanes during playback, on by default, saved), Snapping (the existing setting) and Add Cue Marker. The sample rate moved to the editor toolbar.
- Interface font: Source Sans 3 (SIL Open Font License), bundled through `@fontsource/source-sans-3`, Latin regular and semibold only. Base size 13 px. Nothing loads from the network. Credited in ATTRIBUTION.md.
- Public repo cleanup before the first public release: personal names, private presets, notes about reference material and other products' feature names were removed from tracked files. Private notes live in `CLAUDE.local.md` and the gitignored `private/` folder. The repo history restarts from a clean first commit.

Reason: dense headers need to read as separate from the lanes and from each other, and the controls an editor reaches for most (input, monitor, polarity, dials) belong on the track. An open humanist font reads well at small sizes. The public repo should carry only original, shareable material.

## 2026-10-14: Track separation, Loop Playback, Skip Selection, Files preview, app icon, README previews

Decision:
- Multitrack lanes: a 3 px dark break under every track, alternating row shades, and a clearly lighter shade across the whole selected track (header and lane). Each header has a 5 px color strip in the track's color; tracks with no color take the clip palette by row, and their clips do too.
- Loop Playback (transport bar) repeats the time selection; Skip Selection plays around it, so a cut can be heard before it is made. They are one setting with three states (normal, loop, skip), saved between runs. The renderer jumps at the boundary, sample-accurate, with no fade at the seam. Starting inside a skipped range starts after it; starting after a loop plays straight on. Changing the mode while playing restarts from the same spot.
- Zoom row: Zoom In/Out Amplitude now in the Multitrack too (display only, up to 16x; Alt + wheel as in the Waveform editor), plus Taller and Shorter Tracks, which change every track's height in one undoable step.
- Above the track headers: a header view strip. Mix (the current controls) is the only view built; Inputs/Outputs, Effects, Sends and EQ show disabled with the phase that builds them (P4, P5).
- Files panel: Play/Stop Preview, Loop Preview and Auto-Play (preview a file when selected) under the list. Loop and Auto-Play are saved. No Space shortcut was added to the Files list, because new shortcuts need the project lead's approval; Libraries already has one.
- Voice Chain items (Clip vs Track Effects, the presets row, input and output trim) are logged for P5, not built now.
- App icon: `npx tauri icon brand/icon/icon-1024.png` for every size, then `brand/icon/icon.ico` copied over `src-tauri/icons/icon.ico` to keep the simpler glyph at small sizes. The About dialog shows the logo.
- README: the logo (light and dark), two preview images of the app from a demo session made of generated audio (no real show audio), feature and privacy highlights, and the donation link.

Reason: in a busy session tracks blurred together and the selected track did not stand out. Loop, skip and the full zoom row are part of a common podcast editing workflow. Placeholders follow the existing rule: unbuilt items are visible but disabled with the reason, never fake.

## 2026-10-14: New Session dialog, session templates, clip envelope chip

Decision:
- File > New Session asks for a name, a folder (default Documents/Altered Audio/Sessions), an optional template, sample rate, bit depth and mix (stereo or mono). With a folder, the session gets its own folder and is saved at once, so merged clips and copies have a home from the start; leave it blank to save later. A clash with an existing session reopens the dialog with the reason. The last-used folder, template, rate, depth and mix are remembered; New Audio File remembers its rate, channels and depth too.
- Templates: File > Save as Session Template saves the open session (tracks, levels, clips such as a show's intro and outro, markers, libraries) as a `.altemplate` in the app's config folder. New sessions made from a template take its rate, depth and mix, get their own id, and record which template they came from. Templates can be deleted from the dialog.
- Mix: Mono sets the master to mono, and mixdowns of the session are written mono unless an export asks otherwise.
- Each clip's title bar shows a small envelope chip ("Volume", "Pan", "Vol + Pan" or "Envelopes"); clicking it switches the volume and pan lines.
- Ctrl+A under a dialog no longer selects the page's text.

Reason: these are the dialog choices and clip header details an experienced editor expects. A show template is what makes a weekly episode a two-click start.

## 2026-10-14: Tool strip, tools, transport bar, six starting tracks

Decision:
- A strip under the menu bar switches between the Waveform and Multitrack editors and holds the tools: Move (default), Razor, Slip, Time Selection. Edit > Tool lists the same tools. Shortcuts R (Razor), V (Move), T (Time Selection) and Y (Slip), all approved by the project lead.
- Razor: click a clip to split it there, Shift splits every track at that time, Alt ignores snapping; a dashed guide shows the cut. Slip: drag inside a clip to move its audio while the clip stays put (loops change phase). Time Selection: drag anywhere, even over clips. In the Waveform editor only Time Selection applies.
- Both editors share one bar under the editor: position clock (click to change format), stop, play or pause, go to start, rewind and fast forward (5 s, Shift 1 s), go to end, record and loop (disabled until recording, P4), and zoom buttons (amplitude zoom in the Waveform editor).
- New sessions start with six empty stereo tracks, Track 1 to Track 6. Opening audio into an empty session puts each file on the next empty track.

Reason: a common podcast workflow needs quick tool switching and one transport for both editors. Layout and icons are our own; behavior matches what an experienced editor expects.

## 2026-10-14: Waveform editor design

Decision: the Waveform editor edits one audio file at a time, shown in the Editor panel in place of the multitrack lanes (View > Waveform Editor, or Clip > Open in Waveform Editor, or double-click a file in the Files panel). Internally the file is a second, private session in the core with one track whose clips are the edit list: the first clip covers the whole file, and every edit (delete, cut, paste, silence, gain, fades, crop) splits and rearranges those clips with the same ops, undo and renderer the multitrack uses. Nothing touches the file until Save, which renders the edit list (clip settings only, original channel count) to a temporary file beside the original and then replaces it. Save As writes a new file and leaves the original alone. Files that are not WAV always go through Save As. The Waveform editor has its own undo history; Edit menu commands go to whichever editor is showing.

Built (2026-10-14): double-click a file in Files (or Clip > Open in Waveform Editor, or View > Waveform Editor). Edits: Delete, Cut, Copy, Paste (replaces a selection, otherwise inserts at the playhead), Crop, Silence, Fade In, Fade Out (equal-power curve written into piece envelopes so fades cross earlier edits), Amplify, Normalize (peak target, measured by the core), and a quick level control: drag to raise or lower the selection or whole file. Level scale in dB on the right, amplitude zoom (Alt + wheel or the bar buttons), and a shared transport and zoom bar under the editor. The action registry gained overlays: while the Waveform editor shows, Edit, Save, Close, Zoom and play commands act on it. File menu labels change to Save Audio, Save Audio As and Close Audio File so it is always clear which document they act on.

Reason: one editing engine for both views means one set of rules and tests; the edit list keeps every step undoable without copying audio on each edit, and the file on disk only changes when the user saves. Overwriting a file changes every clip that plays it, which is what Make Unique Copy is for, and the save confirmation says so.

## 2026-10-14: Loop Clip

Decision: Clip > Loop Clip (a checked toggle) makes a clip repeat its source region. The model gains `looped: { length, phase }` on Clip: `length` is the clip's timeline length in session samples, `phase` is where the clip's left edge falls inside the region, in source samples. Looped clips trim right with no limit and trim left back to the session start; left trims, splits and ripple deletes move the phase so every piece keeps its place in the loop. Turning looping off plays the region once from the phase, so nothing jumps. The renderer seeks back to the region start at the end of each pass. The lane shows a dashed line at each repeat and a "Looped" tag. Optional field with a serde default, so older sessions load unchanged and schema_version stays 1.

Reason: music beds and room tone under long segments, without making copies of the file.

Seams: a 2.5 ms dip on each side of every seam inside the clip (not at the clip's own edges, which have their fades), so a region whose ends don't line up never clicks. Found by Check Loudness: a looped pure tone measured +1 dB true peak before the dip.

## 2026-10-14: Make Unique Copy

Decision: Clip > Make Unique Copy gives each selected clip (not locked) its own file: just the part of the source it plays, copied untouched at the source's rate and channel count as 32-bit float, into a "Clip Copies" folder beside the session. The clip keeps its id, position, gain, fades and envelopes; only its file changes. One undo step. Unsaved sessions are asked to save first.

Reason: gives a clip a file of its own before destructive work in the Waveform editor, so other clips sharing the source are never affected. Copying only the used region keeps copies small.

## 2026-10-14: Clip pan envelopes

Decision: View > Envelopes > Clip Pan shows a cyan line per clip, full left at the top, center in the middle, full right at the bottom. Same gestures as volume: press the line to add a point, drag to move, Shift locks time, right-click to delete a point or reset. A tip beside the pointer shows the value while dragging ("Pan L 40", "Volume -6.0 dB"). Pan is a balance, the same rule as track pan: centered plays both sides at full level. The pan envelope's id is the clip id with its lowest bit flipped, so add and remove round-trip exactly. Split, trim and ripple delete carry pan points with the audio, like volume.

Reason: P3 item. One shared envelope path for volume and pan keeps a single set of rules, tests and gestures.

## 2026-10-14: Merge Clips

Decision: Clip > Merge Clips takes two or more selected clips on one track (none locked), renders them with only their own settings (gain, fades, volume and pan envelopes; not the track fader, track pan, mute or Master) to a 32-bit float WAV in a "Merged Clips" folder beside the session, and swaps them for one clip at the first clip's start. One undo step; original files are never touched. Names never overwrite ("l Merged 2.wav"). An unsaved session asks to be saved first, since the merged file needs a home.

Reason: track settings still apply to the merged clip, so baking them would apply them twice. Float keeps any level above 0 dBFS intact.

## 2026-10-14: Public repo prep

Decision:
- Donation: Help > Support Altered Audio Editor opens https://paypal.me/scp21. `paypal.me` joins the link allow list. Nowhere else in the app.
- License stays `GPL-3.0-or-later`.
- The project's private host profiles and personal presets moved to `CLAUDE.local.md`, which is gitignored. Public built-ins are generic: Host, Co-host, Guest profiles; Breath Gate - Low (-14 dB, 2/50/200 ms) and Breath Gate - Medium (-20 dB, same timing). The host profile test fixture is `host.alhost`.
- Test data uses generic names (Host, Show) instead of real people and shows.
- GitHub: under the GeekCast Radio Network organization, repo for Altered Audio Editor.
- There is no separate spec doc. CLAUDE.md plus this file are the source of truth.
- CLAUDE.md intro now says it is a purpose-built tool, not a clone of any commercial product.

Reason: the repo goes public; personal voice settings stay private, while the settings that work still ship as neutral presets.

## 2026-10-14: Update checks, off by default

Decision: Help > Check for Updates always works when clicked. An automatic weekly check exists but is off by default (Preferences, and an unticked box on the first-run screen). Updates come from GitHub Releases via the Tauri updater, signed with the project's release key, fetched as a release file (not the GitHub API, which rate-limits at 60 an hour per IP). A found update is one status bar line; never a pop-up, never mid-playback or recording, never auto-installed. No launcher: one app doesn't need one.

Reason: bug fixes need a way to reach people, but the zero-network promise is the product. Off by default keeps "nothing leaves your machine unless you ask" true. Rejected: on by default (breaks the promise), auto-install (swapping a production tool mid-episode), a launcher with login (wrong for a single open-source app).

## 2026-10-14: Check Loudness (File menu)

Decision: File > Check Loudness measures the mix (or the time selection) with EBU R128 (`ebur128` crate): integrated, loudness range, true peak, sample peak, plus a mono fold-down. Targets, from published podcast loudness guidance: podcast stereo -16 LUFS, podcast mono -19 LUFS, Spotify -14 LUFS, broadcast -23 LUFS, all with true peak at or below -1 dBTP. Pass within 1 LU. When master gain alone reaches the target without breaking the peak ceiling, a one-click "Raise/Lower Master" button applies it as an undoable edit and measures again. When it can't, the advice points to limiting vocals (a limiter at -6 dB, or -3 dB, depending on the episode).

Reason: loudness is the most common podcast delivery mistake, and measuring the real mix (same renderer as export) means what you check is what you ship. Gain only, no hidden limiting: the Voice Chain Limiter (P5) owns peak control.

## 2026-10-14: Privacy, Help links, security hardening

Decision: zero logins, telemetry, tracking or network calls, written into CLAUDE.md, `PRIVACY.md` and the About dialog. Help menu gains "Book: Altered Audio, Podcasting Demystified" (Barnes & Noble) and "Support Altered Audio Editor..." (disabled until `LINKS.donate` is set). `scripts/check-privacy.mjs` runs in `npm run check` and fails the build on network APIs, tracking libraries, HTTP or telemetry crates, sockets in Rust, or a CSP looser than self. CI gains a security job: the privacy guard, `npm audit`, `cargo audit`. Library Clean Up now dry-runs and confirms, refuses folders that aren't the library's own, skips symlinks and non-audio files. Added `SECURITY.md`, `CONTRIBUTING.md`, issue and PR templates.

Reason: trust is the product for a free tool. A guard in the build keeps the promise true as contributors arrive, instead of relying on review.

## 2026-10-14: Clip volume envelopes

Decision: View > Envelopes > Clip Volume shows a yellow volume line on every clip, at 0 dB until it has points. Press on the line to add a point there; drag a point to move it in time (it stays between its neighbors) and level; hold Shift to change only the level. Right-click a point to delete it, or the line to reset the envelope. Each change is one undo step. The setting is saved with the editor settings.
- Levels run from +12 dB at the top of the clip to -60 dB at the bottom, and the bottom is silence. Between points the level moves in a straight line in dB; before the first point and after the last it holds. Playback and every mixdown follow it exactly (same plan and renderer).
- Points are stored on the clip in session samples from the clip start (the `envelopes` field that was already in the model). The volume envelope's id is the clip's id, so undo and redo give exactly the same session.
- Moving a clip moves its envelope with it. Trimming the left edge, splitting, and ripple deletes keep the points lined up with the audio, adding a point at each cut so the level there does not jump.
- Clip Pan envelopes and track automation come later.

Reason: the everyday use is ducking a music bed under a voice and riding a loud or quiet moment, without a separate effect.

## 2026-10-14: Libraries panel redesign: show libraries and bins

Decision (a livelier Libraries panel, organized per show with bins for Ads, Promos, Clips, Intro/Outro and so on):
- One library per show is the model (it already was). The panel header now shows the show: a colored badge with the show's initials, the name, and item and bin counts. Click the name to switch libraries or make a new one; click the badge to set the show color (Automatic picks a stable color from the name).
- Collections are called bins in the interface. The data stays `collections` in `library.json`, so nothing breaks. Bins gain an optional color, and libraries an optional show color (serde defaults; schema regenerated; no version bump).
- New show libraries start with podcast bins: Intro / Outro, Ads, Promos, Clips, Music Beds, Sound Effects, Legal Reads, each with its own color. Older libraries get them from Add Starter Bins (only the missing ones are added).
- Bins show as a strip of chips under the header, each with a color dot and a count, plus All. Click a chip to show that bin. Drag items from the list onto a chip to add them (the chip lights up). Right-click a chip to rename, recolor, move it left or right, or delete it. The + chip makes a new bin.
- Rows are taller and show a colored stripe (the item's color, else its first bin's), a play button on hover for a quick preview, the name with the item's bin dots and first tags under it, a small waveform of the whole file, and the duration.
- The details area under the list adds bin toggles: one chip per bin, on when the item is in it. The details area is capped so the list always keeps room.
- An item can still be in several bins, which suits audio better than one folder per file (a promo can also be an ad).

Reason: matches how a show's assets are actually used, and makes the panel scannable at a glance. Names and look are our own.

## 2026-10-14: Track stems, clip groups, timecode display

Decision, Track Stems (File > Export > Track Stems): pick a folder, and every track that has clips and is not muted is written as its own stereo WAV named "Session - 01 Track.wav". Each stem runs the full length of the session, so stems line up when imported anywhere else. A stem plays its track even if another track is soloed, with the track's volume, pan and the master volume applied. Progress shows per stem, and Cancel stops the rest.

Decision, clip groups (Clip > Group Clips, Ungroup Clips, and right-click): a new optional `group` field on clips (serde default, so no version bump and old files open unchanged; schema regenerated). Clicking any clip in a group selects the whole group, so moves, deletes, copies and exports act on all of it. Ctrl+click adds or removes a whole group. Split parts stay in their group. Pasted copies get a group of their own, apart from the originals. Grouped clips show "Grouped" in their title bar.

Decision, timecode display (View > Time Format > Timecode): hh:mm:ss:ff at a fixed 30 frames per second, non-drop. Used by the ruler, the clock, Time Info and the Markers panel. Clicking the clock cycles time, samples, timecode. A frame rate setting can come with video support if ever needed.

## 2026-10-14: Workspaces, panel tab menu, saved editor settings, Default left column

Decision, Default layout: the left column (Files over Voice Chain, Batch Convert, QuickPass) runs the full window height. Levels and Time Info sit only under the center and right columns. The right column and Time Info are both 270 px so their edges line up. Default layout version 5, so older saved Default arrangements are rebuilt once.

Decision, panel tabs: tabs have no close button. Each group's active tab shows a menu button with three stacked dots (chosen over a hamburger icon so it reads as "more options", not navigation) that opens the same menu as right-clicking the tab. The active tab's title is underlined, in the accent color when its group has focus and in a dim color otherwise; the old accent line along the top of the tab is gone. The left column is now 270 px, like the right column, so the three tabs in the left group fit with the button (Default layout version 6). Right-click a tab for Float Panel (or Dock Group when floating), Maximize Group or Restore Group, Close Panel, Close Other Panels in Group, and Close Group. Panels still turn on and off from the Window menu. This makes it hard to break the layout by accident. Panel-specific items (for example in the Voice Chain) will join this menu when those panels are built.

Decision, workspaces (options beyond Default, so the app covers a full editing job out of the box). This replaces the earlier "Default only" rule in CLAUDE.md.
- Built in: Default; Focused Edit (timeline, files, levels only); Mixing (Mixer up front, tall meters, files and markers); Voiceover (files and markers, libraries); Review and Export (markers and history, wide meters). Names and layouts are our own and describe the job.
- Each workspace remembers its arrangement on its own, saved as you change it, in `workspaces/<id>.alworkspace` in the app config folder. The single saved layout from before is moved there as Default on first run.
- Window > Workspace: pick one; Reset Workspace (a built-in one goes back to its starting layout, a user one to how it was last saved); Save Changes to Workspace (user ones); Save as New Workspace; Delete Workspace (user ones, asks first). Reset to Default Layout stays in the Window menu.
- The workspace in use is remembered between runs.
- Each built-in layout has a version; changing a preset later rebuilds saved arrangements of it once, like the Default.

Decision, editor settings: snapping, Automatic Crossfades and the time display format are saved in preferences and restored at start.

## 2026-10-14: Time selection, ripple delete, clipboard, mixdown, markers (P3, second part)

Decision, time selection:
- Drag on the ruler, on empty lane space, or below the last track to select a time range. A plain click moves the playhead and clears the range. The range is saved with the view (not an edit, so not undoable), shows in the ruler and lanes, and Time Info shows its start, end and duration.
- Uses: Zoom to Selection, Clip > Trim > To Time Selection, Ripple Delete, Add Range Marker, and mixdown or bounce of the range.

Ripple Delete (Edit menu):
- Selected Clips: deletes them and pulls later clips on the same track left by the space they took.
- Time Selection in Selected Track / in All Tracks: removes that stretch of time. Clips inside are deleted, clips across an edge are trimmed, a clip spanning the whole range is split around it, and everything after moves left. Fades at the cut are kept or shortened to fit.
- Gap Between Clips: closes the empty space under the playhead on the selected track.
- A locked clip in the way stops the edit with a message; nothing changes.

Cut, Copy and Paste work on clips in the Editor. Paste puts the clips at the playhead, with the topmost on the selected track, keeping their spacing and track layout. The clipboard is the app's own, not the system clipboard, and lasts until the app closes.

Snapping (Edit > Snapping): on/off, to clips, to the playhead, to markers, and to ruler ticks (off by default). Shift while dragging still turns snapping off for that drag. These settings are not saved between runs yet.

Automatic Crossfades (Clip > Fades, on by default): when a move or trim makes clips overlap on a track, the earlier one fades out and the later one fades in across the overlap. A clip fully inside another is left as a layer.

Clip Color (Clip menu and right-click): six palette colors or the track color. Stored as a palette token so it follows the theme.

Mixdown:
- File > Export > Session Mixdown or Time Selection writes a stereo WAV.
- Multitrack > Mix Down to New File does the same and adds the result to the Files panel.
- Multitrack > Bounce to New Track does the same and places the result on a new track where the range started.
- The mix uses the same plan and renderer as playback at the session rate, so it sounds like what plays. The file uses the session bit depth (16 or 24-bit with triangular dither, or 32-bit float). Progress shows in a dialog with Cancel, and a cancelled export deletes its partial file. If the mix went over full scale, the notice says so and by how much.
- MP3 and loudness checks come with Publish Package (P7) and the loudness scan (P5).

Markers:
- Edit > Markers > Add Cue Marker (at the playhead) and Add Range Marker (from the time selection).
- Markers show as flags on the ruler, with dashed lines through the lanes. Drag a flag to move it (it snaps), click it to jump there, right-click it to rename or delete. Right-click empty ruler to add one.
- The Markers panel (Window menu, not in the Default layout) lists every marker. Click to jump, double-click the name to rename, change Cue or Subclip, delete. Chapter, Ad Slot and Social Clip kinds arrive with their phases.
- Core ops: `add_markers`, `remove_markers`, `set_markers`, all undoable. Markers stay sorted by time.

Dev build speed: the audio crates (symphonia, rubato, FFT, cpal, hound) are optimized in dev builds and our own code is built at opt-level 1. Without this, a mixdown in a dev build ran at about twice real time, too slow for a full episode. Other crates stay unoptimized so builds stay quick.

Reason: these are the edits a podcast cut needs day to day (cut out a stretch across all tracks, close gaps, move bits around, export the episode). `hound` moved from a test-only dependency to a normal one for writing WAV, as the stack list already planned.

## 2026-10-14: Clip editing (P3, first part)

Decision: clips can now be edited in the Editor.
- Drag a clip to move it in time or onto another track. Several selected clips move together and keep their spacing. The group stops at the session start and at the first and last track.
- Drag the lower part of a clip's left or right edge to trim it. Trims stop at the ends of the file and never leave a clip shorter than 10 ms.
- Drag the small square handles in a clip's title bar to set the fade in and fade out. New fades start as an S-curve, which sounds smoothest on speech. The fades never overlap.
- Moves, trims and fade drags snap to the playhead, the session start and other clips' edges within 8 pixels. Hold Shift while dragging to turn snapping off.
- Clip menu and right-click: Split at Playhead (Ctrl+K) splits the selected clips under the playhead, or every clip under it when none are selected. Split All Clips at Playhead always splits every track. Also Trim Start/End to Playhead, Mute Clip, Lock Position, Clip Gain, Fade In and Fade Out. The Fade In and Fade Out menu items add a one-second fade or remove it.
- Locked clips cannot be moved or trimmed. Each drag is one undo step and shows its result live while dragging.

Split moves the fade out to the right part and keeps the fade in on the left part.

Not yet: cut, copy and paste of clips, ripple delete, time selection, crossfades, clip color, merging and grouping. Those come next in P3.

Reason: this is the minimum to cut a real episode. The edit rules are pure functions in `src/panels/editor/clip-edit.ts` with tests; the core enforces locks and file bounds again in `set_clips`, so a bad edit from any source is refused.

## 2026-10-14: File > Open takes audio files

Decision: File > Open shows sessions (.altaudio), .sesx sessions and audio files in one dialog, and several files can be picked. A session opens as before; a .sesx is imported. Audio files go into the Files panel and are selected. If the session has no clips yet, each audio file is also placed on its own track at the start and the view zooms to fit, so opening one recording is enough to start listening and editing. Picking two sessions at once opens the first and says which were skipped.

Reason: opening single audio files, not just sessions, is a basic need. The Waveform editor (single-file destructive editing) is not built yet, so the multitrack is where an opened file can be played. When the Waveform editor arrives, opening an audio file can open it there instead; that is a one-line change in `openAudio`.

Clip playback: clip gain, clip mute and fades (linear, logarithmic, cosine) now shape playback. Changing them restarts playback in place, like moving a clip.

## 2026-10-14: Overview strip, Session Properties, clip edit op

Decision: the Editor gets an overview strip above the ruler. It shows every clip in the session, the playhead, and a window over what the lanes show. Drag the window to scroll, drag its edges to zoom, click outside it to jump, wheel to zoom, double-click to fit.

Session Properties (Multitrack menu) edits name, sample rate and bit depth as one undoable step. Changing the rate converts every timeline position (clips, fades, envelopes, markers, view) so nothing moves in time. Undo restores the exact original sample values rather than converting back, so repeated changes never drift. Source files are never touched; they are converted as they play.

The core has a new `set_clips` op (move, move to another track, trim, gain, mute, lock, fades). Locked clips refuse moves and trims, and a trim can never reach past the end of its file. The editor does not use it yet; dragging, trimming and splitting clips come next (P3), along with clip gain and fades in playback.

Reason: P2 is complete apart from bus tracks and sends. Buses are deferred until effects arrive (P5), because without a Voice Chain on a bus they only group volume, while clip editing is what makes the editor usable for a real episode. Buses can move back up if they are needed sooner.

## 2026-10-14: Mixer and Time Info panels

Decision: the Mixer shows one strip per track plus a Master strip. Each strip has pan, Mute, Solo, a fader and a meter. The fader scale runs from -inf to +12 dB with a bend at -24 dB, so the useful range near 0 dB gets most of the travel. Meters use the same scale as the fader so a level lines up with the fader mark. Master volume is a new `set_master_volume` op, so it is undoable and fader drags merge into one undo step like track faders. Double-click a fader to reset it to 0 dB.

Time Info shows the playhead, and Start, End and Duration of the selected clips (their combined span). The footer shows session length, sample rate, bit depth and a size estimate for a stereo mixdown at the session bit depth.

Reason: the Mixer is the second tab of the center group in the Default layout, and Time Info is in Default under Levels. Both read state the core already has, so they cost little and make the Default layout complete apart from P5 panels. Time selection ranges arrive with editing tools; until then the selected clips define the range.

## 2026-10-14: Playback (P2 start)

Decision:
- Space plays and pauses from the playhead. Stop returns the playhead to where playback started; Pause leaves it where playback reached. Go to Start is a toolbar button (no new shortcut). Clicking the ruler or an empty lane while playing jumps playback there.
- Three threads, per the real-time rule: a render thread decodes and resamples every track's clips into one lock-free ring buffer per track, about half a second ahead; the device callback only mixes those rings with the current gains and writes peak meters. It never allocates, locks, or touches the disk.
- Volume, pan, mute and solo apply on the next audio buffer, from atomics. Edits that move audio in time (adding, removing or moving clips or tracks) restart playback from the same spot.
- Pan is a balance control: centered plays both sides at full level and panning turns the far side down, so a centered voice plays at exactly the fader's level. Solo: if any track is soloed, only soloed tracks play; mute always wins.
- Seeking is sample-accurate, and resampled files have the resampler's delay removed, so a 44.1 kHz file and a 48 kHz file line up exactly (tested with a click track).
- Track headers: drag the dB or pan value left or right (Shift for fine steps), double-click to reset. A drag is one undo step. Small level meters show under each track while playing.
- The Levels panel shows the output level, left and right, from -60 dB to 0 dB with peak hold and over lights.
- The editor pages along to keep the playhead in view while playing. The playhead is saved in the session only when playback stops, so playing never marks the session as changed.
- Previewing a file in Drive Manager or Libraries stops session playback, and the reverse.

Reason: P2 playback per CLAUDE.md (cpal, rtrb, real-time thread rules).

## 2026-10-14: .sesx session import (task 11)

Decision:
- File > Import Session (.sesx) reads the user's own `.sesx` multitrack sessions for interoperability. It reads only the XML structure seen in real files; no other product's code or design is reused.
- Imported: tracks (name, mono or stereo, height, mute, solo, arm, volume as dB, pan), clips (position, source range, fade lengths and log or linear shape, lock), and media files. Media is looked for relative to the `.sesx` first, then at the stored absolute path, then by name beside the `.sesx`.
- Not imported, with a count in the status bar: effects (track, clip and master), buses, sends and automation. Effects are rebuilt in Voice Chain in P5.
- Missing media gets a placeholder so its clips still show where they were, marked "Media offline". The Files panel lists the file as an error.
- The import becomes a new, unsaved session named after the `.sesx` file. Saving it writes an `.altaudio`; the `.sesx` is never changed.
- Checked against two real production sessions: every track, clip and position came across. Real sessions live in `samples/`, which git ignores; `cargo test --test sesx_samples -- --ignored --nocapture` imports whatever is there.

Reason: CLAUDE.md task 11 and the P1 exit gate.

## 2026-10-14: Session save, open, autosave and recovery (task 12)

Decision:
- File > New (Ctrl+N), Open (Ctrl+O), Open Recent (last 10), Save (Ctrl+S), Save As, Save All, Close and Close All. With one session open, Save All and Close All act on it.
- New, Open, Close, Import Session and Exit (including the window close button) ask Save, Don't Save or Cancel when there are unsaved changes. The window title shows the session name with a star while unsaved.
- Sessions save atomically (temporary file, then rename). Each media file is stored with its absolute path and, when on the same drive, a path relative to the session file. Open tries the relative path first, so a show folder can move between drives or machines. Files not found show as errors in the Files panel.
- Files in the Files panel that no clip uses yet are saved with the session too, and so are the open libraries, which reopen with it.
- Autosave writes a recovery copy to the app data folder every 2 minutes while there are unsaved changes. Saving or closing cleanly deletes it. A copy found at startup means the app did not close normally: the user can Recover (it opens as unsaved work, saving to the original file) or Discard.
- Save As, or the first save of an untitled session, names the session after its file.
- The dock never saves or restores an empty layout. That could only come from a failed load and would leave the next start blank.

Reason: CLAUDE.md task 12 and charter rule 7 (work is never lost).

## 2026-10-14: Multitrack view (task 10)

Decision:
- The core owns the open session. The interface sends each edit as an op (add track, move track, set track, add media, add clips, remove clips, and a batch of these). The core applies it and returns the new session plus the op that reverses it, which goes on the undo stack. New ids come from the interface, so redo recreates exactly the same tracks and clips. A batch that fails part way changes nothing. This puts the session where the P2 audio engine will need it.
- Waveforms come from a peak cache (`.alpk`) in the app cache folder: min and max of every 256, 4,096 and 65,536 frames, 16-bit, at most two channels. It is a rebuildable cache, so it is compact binary with a version header, not JSON. It is rebuilt when the source file's size or modified time changes, or when it is damaged. Building runs off the UI thread and shows a percentage on the clip.
- Dropping files, Files panel rows, Drive Manager rows or library items onto a track adds them as clips one after another, starting where they were dropped. Dropping below the last track makes a new track. Files are added to the Files panel too. That is two undo steps (Import, then Add Clips).
- New sessions are 48 kHz. Clips from files at other rates are placed at their correct length; the engine resamples in P2.
- Track headers: name (double-click to rename), mono or stereo badge, Mute, Solo, Arm. Volume and pan are shown but adjusted in P2. Drag a header to reorder; every change is undoable.
- Zoom: Ctrl+= and Ctrl+- (the approved shortcuts), Ctrl+mouse wheel around the pointer, Zoom to Session. Shift+wheel scrolls sideways. Click or drag on the ruler to move the playhead.
- Delete in the Editor deletes selected clips. Removing a file from the Files panel keeps any file a clip still uses.
- Not in P1, now marked with their phase in the menus: markers, snapping, timecode display and time selection (P3), bus tracks (P2).
- The task 3 dev sandbox is gone; the Editor panel shows the real view in every build.

Reason: CLAUDE.md task 10, and the architecture rule that the core owns state the engine will play.

## 2026-10-14: Libraries panel (task 9)

Decision:
- A library is a folder with `library.json` and an `audio/` folder. New libraries go in Documents\Altered Audio\Libraries\<name> (home folder if there is no Documents folder). Any folder can be opened as a library.
- Files are copied in by default; the original path is kept. Files already inside the library folder are used in place.
- Every library change (add, remove, rename, tags, notes, color, collections) is one undo step on the same undo stack as the session. Undo saves the previous `library.json`. Removing an item never deletes its file, so undo always works. Library menu > Clean Up Library Folder deletes copied files that no item uses.
- Which libraries are open, and which one is showing, are remembered between runs (preferences, not undo).
- Space previews, Enter or double-click adds to the session, drag adds to the Files panel or onto a track. Drop files on the panel to copy them in. Delete removes selected items when the Libraries panel was clicked last.
- Collections are named, ordered groups; an item can be in several.

Reason: CLAUDE.md task 9. Network-wide libraries (P7) use the existing `scope` field.

## 2026-10-14: Drive Manager and preview playback (task 8)

Decision:
- Drive Manager lists drives (Windows drive letters; on Linux and macOS the root, mounted drives and /Volumes), plus Home, Desktop, Documents, Music and Downloads. Back, Forward, Up, a typed path, a filter box, Audio only, and Auto-play. Hidden and system files are skipped.
- Preview plays through the core, not the web view, to keep the rule that the interface never touches audio. A preview thread decodes and resamples to the output device's rate (rubato) into a lock-free ring buffer (rtrb); the device callback (cpal) only copies. This is the first piece of the P2 engine.
- The selected file's format, codec, rate, channels and length show at the bottom with a progress bar; click the bar to preview from that point.
- Double-click a folder to open it, a file to add it to the session; drag rows onto the Files panel or a track.
- Small interface settings (last folder, toggles, open libraries) are saved in `preferences.alprefs` in the app config folder. Never browser storage.
- Linux builds now need the ALSA headers (`libasound2-dev`) for audio output.

Reason: CLAUDE.md task 8.

## 2026-10-13: Platform priority

Decision: Windows first, Linux (Zorin OS) second, macOS third. Only Windows ships for now, but the code stays portable from the start instead of being ported later.

- Interface differences by OS live in `src/lib/platform.ts`: the reveal command is Show in Explorer, Show in File Manager, or Reveal in Finder, and help text names the right file manager.
- Duplicate detection in the Files panel ignores path case on Windows and macOS only. On Linux `Intro.wav` and `intro.wav` are different files.
- Claude's workspace is Linux, so every Rust test and live check already runs there. Windows is checked by the project lead.
- Left for later, when a second platform becomes a release target: Linux and macOS installers, Linux CI, Cmd for Ctrl on macOS, macOS signing and notarizing, and per-platform checks of audio devices (P2) and GPU features (P7).

Reason: the project lead may move to Zorin OS. Keeping platform differences in one place now costs little; finding Windows assumptions across the codebase later costs a lot.

## 2026-10-13: QuickPass scope

QuickPass is specified by behavior:

- QuickPass scans selected clips (or all clips) for problems and lists each finding with its start time, length, and clip.
- Silence finding uses two rules: audio below a quiet level for at least a set time is silence; audio above a content level for at least a set time is speech. Both levels and times are user settings, with a "suggest levels" button that measures the clip.
- For each finding, or all at once, the user can Remove it (ripple, so the multitrack stays in sync), Split the clip there and leave the pieces selected for review, or Mark it.
- Later scan types (clicks, clipping) follow the same find, review, act pattern.
- The QuickPass presets in CLAUDE.md (Trim Long Silent Pauses, Mark Non-Silent Areas, and the rest) are starting settings for these scans.

Phase: not assigned yet. It needs the multitrack view, clip splitting (P3), and level analysis, so P5 at the earliest.

## 2026-10-13: Files panel (task 7)

Decision:
- Decoding uses symphonia 0.6.1 (MPL-2.0, GPL-compatible). 0.6 has been the stable line since May 2026 and 0.5 is in maintenance. Only the formats we need are enabled: WAV, AIFF, MP3, FLAC, M4A/MP4 (AAC-LC and ALAC), raw AAC, and OGG Vorbis.
- Import measures each file in the core: format, codec, sample rate, channels, bit depth (lossless only), size, and exact length in samples. The first packet is decoded, so a file we cannot play (for example HE-AAC, which symphonia does not decode) fails at import with a clear reason, not later on the timeline. Files with no stored length (some MP3s) are measured by reading packet headers to the end, without decoding.
- Up to 4 files are measured at once, off the UI thread. Each row shows Reading, Ready, or Error, and the error text is in the row tooltip. Right-click > Try Reading Again re-checks a failed file.
- Importing and removing files are undoable commands. Measuring is not an edit, so Undo then Redo of an import does not re-read the files.
- Dropping files or folders from Explorer: the window's native drop events are hit-tested against panels. A drop that misses every panel still imports into the Files panel, so a drop never silently does nothing. Folders are searched up to 8 levels deep, at most 5,000 files per drop. Files already in the list are skipped (paths compared without case).
- The Import dialogs open from the core (tauri-plugin-dialog used from Rust), so the interface gets no file system permissions.
- Delete, Select All, Deselect All, and Find act on the area the user last clicked. For now that is only the Files panel; the Multitrack view joins in task 10. Menus show why these items are disabled elsewhere.
- Columns: Name, Status, Duration always fit at the 250px default; Sample Rate and Channels appear when the panel is wider. Click a header to sort, click again to reverse, a third time for import order.
- Close Unused Media waits for task 10, since "unused" needs clips to exist.

Reason: CLAUDE.md task 7, plus the charter rules on undo, drag and drop everywhere, and never freezing the window.

## 2026-10-13: Keymap (task 6)

Decision:
- Shipped shortcuts live in `src/keymap/default.json` (format `altaudio.keymap`, version 1). It holds exactly the 15 approved shortcuts; a test fails on any other.
- The user's changes are saved to `keymap.alkeys` in the app config folder, holding only what differs from the defaults. New defaults in later versions still reach users who never changed that command. A cleared command is stored as an empty list.
- Chords have one spelling: modifiers in the order Ctrl, Alt, Shift, then the key (`Ctrl+Shift+K`).
- Edit > Keyboard Shortcuts (also Help > Keyboard Shortcuts) lists every menu command with search, Change, Clear, and Default per row, and Reset All. Taking a shortcut that another command holds asks first, then moves it.
- One dispatcher routes key presses to actions. It ignores keys while typing in a text field or while a dialog is open. Held keys repeat only for Undo, Redo, and Zoom; Space fires once per press.
- Release builds block the webview's reload, print, and page-search keys (F5, Ctrl+R, Ctrl+P and similar) so a stray key cannot discard work. This adds no shortcuts.
- Menus and tooltips read shortcut text from the keymap, so a change shows everywhere at once.

Reason: CLAUDE.md asks for an editable keymap as data with only approved defaults. Saving only differences keeps upgrades clean.

## 2026-10-13: Window menu and Help links

Decision:
- The Window menu is back: every panel toggle, Reset to Default Layout, Minimize. View keeps editors, zoom, time format, envelopes, and the Podcast and Voiceover modes. Menu ids `view.panel.*` became `window.panel.*`; `session.*` became `multitrack.*`.
- Help gets The GCRN Website (https://thegcrn.com/) and GCRN Plugins, one link to a single plugins page. The plugins page is not live, so that item is disabled until `LINKS.plugins` in `src/app/links.ts` is set.
- Links open in the default browser through the core, which only allows https links on thegcrn.com.

Reason: Window is where long-time editors look for panels. One plugins page means new plugins never need an app update.

## 2026-10-13: Drive Manager

Decision: the drive and folder browser panel is Drive Manager (`drive-manager`), replacing File Manager. Default layout version 4.

Reason: says exactly what it browses, and reads clearly apart from the Files panel.

## 2026-10-13: Multitrack menu name

Decision: the track and mixdown menu is called Multitrack, not Session. Opens with Alt+M.

Reason: Multitrack is the industry-standard term. The menu contents and the rest of the tree stay our own.

## 2026-10-13: Name cleanup

Decision:
- Panels: the selection readout panel is now Time Info (`time-info`); the media browsing panel is now File Manager (`file-manager`). Default layout version bumped to 3 so saved layouts with the old ids are replaced.
- Effects get our own names: Noise Reducer, Speech Leveler, Dynamics, Limiter, Click Repair, Breath Gate. The full map of old names is kept in the private naming map.
- Presets: four EQ presets become Hum Cut 60 Hz (US), Hum Cut 50 Hz (Europe), Rumble Cut, Subsonic Cut, rebuilt with our own settings. Other preset names stay for now.
- Tests now fail on any of the old names in menus or panel titles.

Reason: CLAUDE.md bans other products' panel and feature names in the UI. The remaining preset names were judged to be ordinary descriptions.

## 2026-10-13: Default layout v2

Decision: right column is Audio FX Panel over Libraries. The selection readout (now Time Info) moves to the right end of the bottom strip, beside Levels. Saved layouts now record which Default they came from (`defaultVersion`); a layout from an older Default is replaced on next launch.

Reason: Libraries needed a home in Default, and pre-release it is better to roll everyone onto the new Default than keep stale arrangements.

## 2026-10-13: Menu bar (task 5)

Decision:
- Seven menus: File, Edit, Session, Clip, Effects, View, Help, grouped around the common podcast workflow. The Window menu is folded into View > Panels; Favorites is dropped; track and mixdown commands live under Session.
- Items use our names (Voice Chain, Loudness Match, QuickPass, Settings) and our own effect groupings. Branded, video, CD, account and extension items are left out.
- Podcast workflow items sit at their workflow step: Speaker Balance, Transcribe Session and Remove Filler Words under Session; Check Loudness and Publish Package under File; Remove Music under Clip.
- Menus are data (`src/app/menus/model.ts`). Every item names an action id; actions live in one registry (`src/app/actions.ts`). No registered action means disabled, with the reason as a tooltip.
- Only the shortcuts approved in CLAUDE.md appear, each once. A test enforces this and bans other products' feature names.
- Keyboard: Alt plus the underlined letter opens a menu (standard Windows menu access, not an app shortcut). Arrows, Enter and Escape work as in any Windows menu.
- Built now: Undo and Redo (labels name the edit), every panel toggle, Reset to Default Layout, About, Exit. Exit gets an unsaved-changes prompt in task 12.
- The menu bar is drawn in the app, not a native Windows menu, so it matches the dark theme.

Reason: CLAUDE.md allows similar menus but not an identical tree. Folding Window into View and merging the track menus gives a structure experienced editors find their way around in seconds, with a tree of our own.

## 2026-10-13: Panel dragging uses pointer events

Decision: the docking layout uses `dndStrategy: "pointer"`. Floating groups get a Dock button and a Dock item on the tab right-click menu, which return their panels to their home spot. App drag and drop (Files to tracks, clips between tracks) must also use pointer events, not HTML5 drag and drop.

Reason: panels could be dragged out but not back in. On Windows, the webview swallows HTML5 drop events while Tauri's native file drop is on, and native file drop is the only way to get real file paths from Explorer (task 7). Pointer events work with it on.

## 2026-10-13: Visual direction: panel framing and value color

Decision: each panel group now sits in its own frame with a 2px dark gutter. Added a `--value` token (light blue) for editable numbers and actionable names, used from task 7 on. Kept our own tab treatment, icons, and header layout. No per-panel menu icon in the header.

Reason: density, framing, and color for editable values are common to pro audio tools and help in long sessions. Header chrome stays our own, per the originality rules in CLAUDE.md.

## 2026-10-13: Docking shell and Default layout

Decision:
- Use the `dockview` package, not `dockview-core` directly. In version 8, `dockview-core` is marked internal and ships no stylesheet; `dockview` re-exports it with the CSS.
- Default layout: left column 250px (Files on top; Voice Chain, Batch Convert, QuickPass tabbed below), center Editor and Mixer tabs, right column 200px (Audio FX Panel over the selection readout, now Time Info), Levels across the full bottom.
- Markers, History and Favorites are not in Default. History and Markers can be turned on. Favorites is dropped.
- One Default layout, no named presets. The user's arrangement saves itself to `workspace.alworkspace` in the app config folder (never browser storage), 400 ms after the last change. Reset to Default deletes it and rebuilds.
- A damaged or unreadable saved layout falls back to Default with a message instead of blocking startup.
- Turned-off panels come back to a fixed home spot (registry `home` list), falling back to a window edge.
- Layout changes are not session edits and do not go through the undo stack.
- Every group has float and maximize buttons (shown on hover) and a right-click tab menu with the same plus close.
- Panels without features show a plain "not built yet" note naming the task or phase, never fake controls.
- A temporary Panels menu in the top bar holds the toggles and Reset until the View menu exists (task 5).

Reason: puts the common podcast workflow in reach, keeps the theme ours (compact strips, our tokens), and keeps layout state out of the session file.

## 2026-10-13: CLAUDE.md layout conflict resolved

Decision: removed a second, conflicting "Confirmed Default workspace layout" section. Kept the original Default. Dropped Favorites from the task 5 menu list.

Reason: two Default layouts contradicted each other, and the second one conflicted with the original-design guardrail.

## 2026-10-13: CLAUDE.md rewrite applied to finished work

Decision: aligned tasks 1 to 3 with the revised CLAUDE.md.
- Redo is Ctrl+Y. Ctrl+Shift+Z is not bound (standard Windows shortcuts only).
- `HostProfile` now means a host voice profile (for example Host, Co-host, Guest): gate settings plus the Voice Chain preset to load. It no longer describes podcast hosting export settings. Tracks reference it with `host_profile_id`.
- New scaffolding fields: `Track.role` (Standard or Guest, P5), `MarkerKind::AdSlot` with `contracted_duration` (P5), `MarkerKind::SocialClip` (P7), `Library.scope` (Show or Network, P7), `Session.modes` (podcast and voiceover toggles), `EpisodeMetadata.show_id` (P6).
- Comments use the app's own names (for example Voice Chain).
- The v1 fixtures and schemas were regenerated.

Reason: the format is unreleased and no real files exist yet, so changing v1 costs nothing now. After the first real session is saved, v1 is frozen and changes need a migration.

## 2026-10-13: Undo stack design

Decision:
- Commands may be async. The stack runs every operation strictly one at a time, in call order, so fast repeated Ctrl+Z presses are safe while edits round-trip to the Rust core.
- A command that throws is not recorded. A failed undo leaves the position unchanged.
- Rapid edits of the same kind (fader drags) merge into one entry when they arrive within 750 ms and the command allows it.
- `CompoundCommand` records several changes as one entry and rolls back on partial failure.
- Commands must never call the stack. Synchronous nesting is caught; async nesting would deadlock, so it is a rule.
- History tracks a save point, so the stack can tell whether the session has unsaved changes. Task 12 uses this for the title bar and autosave.
- One stack per open document. In P1 that is a single `sessionHistory`.
- Limit: 1000 entries, oldest dropped first.

Reason: the core is async from day one, and losing or reordering an undo is the worst bug an editor can have.

## 2026-10-13: Dev sandbox

Decision: a dev-only sandbox (`src/app/dev/CommandSandbox.svelte`) drives the undo stack until real editing exists. It is compiled out of release builds. Delete it when the multitrack view (task 10) produces real commands.

Reason: the History panel has nothing to show until there are edits, and the project lead needs to try it to sign off.

## 2026-10-13: Hard-coded undo and redo shortcuts

Decision: Ctrl+Z and Ctrl+Y are bound in `src/app/shortcuts.ts` until the keymap (task 6) replaces them. They are skipped while typing in a text field so the field's own undo works.

## 2026-10-11: File format ids and extensions

Decision: every file has a `format` header (`altaudio.session`, `altaudio.library`, `altaudio.template`, `altaudio.workspace`, `altaudio.host_profile`) next to `schema_version`. Extensions: `.altaudio`, `library.json`, `.altemplate`, `.alworkspace`, `.alhost` (host voice profile).

Reason: the header lets the loader reject the wrong kind of file with a clear message instead of a confusing parse error. `.altemplate` and `.alhost` are new names, not set in CLAUDE.md, and can still change before any are shipped.

## 2026-10-11: Versioning rules

Decision:
- Adding a field with a serde default is not a version bump.
- Renaming, removing, or changing the meaning of a field is. It needs a `migrate_step`, a new fixture folder, and a passing fixture test.
- Files from a newer app are refused with a message to update, never half-loaded.
- A template's nested session is upgraded with the session migrations.

Reason: "never break an old file." Frozen fixtures in `src-tauri/tests/fixtures/v<N>/` are opened by every build, so a breaking change fails CI.

## 2026-10-11: Schemas generated from Rust, checked in CI

Decision: JSON Schemas in `/schemas` are generated from the Rust types with `schemars`. A test fails when they drift. Regenerate with `UPDATE_SCHEMAS=1 cargo test --test schemas`.

Reason: one source of truth. The schemas can never silently disagree with the code. Field doc comments become schema descriptions.

## 2026-10-11: Units in the session model

Decision: timeline positions are integer samples at the session rate. Clip source offsets are integer samples at the source file's own rate. Levels in dB. Pan from -100 to 100.

Reason: integers avoid float drift on long sessions and match how `.sesx` stores positions. Keeping source offsets in the file's own rate means a 44.1 kHz file in a 48 kHz session is never rounded.

## 2026-10-11: Atomic saves

Decision: saves write a `.tmp` file beside the target, sync it, then rename it over the target.

Reason: a crash or power cut during save leaves the old file intact. Needed for autosave (P1 task 12).

## 2026-10-11: Session validation reports, it does not refuse

Decision: `session::validate` returns a list of issues (missing media, unknown bus, routing loops, duplicate ids). Loading does not fail on them.

Reason: a slightly broken session should still open so the user can fix it. Refusing to open work is worse than showing a warning.

## 2026-10-11: Model placement

Decision: `HostProfile` and `Workspace` live in `session/` beside the session model. (`export/` is P3+ and gets no code in P1.) `Library`, `LibraryItem` and `Template` live in `library/`. A `MediaRef` type (source file reference) was added, though not in the CLAUDE.md type list, because clips need to point at a file.

Reason: `session/` owns all file formats and migrations. `export/` is P3+ and gets no code in P1.

## 2026-10-11: TypeScript 5.9, not 7

Decision: pin TypeScript `~5.9`.

Reason: TypeScript 7 (the native rewrite) is out, but `svelte-check` supports 5 and 6 only. Proven over new.

## 2026-10-11: License identifier

Decision: `GPL-3.0-or-later`.

Reason: the FSF's recommended form, and it stays compatible with GPL code from Audacity. Change to `GPL-3.0-only` before the first public release if preferred. It is one find-and-replace plus the header check.

## 2026-10-11: Rust lib name

Decision: the crate builds a library named `altered_audio_editor_lib` plus the app binary.

Reason: the standard Tauri 2 layout. It lets integration tests in `src-tauri/tests` use the core without starting a window, and avoids a Windows name clash between the lib and the exe.

## Windows distribution

The installer (NSIS, per-user or all-users, shows the GPL license) is the main download. A portable zip with the bare exe is attached beside it. Builds run in GitHub Actions on version tags and make a draft release, so nothing goes public until it's tested. MSI is skipped unless someone asks. Files are unsigned until code signing is set up.

## 2026-10-15: Empty-track right-click menu, nested menus, Batch Convert templates

Decision:

- The right-click menu on empty lane space is the session menu: Paste, Ripple Delete, Select, Groups, Track, Insert, Marker, Mixdown Session to New File, Export Mixdown, Bounce to New Track, Session Properties. Clips keep their own menu (clip items plus track items). Right-clicking empty space keeps any chosen clips and makes that lane the chosen track.
- Context menus now nest. A row with an arrow opens its submenu on hover or with the Right arrow key, and Left arrow or Escape closes it. A submenu flips to the left or lifts up when it would run off screen.
- Every entry in the menu runs a registered app action, so it cannot drift from the menu bar. Entries whose feature is not built show disabled, with the reason as a tooltip.
- New Select actions, also added to Edit > Select: Next Clip in Selected Track, Clips to End of Track, Clips to Start of Session, Clips to End of Session, Select Current View Time, Select All Time, Select Time Around Selected Clips, Clear Time Selection. The "clips to" actions count from the earliest chosen clip, or from the playhead when none is chosen. The rules are in `src/app/clip-select.ts` with tests.
- No keyboard shortcuts were added.
- Batch Convert naming is one template with tokens: {name}, {number} (position in the list, at least two digits), {date} (2026-10-15) and {format}. Buttons under the field insert a token at the cursor. Settings saved with the old Before and After boxes become a template on load, so nothing is lost.

Reason: the empty-lane menu is where an editor reaches for track and session actions without going to the menu bar. Templates replace two boxes with something that can number a batch and date-stamp it, which is what people do with a folder of call recordings.

## 2026-10-15: QuickPass silence scan built

QuickPass now scans the selected clips (or every clip on a track that plays when none are selected) and lists silences. The Rust side only measures: `quickpass_levels` renders each clip as it plays (gain and Voice Chain included) and returns its level in 10 ms windows. All the rules live in `src/panels/quickpass/quickpass.ts` so they are unit-tested: a stretch below the quiet level for at least the set time is a silence, and a burst of sound shorter than the speech time does not break it. Suggest Levels sets the two levels from the clip's own noise floor and speaking level. Each finding can be removed (gap closes), split out (the piece is left selected), or marked (range marker); "all" versions of each make a single undo step. Remove closes the gap on the clip's own track by default; a setting closes it on all tracks to keep them in sync. Clicking a row moves the playhead. Settings are remembered. No new shortcuts. Click and clipping scans follow the same find, review, act pattern later. Presets (Trim Long Silent Pauses and so on) wait for P5 with the rest of the effects presets.

## 2026-10-15: first release is v0.1.1

The repository history was rewritten into one clean commit before going public, and the first `v0.1.0` tag and draft pointed at the old history. The first public release is therefore v0.1.1, so the number alone shows it came from the clean history. Version lives in `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock` and `src-tauri/tauri.conf.json`; change all five together.
