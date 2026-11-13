// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * Where to scroll the Editor so a position that has jumped out of view (Home, End, next marker)
 * is on screen. Null when it is already in view and the view should stay put.
 *
 * `scroll` is the first visible sample, `span` how many samples fit across the Editor, and `at`
 * where in the new view the position lands: 0.5 is the middle, 0.9 near the right edge.
 */
export function revealScroll(scroll: number, span: number, to: number, at = 0.5): number | null {
  if (!(span > 0)) return null;
  if (to >= scroll && to <= scroll + span) return null;
  return Math.max(0, Math.round(to - span * at));
}
