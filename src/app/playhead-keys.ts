// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/** Keyboard moves of the playhead: a step, a page, or to the ends of the time selection. */

import { parseTime } from "../panels/time-info/time-info";

export type HeadMove = "step-left" | "step-right" | "page-left" | "page-right" | "to-in" | "to-out";

export interface HeadContext {
  /** Playhead, in samples. */
  pos: number;
  /** The end of the content, in samples. */
  end: number;
  /** Zoom: samples per pixel. */
  spp: number;
  /** Width of the lane in pixels. */
  width: number;
  /** The time selection, whose ends are the In and Out points. */
  sel: { start: number; end: number } | null;
}

const clamp = (n: number, end: number) => Math.max(0, Math.min(Math.round(n), Math.max(0, end)));

/** Where the playhead goes, or null when the move has nowhere to go (no selection for In and Out). */
export function headTarget(move: HeadMove, c: HeadContext): number | null {
  const step = Math.max(1, Math.round(c.spp));
  const page = Math.max(step, Math.round(Math.max(1, c.width) * c.spp));
  switch (move) {
    case "step-left":
      return clamp(c.pos - step, c.end);
    case "step-right":
      return clamp(c.pos + step, c.end);
    case "page-left":
      return clamp(c.pos - page, c.end);
    case "page-right":
      return clamp(c.pos + page, c.end);
    case "to-in":
      return c.sel && c.sel.end > c.sel.start ? clamp(c.sel.start, c.end) : null;
    case "to-out":
      return c.sel && c.sel.end > c.sel.start ? clamp(c.sel.end, c.end) : null;
  }
}

/** A typed time for Jump to Time, kept inside the content. Null when the text is not a time. */
export function jumpTarget(
  text: string,
  rate: number,
  format: "time" | "samples" | "timecode" | "seconds" | "bars",
  fps: number,
  end: number,
): number | null {
  const at = parseTime(text, rate, format, fps);
  return at === null ? null : clamp(at, end);
}

export type EdgeMove = "start-left" | "start-right" | "end-left" | "end-right";

/**
 * Moves one border of the time selection by `step` samples, keeping at least one
 * sample selected and staying inside 0..end. Null when there is no selection.
 */
export function adjustSelection(
  sel: { start: number; end: number } | null,
  move: EdgeMove,
  step: number,
  end: number,
): { start: number; end: number } | null {
  if (!sel || sel.end <= sel.start) return null;
  const n = Math.max(1, Math.round(step));
  const top = Math.max(1, Math.round(end));
  switch (move) {
    case "start-left":
      return { start: clamp(sel.start - n, top), end: sel.end };
    case "start-right":
      return { start: Math.min(clamp(sel.start + n, top), sel.end - 1), end: sel.end };
    case "end-left":
      return { start: sel.start, end: Math.max(clamp(sel.end - n, top), sel.start + 1) };
    case "end-right":
      return { start: sel.start, end: Math.max(sel.start + 1, clamp(sel.end + n, top)) };
  }
}
