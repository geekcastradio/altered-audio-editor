# Altered Audio Editor 0.60.0

A larger batch of the move from TypeScript to Rust. Nothing here changes how the app looks or works. The interface still uses its TypeScript code; the Rust copies are tested against the same fixtures so they can take over later.

## Moved to Rust, with tests that compare both sides

- Markers and chapters: marker naming, kinds and colors, filtering, range lengths, chapter export (timestamps, CSV, podcast JSON) and chapter warnings.
- Voice Chain math: EQ graph widths and Q, Phase Rotate curve points, band split edges.
- Clip properties: channel labels, channel mix presets, Music Fit plan text.
- Files panel formats: duration, rate, channels, size and status text, plus list sorting and search.
- Match EQ and Publish Package: response matching, episode checks, file lists and VTT chapters.
- Automation lanes: lane heights, value ranges, snapping, adding, moving and thinning points.
- Library view: grouping, filtering, sorting and folding. Name sorting matches the browser only for letters, digits and spaces; see `docs/decisions.md`.
- Pitch edit choices, the welcome plan and bug report text from the previous batch.

## Notes

- No new settings, shortcuts or menu items.
- Free, GPLv3, no telemetry.
