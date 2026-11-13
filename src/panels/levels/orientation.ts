// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

export type LevelsOrientation = "horizontal" | "vertical";

/** How much longer one side must be before the meters turn. Near-square panels keep what they had. */
const TURN = 1.2;

/**
 * Which way the Levels meters run. A panel docked to the side of the window is tall and narrow, so the
 * bars stand up; one docked along the top or bottom is wide and short, so they lie down. A panel close
 * to square keeps its current direction, so resizing around the middle does not flicker.
 */
export function levelsOrientation(width: number, height: number, previous: LevelsOrientation): LevelsOrientation {
  if (!(width > 0) || !(height > 0)) return previous;
  if (height > width * TURN) return "vertical";
  if (width > height * TURN) return "horizontal";
  return previous;
}
