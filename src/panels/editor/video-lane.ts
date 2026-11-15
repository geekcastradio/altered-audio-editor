// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/** The Video Reference bar in the Multitrack: where the picture sits on the time axis. */

export interface LaneBar {
  /** Pixels from the left edge of the lanes. */
  left: number;
  width: number;
}

/** Narrowest bar drawn, so a very short or unknown-length video can still be grabbed. */
export const MIN_BAR = 24;

/**
 * The bar for a video that starts at `offset` seconds and lasts `duration` seconds (0 when
 * unknown, which draws a bar out to the right edge). Null when it is wholly out of view.
 */
export function laneBar(offset: number, duration: number, spp: number, scroll: number, rate: number, widthPx: number): LaneBar | null {
  if (!(spp > 0) || !(rate > 0)) return null;
  const start = (offset * rate - scroll) / spp;
  const end = duration > 0 ? start + (duration * rate) / spp : widthPx;
  if (end <= 0 || start >= widthPx) return null;
  const left = Math.max(0, start);
  const right = Math.min(widthPx, end);
  return { left, width: Math.max(MIN_BAR, right - left) };
}

/** The new start time after dragging the bar `dx` pixels; never before the session start. */
export function draggedOffset(offset: number, dx: number, spp: number, rate: number): number {
  const next = offset + (dx * spp) / rate;
  return Math.max(0, Math.round(next * 1000) / 1000);
}

/** The file name at the end of a path, either slash. */
export function baseName(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}
