// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * The spectral box: a rectangle in time (samples) and frequency (a position
 * on the edits' own log axis) that can be moved, resized by its edges and
 * corners, typed in, or made from a frequency band alone.
 */

import { freqAt, fracOf, type SpectralShape } from "./spectrum";

export type BoxShape = Extract<SpectralShape, { kind: "box" }>;

/** Which part of a box a point is on: an edge or corner, the inside, or nothing. */
export type BoxHit = "move" | "n" | "s" | "e" | "w" | "ne" | "nw" | "se" | "sw";

/** Pixel rectangle of a box on screen (top is smaller than bottom). */
export interface PixelBox {
  left: number;
  right: number;
  top: number;
  bottom: number;
}

/** How close, in pixels, a point must be to an edge to take hold of it. */
export const GRAB = 6;

/**
 * What the pointer is on. Edges and corners win over the inside, so a thin
 * box can still be resized. A box too small to have an inside gives its
 * edges the room.
 */
export function hitBox(r: PixelBox, x: number, y: number, grab = GRAB): BoxHit | null {
  const w = r.right - r.left;
  const h = r.bottom - r.top;
  // Never let the grab zones cover more than a third of a side each.
  const gx = Math.min(grab, w / 3);
  const gy = Math.min(grab, h / 3);
  if (x < r.left - grab || x > r.right + grab || y < r.top - grab || y > r.bottom + grab) return null;
  const dw = Math.abs(x - r.left);
  const de = Math.abs(x - r.right);
  const west = dw <= Math.max(gx, 2) && (dw <= de || de > Math.max(gx, 2));
  const east = de <= Math.max(gx, 2) && !west;
  // On a box too thin for both, the nearer side wins.
  const dn = Math.abs(y - r.top);
  const ds = Math.abs(y - r.bottom);
  const north = dn <= Math.max(gy, 2) && (dn <= ds || ds > Math.max(gy, 2));
  const south = ds <= Math.max(gy, 2) && !north;
  if (north && west) return "nw";
  if (north && east) return "ne";
  if (south && west) return "sw";
  if (south && east) return "se";
  if (north) return "n";
  if (south) return "s";
  if (west) return "w";
  if (east) return "e";
  if (x > r.left && x < r.right && y > r.top && y < r.bottom) return "move";
  return null;
}

/** Cursor for a hit. */
export function cursorFor(hit: BoxHit | null): string | null {
  switch (hit) {
    case "move":
      return "move";
    case "n":
    case "s":
      return "ns-resize";
    case "e":
    case "w":
      return "ew-resize";
    case "ne":
    case "sw":
      return "nesw-resize";
    case "nw":
    case "se":
      return "nwse-resize";
    default:
      return null;
  }
}

/** The smallest box a drag can make, in samples and in axis fraction. */
const MIN_SAMPLES = 8;
const MIN_FRAC = 0.002;

/**
 * Moves or resizes `box` by a drag of `dSamples` across and `dFrac` up the
 * axis. Moving keeps the size and stops at the file's edges; resizing moves
 * only the grabbed sides and cannot turn the box inside out.
 */
export function adjustBox(box: BoxShape, hit: BoxHit, dSamples: number, dFrac: number, total: number): BoxShape {
  let { start, end, bottom, top } = box;
  if (hit === "move") {
    const w = end - start;
    const h = top - bottom;
    start = Math.max(0, Math.min(total - w, start + dSamples));
    end = start + w;
    bottom = Math.max(0, Math.min(1 - h, bottom + dFrac));
    top = bottom + h;
    return { kind: "box", start: Math.round(start), end: Math.round(end), bottom, top };
  }
  if (hit.includes("w")) start = Math.max(0, Math.min(end - MIN_SAMPLES, start + dSamples));
  if (hit.includes("e")) end = Math.min(total, Math.max(start + MIN_SAMPLES, end + dSamples));
  if (hit.includes("n")) top = Math.min(1, Math.max(bottom + MIN_FRAC, top + dFrac));
  if (hit.includes("s")) bottom = Math.max(0, Math.min(top - MIN_FRAC, bottom + dFrac));
  return { kind: "box", start: Math.round(start), end: Math.round(end), bottom, top };
}

/** The numbers a person reads and types for a box. */
export interface BoxValues {
  /** Seconds. */
  start: number;
  end: number;
  /** Hertz. */
  low: number;
  high: number;
}

export function boxValues(box: BoxShape, rate: number, nyquist: number): BoxValues {
  return {
    start: box.start / rate,
    end: box.end / rate,
    low: freqAt(box.bottom, nyquist),
    high: freqAt(box.top, nyquist),
  };
}

/**
 * A box from typed numbers. Values are held inside the file and the
 * frequency axis, and the ends are put in order, so any entry gives a valid
 * box. Returns null when a number is not a number.
 */
export function boxFromValues(v: BoxValues, rate: number, total: number, nyquist: number): BoxShape | null {
  if (![v.start, v.end, v.low, v.high].every(Number.isFinite)) return null;
  const a = Math.round(Math.min(v.start, v.end) * rate);
  const b = Math.round(Math.max(v.start, v.end) * rate);
  const start = Math.max(0, Math.min(total, a));
  const end = Math.max(start + 1, Math.min(total, b));
  const lo = fracOf(Math.max(0, Math.min(v.low, v.high)), nyquist);
  const hi = fracOf(Math.max(v.low, v.high), nyquist);
  return { kind: "box", start, end, bottom: lo, top: Math.max(hi, Math.min(1, lo + MIN_FRAC)) };
}

/**
 * A frequency band across time: from `fracA` to `fracB` on the axis (either
 * order), over the time selection when there is one, else the whole file.
 */
export function bandBox(fracA: number, fracB: number, time: { start: number; end: number } | null, total: number): BoxShape {
  const lo = Math.max(0, Math.min(1, Math.min(fracA, fracB)));
  const hi = Math.max(0, Math.min(1, Math.max(fracA, fracB)));
  const start = time ? Math.max(0, Math.min(time.start, time.end)) : 0;
  const end = time ? Math.min(total, Math.max(time.start, time.end)) : total;
  return { kind: "box", start, end, bottom: lo, top: Math.max(hi, Math.min(1, lo + MIN_FRAC)) };
}

/** The range line for a box: "low to high Hz, range Hz". */
export function rangeText(v: BoxValues): string {
  const f = (hz: number) => (hz >= 1000 ? `${+(hz / 1000).toFixed(2)} kHz` : `${+hz.toFixed(hz < 100 ? 1 : 0)} Hz`);
  return `${f(v.low)} to ${f(v.high)}, range ${f(v.high - v.low)}`;
}
