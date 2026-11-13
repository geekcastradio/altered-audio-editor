# Altered Audio Editor 0.26.0

Tighter dialogue, synced double-enders, and two dialogs that finally open.

## What's new

- **Close Dead Space.** Multitrack > Close Dead Space shortens every stretch where none of your tracks has audio down to a pause you pick (0 to 2 seconds, 0.5 by default). Everything after each stretch moves left together, so the tracks stay in step, and markers move with them. It is one undo step. It leaves alone any pause that is already short, and it will not run if a track has MIDI clips or a locked clip is in the way.
- **Find Isolated Short Clips.** Multitrack > Find Isolated Short Clips lists the stray bumps, coughs and breaths that QuickPass can leave behind as tiny clips on their own. Set how short is short and how much empty room counts as isolated. Each one has a Play button, and clips with audio from another track under them are tagged, since those may be bleed or a quick reply. Nothing is checked for you. Check the ones to remove and remove them in one undo step.
- **Sync Tracks.** Multitrack > Sync Tracks lines up two recordings of the same conversation, such as a call recording against a local mic. It reports how far the second track trails the first, how fast their clocks drift apart, and how sure it is, then moves the second track into line in one undo step. The first track is never changed. Both tracks need at least two minutes of shared audio, and the offset has to be under ten minutes. Two microphones that each heard only their own voice have nothing in common to match, and Sync Tracks says so.

## Fixed

- **Fit Music to Length and Time Stretch dialogs** now open. In 0.25.0, choosing either from the Clip menu did nothing.
- **Importing audio** now goes on the first track with room at that spot. In a new session the first file lands on Track 1, not a new Track 7 under the six empty ones. Dropping several files side by side fills the empty tracks first too.
- **Home, End and marker jumps** now scroll the Multitrack view to the playhead. Before, only the overview bar followed.
- **QuickPass** can be cancelled for real, and you can scan again right after. A scan that cannot read its clips now says so instead of looking like "nothing found".

## Also new

- **Levels meter turns to fit.** Docked to the side of the window, the Levels meter stands up with the scale down the right, like Audition. Docked along the top or bottom it lies down as before.
- **Panel drop guides.** Dragging a panel over another now draws outlined zones for beside, above, below or as a tab, so you can see where it will land before you let go.
- **Voice Chain readouts** are typeable. Click Input, Output or Mix, type a value, press Enter.

## Good to know

- Sync Tracks measures drift but does not correct it yet. Over a long recording, moving the track fixes the start, and the two can still slip apart. The report tells you how much per hour.
- Sync Tracks listens to the first 30 minutes of each track.
- Close Dead Space and Find Isolated Short Clips only touch clips. They do not move MIDI clips.
