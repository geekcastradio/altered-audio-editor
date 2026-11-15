# Hand test 0.32.0: takes and comping

Take lanes in this release live in a Takes panel. Cycle (lap) recording is not part of it yet.

## Make takes

1. Record or place the same line three times on one track so the clips overlap (drag two copies of a clip over the first, or record again over the same stretch).
2. Select the three clips. Clip > Make Takes from Selected Clips. The Takes panel opens with one set and three takes. Only the first plays; the others are drawn dim.
3. Play it: you hear take 1 only. Undo, redo: the marks come and go as one step.
4. Select clips that are not on one track, or only one clip: the menu item is grayed out with a reason.

## Comp

5. Make a time selection over a stretch in the middle. In the Takes panel press Use over selection on Take 2.
6. The track now has Take 1, then Take 2 over the selection, then Take 1 again. Take 2's piece is bright and the Take 1 piece under it is dim. Play it through: the switch is where the selection starts and ends.
7. Clear the selection and press Use on Take 3: the whole stretch comes from Take 3.
8. Pick a take over a stretch where it has no audio (a shorter take): the range is cut back to what the take covers and nothing goes silent.
9. Undo each pick: one step each.

## Flatten and delete

10. Delete a take: its clips go. If a stretch is now silent the notice says so. Undo brings it back.
11. Flatten (panel button, or select a take clip and Clip > Flatten Takes): the dim pieces go and the rest become ordinary clips; the Takes panel is empty again. Undo restores it.

## Elsewhere

12. Move, trim, split, fade or change gain on a take piece: it works as with any clip, and the take stays a take.
13. Mix Down or export a mix: only the playing takes are in it.
14. File > Export as .sesx: dim takes are not written.
15. Settings > Multitrack: turn Take lanes off. The Takes panel says so and the two Clip menu items are grayed out. A session that has takes still plays the chosen takes.
16. Save and reopen the session: the takes and which one plays come back. Note: an older build opens a session with takes and plays every take at once.
