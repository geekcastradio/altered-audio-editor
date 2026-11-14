# 0.26.0 hand test

Run on the installer build from the draft release. Tick each line.

## Fit Music and Time Stretch (the 0.25.0 bug)
- [ ] Import a music MP3, click the clip so it is highlighted, Clip > Fit Music to Length. The dialog opens.
- [ ] Find Joins offers plans. Apply one. The clip is the new length and one undo puts it back.
- [ ] Clip > Time Stretch opens its dialog. Apply a stretch, then reset it.
- [ ] With no clip selected, Fit Music shows its reason in the status bar.
- [ ] Same on Mac (ask the Mac tester).

## Import
- [ ] New session, import one file with no track selected. It lands on Track 1.
- [ ] Import a second file at the same spot. It lands on Track 2.
- [ ] Import with Track 4 selected. It lands on Track 4.
- [ ] Drop three files together with "one per track" on and nothing selected. They fill Tracks 1, 2 and 3.

## Close Dead Space
- [ ] On AGU_EP478 after a QuickPass, Multitrack > Close Dead Space with 0.5 s. Silent stretches shrink, speech is not cut, tracks stay in sync.
- [ ] Play across a closed stretch. No clicks.
- [ ] Markers after a closed stretch moved with the audio.
- [ ] One undo puts everything back.
- [ ] With a locked clip in the way, a notice explains and nothing changes.
- [ ] Pause set to 0 and to 2 both work. 3 is refused.

## Find Isolated Short Clips
- [ ] Run it on AGU_EP478 with the defaults. Play a few. Are they real bumps and breaths? Note anything that is real speech.
- [ ] Nothing is checked at the start. Remove two checked clips. One undo brings both back.
- [ ] Tagged "under another track" clips really do have audio from another track under them.
- [ ] Try 1.5 s and 4 s. The list changes sensibly.

## Sync Tracks
- [ ] Import the same file twice on two tracks and nudge one by about 1.5 s. Sync Tracks reports about 1.5 s. Move Track lines them up. Undo works.
- [ ] A real double-ender with a call recording and a local mic: offset and drift look right by ear. Note the drift per hour.
- [ ] Two unrelated recordings: a message says they share no audio.
- [ ] A track shorter than two minutes: a message says so.
- [ ] Moving a track that would start before zero is refused with a notice.

## Home, End and jumps
- [ ] Zoom in on a long session, press End. The Multitrack view scrolls so the end of the session is on screen and the playhead sits near the right edge.
- [ ] Press Home. The view scrolls back to the start.
- [ ] Next and previous marker, and next and previous edge, scroll the view so the playhead is visible, centered.
- [ ] If the playhead is already on screen, the view does not jump around.
- [ ] The overview bar and the Multitrack view agree on where the playhead is.

## Voice Chain readouts
- [ ] Click the Input readout, type a value such as -3 or +2.5, press Enter. The slider moves and the sound changes. Same for Output and Mix.
- [ ] Escape cancels and keeps the old value. A value past the limit is clamped, not rejected.
- [ ] Dragging the slider still updates the readout. One undo reverts a typed change.
- [ ] The readouts fit their boxes at the default panel width.

## QuickPass cancel and retry
- [ ] Start a QuickPass scan and press Cancel. The panel returns to ready within a second and CPU use drops.
- [ ] Scan again right after cancelling. It runs and finds the same result as a fresh scan.
- [ ] Start a scan, then change the scan type. The old scan stops and the new one runs.
- [ ] If a scan fails, the panel shows a message naming the problem instead of staying on Scanning. Note the message and which clips were selected.
- [ ] If a scan ever hangs: note the clip, whether CPU is high or idle, and whether it was the first scan.

## Levels meter direction
- [ ] With Levels docked along the bottom, the bars lie down as before.
- [ ] Drag the Levels panel to the left or right edge of the window. The bars stand up, the dB scale runs down the right side, and L and R readouts sit under the bars.
- [ ] Play something. The bars rise from the bottom, the peak line holds, and the over light at the top turns red and clears when clicked.
- [ ] Drag it back to the top or bottom. It lies down again. Resizing near square does not flicker.
- [ ] A 3D Space session in a side dock shows every speaker channel as its own bar.

## Panel drop guides
- [ ] Drag a panel tab over another panel. Outlined zones appear on that panel: a strip down each side, a strip across the top and bottom, and a box in the middle.
- [ ] The blue block still shows where the panel will land, and the zone under the pointer matches the block.
- [ ] Drop in the middle: it joins as a tab. Drop in a side strip: it docks beside. Drop in the top strip: it docks above.
- [ ] The lines are visible on a narrow side panel and on a wide one such as the Multitrack.
- [ ] Nothing is left on screen after the drop or after cancelling the drag.
