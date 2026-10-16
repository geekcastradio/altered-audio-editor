// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * Clip volume and pan envelopes: reading points, mapping values to the lane, and
 * keeping points lined up with the audio when a clip is trimmed or
 * split. The level is held flat before the first point and after the
 * last, and moves in straight lines (in dB) between points, the same as
 * playback.
 */

import type { ClipDoc, EnvelopePointDoc } from "../../stores/session";

export const DB_MIN = -60;
export const DB_MAX = 12;

/** Which clip envelope. */
export type EnvKind = "volume" | "pan";

/**
 * Range of each envelope. `top` is the value drawn at the top of the
 * lane: +12 dB for volume, full left for pan.
 */
export const RANGE: Record<EnvKind, { top: number; bottom: number; step: number }> = {
  volume: { top: DB_MAX, bottom: DB_MIN, step: 0.1 },
  pan: { top: -100, bottom: 100, step: 1 },
};

export function pointsOf(clip: ClipDoc, kind: EnvKind): EnvelopePointDoc[] {
  return clip.envelopes?.find((e) => e.target.type === kind && e.enabled)?.points ?? [];
}

export function volumePoints(clip: ClipDoc): EnvelopePointDoc[] {
  return pointsOf(clip, "volume");
}

/** Value to a y position in a box `top` to `top + height`. */
export function valueToY(kind: EnvKind, v: number, top: number, height: number): number {
  const r = RANGE[kind];
  const lo = Math.min(r.top, r.bottom);
  const hi = Math.max(r.top, r.bottom);
  const c = Math.max(lo, Math.min(hi, v));
  return top + ((c - r.top) / (r.bottom - r.top)) * height;
}

/** A y position to a value, rounded to the envelope's step. */
export function yToValue(kind: EnvKind, y: number, top: number, height: number): number {
  const r = RANGE[kind];
  const f = Math.max(0, Math.min(1, (y - top) / Math.max(1, height)));
  const v = r.top + f * (r.bottom - r.top);
  return Math.round(v / r.step) * r.step;
}

/** Clamps a value to the envelope's range. */
export function clampValue(kind: EnvKind, v: number): number {
  const r = RANGE[kind];
  return Math.max(Math.min(r.top, r.bottom), Math.min(Math.max(r.top, r.bottom), v));
}

/** "L 40", "C", "R 25" for pan; "-6.0 dB" for volume. */
export function formatValue(kind: EnvKind, v: number): string {
  if (kind === "volume") return `${v > 0 ? "+" : ""}${v.toFixed(1)} dB`;
  const p = Math.round(v);
  return p === 0 ? "C" : p < 0 ? `L ${-p}` : `R ${p}`;
}

/** Level in dB at `pos` (samples from the clip start). 0 dB when there are no points. */
export function levelAt(points: readonly EnvelopePointDoc[], pos: number): number {
  if (points.length === 0) return 0;
  const first = points[0]!;
  const last = points[points.length - 1]!;
  if (pos <= first.position) return first.value;
  if (pos >= last.position) return last.value;
  for (let i = 1; i < points.length; i++) {
    const b = points[i]!;
    if (pos <= b.position) {
      const a = points[i - 1]!;
      const t = (pos - a.position) / Math.max(1, b.position - a.position);
      return a.value + (b.value - a.value) * t;
    }
  }
  return last.value;
}

/** dB to a y position in a box `top` to `top + height`: +12 at the top, -60 at the bottom. */
export function dbToY(db: number, top: number, height: number): number {
  const d = Math.max(DB_MIN, Math.min(DB_MAX, db));
  return top + ((DB_MAX - d) / (DB_MAX - DB_MIN)) * height;
}

export function yToDb(y: number, top: number, height: number): number {
  const f = Math.max(0, Math.min(1, (y - top) / Math.max(1, height)));
  return Math.round((DB_MAX - f * (DB_MAX - DB_MIN)) * 10) / 10;
}

/**
 * The points for the part of a clip from `from` to `to` (samples from
 * its start), moved so `from` becomes 0. Levels at both ends are kept
 * by adding a point there, so the cut piece sounds the same.
 */
export function sliceVolume(points: readonly EnvelopePointDoc[], from: number, to: number): EnvelopePointDoc[] {
  if (points.length === 0) return [];
  const inside = points.filter((p) => p.position > from && p.position < to).map((p) => ({ ...p, position: p.position - from }));
  const curve = points[0]!.curve;
  return [
    { position: 0, value: levelAt(points, from), curve },
    ...inside,
    { position: Math.max(1, to - from), value: levelAt(points, to), curve },
  ];
}

/** Adds a point at `pos`, or moves the one already there. Returns the new list and the point's index. */
export function addPoint(points: readonly EnvelopePointDoc[], pos: number, value: number): { points: EnvelopePointDoc[]; index: number } {
  const p = Math.max(0, Math.round(pos));
  const next = points.filter((x) => x.position !== p).concat({ position: p, value, curve: "linear" });
  next.sort((a, b) => a.position - b.position);
  return { points: next, index: next.findIndex((x) => x.position === p) };
}

/** Moves point `i` to `pos` and `value`, keeping it between its neighbors. */
export function movePoint(
  points: readonly EnvelopePointDoc[],
  i: number,
  pos: number,
  value: number,
  len: number,
  kind: EnvKind = "volume",
): EnvelopePointDoc[] {
  const lo = i > 0 ? points[i - 1]!.position + 1 : 0;
  const hi = i < points.length - 1 ? points[i + 1]!.position - 1 : len;
  const next = points.map((p) => ({ ...p }));
  next[i] = { ...next[i]!, position: Math.round(Math.max(lo, Math.min(hi, pos))), value: clampValue(kind, value) };
  return next;
}

/** Label of the envelope chip in a clip's title bar. */
export function chipLabel(showVolume: boolean, showPan: boolean): string {
  if (showVolume && showPan) return "Vol + Pan";
  if (showVolume) return "Volume";
  if (showPan) return "Pan";
  return "Envelopes";
}

/**
 * Where the envelope chip sits in a clip's title bar: right-aligned,
 * clear of the fade-out handle. Null when the clip is too narrow.
 * Width comes from the label length so drawing and hit-testing agree.
 */
export function chipBox(x0: number, x1: number, label: string): { x: number; w: number } | null {
  const w = Math.round(label.length * 5.6 + 22);
  if (x1 - x0 < w + 70) return null;
  return { x: x1 - w - 14, w };
}
