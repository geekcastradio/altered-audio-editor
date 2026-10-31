// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * Recording automation: the maths of one take. While playback runs, the
 * controls of tracks and buses in Write, Latch or Touch mode feed moves
 * into a `Take`; when playback stops it hands back each lane's new points.
 *
 * - Write: from the start of the take a lane that exists is rewritten with
 *   the control's value, all the way to the stop. A control with no lane
 *   starts writing when it is first moved.
 * - Latch: starts at the first touch and writes until the stop.
 * - Touch: writes while the control is held; on release the lane takes
 *   over again after a short glide.
 *
 * Positions are session samples at the playhead, which is where the sound
 * is heard. The half-second a routed mix runs behind only delays how soon
 * a moved fader is heard, not where its point lands.
 */

import type { EnvelopePointDoc } from "../stores/session";
import { autoAt } from "../panels/editor/automation";

export type RecordMode = "write" | "latch" | "touch";

/** How long the lane takes to come back after a release, in seconds. */
export const GLIDE_SECONDS = 0.02;

/** Default share of a lane's range under which a point is not worth keeping. */
export const DEFAULT_THIN = 0.003;

interface Segment {
  from: number;
  /** Null while it is still writing. */
  to: number | null;
  points: EnvelopePointDoc[];
}

interface Lane {
  mode: RecordMode;
  base: EnvelopePointDoc[];
  /** The value the lane holds with no points. */
  rest: number;
  /** How far a point may sit off the line between its neighbors and be dropped. */
  tolerance: number;
  segments: Segment[];
}

const pt = (position: number, value: number): EnvelopePointDoc => ({ position: Math.max(0, Math.round(position)), value, curve: "linear" });

/**
 * Drops points that lie within `tolerance` of the straight line between
 * the points kept either side of them (Douglas-Peucker on the value axis).
 * The first and last points always stay. Zero keeps everything.
 */
export function thin(points: readonly EnvelopePointDoc[], tolerance: number): EnvelopePointDoc[] {
  if (tolerance <= 0 || points.length <= 2) return [...points];
  const keep = new Array<boolean>(points.length).fill(false);
  keep[0] = true;
  keep[points.length - 1] = true;
  const stack: [number, number][] = [[0, points.length - 1]];
  while (stack.length > 0) {
    const [a, b] = stack.pop()!;
    const pa = points[a]!;
    const pb = points[b]!;
    let worst = -1;
    let at = -1;
    for (let i = a + 1; i < b; i++) {
      const p = points[i]!;
      const f = pb.position === pa.position ? 0 : (p.position - pa.position) / (pb.position - pa.position);
      const d = Math.abs(p.value - (pa.value + (pb.value - pa.value) * f));
      if (d > worst) {
        worst = d;
        at = i;
      }
    }
    if (at >= 0 && worst > tolerance) {
      keep[at] = true;
      stack.push([a, at], [at, b]);
    }
  }
  return points.filter((_, i) => keep[i]);
}

export class Take {
  private readonly lanes = new Map<string, Lane>();

  /** `rate` is the session's sample rate, for the glide after a release. */
  constructor(private readonly rate: number) {}

  /** Starts watching a lane. Write mode with a lane that has points begins writing at `pos` at once. */
  arm(key: string, mode: RecordMode, base: readonly EnvelopePointDoc[], rest: number, tolerance: number, pos: number, value: number): void {
    if (this.lanes.has(key)) return;
    const lane: Lane = { mode, base: [...base], rest, tolerance, segments: [] };
    this.lanes.set(key, lane);
    if (mode === "write" && base.length > 0) this.begin(lane, pos, value);
  }

  has(key: string): boolean {
    return this.lanes.has(key);
  }

  /** True while the lane is being written right now. */
  writing(key: string): boolean {
    const lane = this.lanes.get(key);
    const last = lane?.segments[lane.segments.length - 1];
    return !!last && last.to === null;
  }

  private begin(lane: Lane, pos: number, value: number): void {
    lane.segments.push({ from: Math.max(0, Math.round(pos)), to: null, points: [pt(pos, value)] });
  }

  /** A control was touched (Touch, Latch) or moved for the first time (Write with no lane yet). */
  grab(key: string, pos: number, value: number): void {
    const lane = this.lanes.get(key);
    if (!lane || this.writing(key)) return;
    // Latch writes from the first touch on; a later touch only continues it.
    if (lane.mode === "latch" && lane.segments.length > 0) return;
    this.begin(lane, pos, value);
  }

  /** The control moved to `value` at `pos`. Moves outside a write are not recorded. */
  set(key: string, pos: number, value: number): void {
    const lane = this.lanes.get(key);
    if (!lane) return;
    if (!this.writing(key)) {
      if (lane.mode === "write") this.begin(lane, pos, value);
      else return;
    }
    const seg = lane.segments[lane.segments.length - 1]!;
    const last = seg.points[seg.points.length - 1]!;
    const p = Math.max(0, Math.round(pos));
    if (p === last.position) seg.points[seg.points.length - 1] = pt(p, value);
    else if (p > last.position) seg.points.push(pt(p, value));
  }

  /** The control was let go. Touch stops writing; Latch and Write keep going. */
  release(key: string, pos: number): void {
    const lane = this.lanes.get(key);
    if (!lane || lane.mode !== "touch" || !this.writing(key)) return;
    const seg = lane.segments[lane.segments.length - 1]!;
    const last = seg.points[seg.points.length - 1]!;
    seg.points.push(pt(Math.max(pos, last.position), last.value));
    seg.to = Math.max(pos, last.position);
  }

  /** Playback stopped at `pos`: every lane's points, merged with what was there. Lanes nothing was written to are left out. */
  finish(pos: number): Map<string, EnvelopePointDoc[]> {
    const out = new Map<string, EnvelopePointDoc[]>();
    for (const [key, lane] of this.lanes) {
      for (const seg of lane.segments) {
        if (seg.to === null) {
          const last = seg.points[seg.points.length - 1]!;
          seg.points.push(pt(Math.max(pos, last.position), last.value));
          seg.to = Math.max(pos, last.position);
        }
      }
      if (lane.segments.length === 0) continue;
      out.set(key, this.merge(lane));
    }
    return out;
  }

  private merge(lane: Lane): EnvelopePointDoc[] {
    const glide = Math.max(1, Math.round(this.rate * GLIDE_SECONDS));
    const spans = lane.segments.map((s) => ({ from: s.from, to: s.to ?? s.from, points: thin(s.points, lane.tolerance) }));
    const inside = (p: number) => spans.some((s) => p >= s.from && p <= s.to);
    // What was there, outside what was written, stays.
    const merged: EnvelopePointDoc[] = lane.base.filter((p) => !inside(p.position));
    for (const s of spans) {
      const lastBase = lane.base[lane.base.length - 1];
      // The lane's value just before the write holds up to it, so the write starts clean.
      if (lane.base.length > 0 && s.from > 0) merged.push(pt(s.from - 1, autoAt(lane.base, s.from - 1, lane.rest)));
      merged.push(...s.points);
      // After it the lane comes back, unless there was nothing further on.
      if (lastBase && lastBase.position > s.to) {
        const back = s.to + glide;
        merged.push(pt(back, autoAt(lane.base, back, lane.rest)));
      }
    }
    merged.sort((a, b) => a.position - b.position);
    // One point per position: a later one wins, then the tidy pass drops duplicates.
    const unique: EnvelopePointDoc[] = [];
    for (const p of merged) {
      const prev = unique[unique.length - 1];
      if (prev && prev.position === p.position) unique[unique.length - 1] = p;
      else unique.push(p);
    }
    return unique;
  }
}

/** A lane's thinning tolerance: a share of its range, `share` 0 for none. */
export function toleranceFor(range: { top: number; bottom: number }, share: number): number {
  return Math.abs(range.top - range.bottom) * Math.max(0, share);
}
