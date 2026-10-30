// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * How-to text for an effect's window. It shows behind the (i) in the window's title bar,
 * so the controls get the room. Effects with nothing to add just show their description.
 */

import { PHASE_CURVE } from "./phase-curve";

const HELP: Readonly<Record<string, string>> = {
  "altaudio.amp_model":
    "Input sets how hard the capture is driven. Level Match brings every capture to about the same loudness, so switching between them is fair. The capture plays in mono; both channels get the result.",
  "altaudio.opto_leveler":
    'Turn up Peak Reduction until the needle shows a few dB of leveling, then set Gain for the level you want. Emphasis makes it listen harder to "s" sounds.',
  "altaudio.tube_preamp": "Drive adds warmth; past halfway it starts to sound overdriven. The needle shows the output against a 0 mark at -18 dBFS.",
  "altaudio.parametric_eq":
    "Drag a node to move its band. Drag the small handles either side of a band, or use the mouse wheel over its node, to change its width (Q). Double-click HP or LP to switch them on or off.",
  "altaudio.shaped_noise":
    "Frequencies under the orange threshold are turned down. Drag a node to raise or lower the threshold around it, or pick it below to type its settings. The mouse wheel over a node changes its width; double-click switches it on or off.",
  [PHASE_CURVE]: "Drag a point; click empty space to add one (up to 8); double-click or right-click a point to remove it.",
};

/** The text behind an effect's (i): its description, then any how-to. Empty when there is none. */
export function effectHelp(effect: string, description?: string | null): string {
  return [description?.trim(), HELP[effect]].filter(Boolean).join("\n\n");
}
