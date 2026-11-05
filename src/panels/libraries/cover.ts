// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/** Helpers for the Show Appearance popup: which dropped files are pictures, and where the popup sits. */

/** A JPEG or PNG, the only pictures a library cover can be. */
export function isPicturePath(path: string): boolean {
  return /\.(jpe?g|png)$/i.test(path);
}

/** The pictures among dropped or picked files, in order. */
export function picturesOnly(paths: readonly string[]): string[] {
  return paths.filter(isPicturePath);
}

export interface Anchor {
  left: number;
  top: number;
  bottom: number;
}

/**
 * Top left corner for a popup of `size` hung under `anchor`, kept inside the
 * window. When there is no room below it opens above the anchor instead.
 */
export function popoverPosition(
  anchor: Anchor,
  size: { w: number; h: number },
  view: { w: number; h: number },
  gap = 6,
  edge = 8,
): { left: number; top: number } {
  const left = Math.max(edge, Math.min(anchor.left, view.w - size.w - edge));
  let top = anchor.bottom + gap;
  if (top + size.h > view.h - edge) top = Math.max(edge, anchor.top - gap - size.h);
  return { left, top };
}
