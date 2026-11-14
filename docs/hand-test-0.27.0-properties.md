# Hand test: Stretch and Remix in the Properties panel

Open a session with a music clip on a track (a few minutes long works best) and a voice clip.

## The panel

1. Select the music clip. Open the Properties panel (Window > Properties). You should see Stretch, Remix and, for files with more than two channels only, Source Channels, each as a header with its state on the right. Click a header: it opens and closes.
2. The Stretch header says Off, Real-time or Rendered. Change Mode and check the header follows.

## Menu commands open the sections

3. With the music clip selected, choose Clip > Fit Music to Length. The Properties panel shows (even if it was closed) with Remix open. No dialog appears.
4. Choose Clip > Time Stretch with a clip selected. Properties shows with Stretch open. No dialog appears.

## Remix

5. Target Duration opens on the clip's own length. Type a shorter length, such as 30, and press Enter or click Find Joins. Up to three plans appear, each with joins, match and where it cuts or repeats. Remix Duration shows the length of the chosen plan.
6. Pick Option 2, check Remix Duration changes. Click Apply Remix. The clip is replaced by joined pieces. Ctrl+Z brings back the original clip in one step.
7. Type a different length after finding joins: the plans grey out and ask you to find joins again.
8. Click 15 s, 30 s, 60 s: the box fills. Make a time selection: a Selection button appears and fills the box with its length.
9. Select two clips: Remix says to select one. A looped or locked clip says why it cannot be remixed.

## Stretch to the time selection

10. Select a clip, make a time selection on the timeline. Stretch shows Fit to Time Selection. Click it: the clip stretches to the selection and moves there. Undo is one step.

## Not in this part

The extra Remix sliders (Edit Length, Features, Minimum Loop, Maximum Slack) and live re-remixing when you drag the clip edge are not here yet. They need the Remix clip property, a separate release. The old Time Stretch and Fit Music dialogs still exist in the code but nothing opens them now.
