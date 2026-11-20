// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/** How close, in pixels, the pointer must be to a selection border to grab it. */
export const EDGE_GRAB_PX = 6;

export type Edge = "start" | "end";

/**
 * Which border of a time selection the pointer is over, or null. `x` and the
 * view are in the same pixel space. When the selection is so narrow that both
 * borders are in reach, the closer one wins, and a tie goes to the side the
 * pointer is outside of (so a tiny selection can still be widened both ways).
 */
export function edgeAt(
  sel: { start: number; end: number } | null | undefined,
  x: number,
  view: { scroll: number; spp: number },
  grabPx: number = EDGE_GRAB_PX,
): Edge | null {
  if (!sel || sel.end <= sel.start || view.spp <= 0) return null;
  const xs = (sel.start - view.scroll) / view.spp;
  const xe = (sel.end - view.scroll) / view.spp;
  const ds = Math.abs(x - xs);
  const de = Math.abs(x - xe);
  const inS = ds <= grabPx;
  const inE = de <= grabPx;
  if (inS && inE) {
    if (ds < de) return "start";
    if (de < ds) return "end";
    return x < xs ? "start" : "end";
  }
  if (inS) return "start";
  if (inE) return "end";
  return null;
}

/** The anchor for dragging this border: the border that stays where it is. */
export function anchorFor(sel: { start: number; end: number }, edge: Edge): number {
  return edge === "start" ? sel.end : sel.start;
}
