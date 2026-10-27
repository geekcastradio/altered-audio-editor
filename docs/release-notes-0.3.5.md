# Altered Audio Editor 0.3.5

A smaller release built from the second Audiform batch and a few fixes to our own checks.

## New and improved
- **Better time stretch.** Stretching now uses a phase-locked phase vocoder. Chords and voices sound clearer, sharp attacks stay sharp, and stereo stays stable. This applies to playback and to export.
- **Resizable automation lanes.** Drag the bottom edge of an automation lane to make it taller or shorter. The size is saved with the session and can be undone.
- **Easier automation picking.** The automation picker now lists each effect's controls under that effect's name. Plug-ins that could not be scanned show their controls once they are running.
- **Smoother playhead dragging.** Dragging the playhead follows your pointer, and playback jumps to that spot when you let go.

## Security checks
- The automatic security audit now ignores three zip advisories that come only from the updater plug-in. Our own code uses the fixed version, and updates are signed before anything is unpacked.

## Credit
Time stretch, lane resizing, the grouped picker and the playhead fix started in the Audiform fork by nwh212 and were brought over with changes to fit this project. Thank you.

## Test checklist
- Stretch a voice clip and a music clip to about 80% and 125% and compare against 0.3.1, once in playback and once in an export
- Drag the playhead across the full overview width and confirm the overview bar still reaches the left edge
- Resize an automation lane, save, reopen, and undo the resize
- Pick controls from a built-in effect and from a plug-in
- Updater: install 0.3.1, then confirm it offers 0.3.5
