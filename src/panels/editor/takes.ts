// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * Take lanes and comping. A set of takes is a group of clips on one
 * track that cover the same passage; each has a take number, and the
 * `active` ones play. Comping splits the takes at the edges of a range
 * and switches which take plays there. Pure functions that return ops.
 */

import type { ClipDoc, Op, SessionDoc, TakeDoc } from "../../stores/session";
import { clipEnd, isolate, mediaOf } from "./clip-edit";

export interface TakeLane {
  number: number;
  clips: ClipDoc[];
}

export interface TakeSet {
  set: string;
  trackId: string;
  start: number;
  end: number;
  /** Lanes in take order. */
  lanes: TakeLane[];
}

/** True for a take that is not the chosen one: it stays on the track, silent. */
export function isOffTake(c: ClipDoc): boolean {
  return !!c.take && !c.take.active;
}

/** Every set of takes in the session, in track order, then by start. */
export function takeSets(doc: SessionDoc): TakeSet[] {
  const media = mediaOf(doc);
  const out: TakeSet[] = [];
  for (const t of doc.tracks) {
    const bySet = new Map<string, ClipDoc[]>();
    for (const c of t.clips) {
      if (!c.take) continue;
      const list = bySet.get(c.take.set) ?? [];
      list.push(c);
      bySet.set(c.take.set, list);
    }
    for (const [set, clips] of bySet) {
      const lanes = new Map<number, ClipDoc[]>();
      for (const c of clips) {
        const n = c.take!.number;
        lanes.set(n, [...(lanes.get(n) ?? []), c]);
      }
      out.push({
        set,
        trackId: t.id,
        start: Math.min(...clips.map((c) => c.start)),
        end: Math.max(...clips.map((c) => clipEnd(c, media, doc.sample_rate))),
        lanes: [...lanes.entries()].sort((a, b) => a[0] - b[0]).map(([number, cs]) => ({ number, clips: cs.sort((a, b) => a.start - b.start) })),
      });
    }
  }
  return out.sort((a, b) => a.start - b.start);
}

/** The next free take number in a set. */
export function nextTakeNumber(doc: SessionDoc, set: string): number {
  let n = 0;
  for (const t of doc.tracks) for (const c of t.clips) if (c.take?.set === set) n = Math.max(n, c.take.number);
  return n + 1;
}

/**
 * Turns the chosen clips (all on one track, none already a take) into one
 * set of takes. They are numbered by start time. Each one plays unless it
 * overlaps one that already plays, so clips that do not overlap all stay
 * audible. Returns null when fewer than two clips qualify.
 */
export function makeTakesOps(doc: SessionDoc, ids: Iterable<string>, newSet: () => string): Op | null {
  const want = new Set(ids);
  const media = mediaOf(doc);
  const rate = doc.sample_rate;
  for (const t of doc.tracks) {
    const picked = t.clips.filter((c) => want.has(c.id) && !c.take && !c.locked).sort((a, b) => a.start - b.start || a.id.localeCompare(b.id));
    if (picked.length < 2) continue;
    const set = newSet();
    const playing: [number, number][] = [];
    const changes = picked.map((c, i) => {
      const a = c.start;
      const b = clipEnd(c, media, rate);
      const clash = playing.some(([x, y]) => a < y && b > x);
      if (!clash) playing.push([a, b]);
      return { id: c.id, take: { set, number: i + 1, active: !clash } };
    });
    return { op: "set_clips", changes };
  }
  return null;
}

/**
 * Makes `number` the take that plays over the range a..b. The takes of the
 * set are split at the edges of the range, the pieces of that take inside
 * switch on, and the pieces of the other takes that it covers switch off.
 * The range is cut back to what the chosen take covers, so nothing it
 * lacks goes silent.
 */
export function pickRangeOps(doc: SessionDoc, set: string, number: number, a: number, b: number, newId: () => string): Op[] {
  const track = doc.tracks.find((t) => t.clips.some((c) => c.take?.set === set));
  if (!track) return [];
  const media = mediaOf(doc);
  const rate = doc.sample_rate;
  // Only the stretch the chosen take actually covers can be switched to it.
  const mine = track.clips.filter((c) => c.take?.set === set && c.take.number === number);
  if (!mine.length) return [];
  a = Math.max(a, Math.min(...mine.map((c) => c.start)));
  b = Math.min(b, Math.max(...mine.map((c) => clipEnd(c, media, rate))));
  if (b <= a) return [];
  const ops: Op[] = [];
  const inside = new Map<number, string[]>();
  for (const n of new Set(track.clips.filter((c) => c.take?.set === set).map((c) => c.take!.number))) {
    const only = { ...doc, tracks: doc.tracks.map((t) => (t.id === track.id ? { ...t, clips: t.clips.filter((c) => c.take?.set === set && c.take.number === n) } : t)) };
    const r = isolate(only, [track.id], a, b, newId);
    ops.push(...r.ops);
    inside.set(n, r.inside);
  }
  // Where each piece sits after the split: look it up from the ops, or the original clip.
  const placed = new Map<string, [number, number]>();
  for (const c of track.clips) placed.set(c.id, [c.start, clipEnd(c, media, rate)]);
  for (const op of ops) {
    if (op.op !== "restore_clips") continue;
    for (const [, c] of op.clips as [string, ClipDoc][]) placed.set(c.id, [c.start, clipEnd(c, media, rate)]);
  }
  const covered = (inside.get(number) ?? []).map((id) => placed.get(id)!).filter(Boolean);
  const changes: { id: string; take: TakeDoc }[] = [];
  for (const [n, ids] of inside) {
    for (const id of ids) {
      const span = placed.get(id);
      if (!span) continue;
      const active = n === number ? true : !covered.some(([x, y]) => span[0] < y && span[1] > x);
      changes.push({ id, take: { set, number: n, active } });
    }
  }
  if (changes.length) ops.push({ op: "set_clips", changes });
  return ops;
}

/** Flatten Takes: drops the silent pieces and turns the rest into ordinary clips. */
export function flattenOps(doc: SessionDoc, set: string): Op[] {
  const off: string[] = [];
  const keep: string[] = [];
  for (const t of doc.tracks) {
    for (const c of t.clips) {
      if (c.take?.set !== set) continue;
      (c.take.active ? keep : off).push(c.id);
    }
  }
  const ops: Op[] = [];
  if (off.length) ops.push({ op: "remove_clips", ids: off });
  if (keep.length) ops.push({ op: "set_clips", changes: keep.map((id) => ({ id, clear_take: true })) });
  return ops;
}

/** Delete Take: removes every piece of one take. */
export function deleteTakeOps(doc: SessionDoc, set: string, number: number): Op[] {
  const ids: string[] = [];
  for (const t of doc.tracks) for (const c of t.clips) if (c.take?.set === set && c.take.number === number) ids.push(c.id);
  return ids.length ? [{ op: "remove_clips", ids }] : [];
}

/** Stretches of a set where no take plays, as [from, to] pairs. */
export function silentGaps(doc: SessionDoc, set: string): [number, number][] {
  const s = takeSets(doc).find((x) => x.set === set);
  if (!s) return [];
  const media = mediaOf(doc);
  const spans = s.lanes
    .flatMap((l) => l.clips)
    .filter((c) => c.take!.active)
    .map((c) => [c.start, clipEnd(c, media, doc.sample_rate)] as [number, number])
    .sort((x, y) => x[0] - y[0]);
  const gaps: [number, number][] = [];
  let at = s.start;
  for (const [x, y] of spans) {
    if (x > at) gaps.push([at, x]);
    at = Math.max(at, y);
  }
  if (at < s.end) gaps.push([at, s.end]);
  return gaps;
}
