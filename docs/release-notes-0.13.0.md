# Altered Audio Editor 0.13.0

A big round of ready-made settings, one new way to shape dynamics, and a few editing gaps closed.

## New effects

- **Dynamics Curve.** Draw the whole dynamics response as one curve: squeeze the loud parts, lift the quiet parts and close the gate on the very quiet parts, all in one effect. Drag up to eight points on the graph, add a point by clicking, remove one by double-clicking. A knee rounds the corners, and look-ahead lets the gain move before a loud sound arrives. Find it under Effects > Dynamics.
- **Center Isolator.** Keeps or removes what sits in the middle of a stereo mix, such as a lead voice. It works bin by bin, so it can pull a voice out from under music better than the simple Center Remover. Choose Keep or Remove, set how wide "the middle" is, and limit the work to a range of frequencies. Find it under Effects > Stereo and Channels.

## Presets

- **Built-in presets for more effects.** Dynamics, Limiter, Speech Leveler, Parametric EQ, Noise Reducer, Precision Filter, Click Repair, Breath Gate, Expander and Dynamics Curve now ship with named presets for common jobs. Pick them from each effect's preset menu.
- **Podcast Voice chains.** New: Podcast Voice - Host, Podcast Voice - Co-Host and Podcast Voice - Guest (the old Podcast Voice 2), plus Podcast Voice - Rumble Cut and Podcast Voice - Quick Level.
- **Batch recipes.** Batch Convert's "Process with" list now includes a one-effect recipe for every built-in preset above, grouped as Compression, Leveling, Limiting, Breath gates, Noise reduction, Repair, Filters and EQ.

## Editing

- **Silence Time Selection in Selected Clips** (Edit menu). Dips each selected clip's volume across the time selection, with short ramps in and out so there is no click. The audio is not changed, so the dip can be dragged or removed like any other envelope.
- **Bring Clip to Front and Send Clip to Back** (Clip menu). Changes which overlapping clip sits on top. Several selected clips keep their order among themselves.
- **Set Current Clipboard** (Edit menu). Five clipboards. Copy, Cut and Paste use the current one, so you can keep several pieces at once. The clipboards last for the session.
