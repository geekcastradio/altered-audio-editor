// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * Close Dead Space: after QuickPass has taken the silence out of each track, the tracks can still
 * share stretches where nobody is talking. This finds the stretches where no track has a clip and
 * closes them down to a short pause, moving everything after them left together so the tracks stay
 * in step. Pure functions over the session; the panel turns the result into one undo step.
 *
 * Values are session samples. Only the empty time between clips is touched, so no clip is cut.
 */

import { LockedError, clipEnd, mediaOf } from "../editor/clip-edit";
import type { ClipChange, MarkerChange, Op, SessionDoc } from "../../stores/session";

export interface DeadSpace {
  /** Where the empty time starts (the end of the clip before it). */
  start: number;
  /** Where it ends (the start of the next clip). */
  end: number;
  /** How much of it is taken out. The pause that stays is `end - start - remove`. */
  remove: number;
}

export interface CloseOptions {
  /** The pause left where a stretch is closed, in samples. */
  pause: number;
  /** Stretches that would give back less than this are left alone, in samples. */
  minRemove: number;
}

/**
 * The stretches between the first and last clip where none of `trackIds` has a clip, in timeline
 * order. Muted clips and tracks still count as occupying time, so nothing is ever moved over them.
 */
export function deadSpaces(doc: SessionDoc, trackIds: Iterable<string>, o: CloseOptions): DeadSpace[] {
  const which = new Set(trackIds);
  const media = mediaOf(doc);
  const spans: [number, number][] = [];
  for (const t of doc.tracks) {
    if (!which.has(t.id)) continue;
    for (const c of t.clips) spans.push([c.start, clipEnd(c, media, doc.sample_rate)]);
  }
  spans.sort((a, b) => a[0] - b[0] || a[1] - b[1]);
  const out: DeadSpace[] = [];
  let reach = -1;
  for (const [s, e] of spans) {
    if (reach >= 0 && s > reach) {
      const remove = s - reach - Math.max(0, o.pause);
      if (remove > 0 && remove >= Math.max(1, o.minRemove)) out.push({ start: reach, end: s, remove });
    }
    reach = Math.max(reach, e);
  }
  return out;
}

export interface Closed {
  op: Op;
  spaces: DeadSpace[];
  /** Total samples taken out. */
  removed: number;
  /** Clips that moved. */
  moved: number;
}

/**
 * Closes the dead space on `trackIds` and moves their clips, and the markers after it, left. Null
 * when there is nothing to close; "blocked" when a track has MIDI clips, which a ripple does not
 * move yet (so the track would drift out of step). Throws LockedError when a clip that has to move
 * is locked.
 */
export function closeDeadSpace(doc: SessionDoc, trackIds: Iterable<string>, o: CloseOptions): Closed | "blocked" | null {
  const which = new Set(trackIds);
  for (const t of doc.tracks) if (which.has(t.id) && (t.midi_clips?.length ?? 0) > 0) return "blocked";
  const spaces = deadSpaces(doc, which, o);
  if (spaces.length === 0) return null;
  // Each stretch gives back `remove` samples from the point where the kept pause ends.
  const cuts = spaces.map((s) => ({ from: s.end - s.remove, len: s.remove }));
  const shift = (p: number) => cuts.reduce((n, c) => n + Math.min(Math.max(p - c.from, 0), c.len), 0);
  const changes: ClipChange[] = [];
  for (const t of doc.tracks) {
    if (!which.has(t.id)) continue;
    for (const c of t.clips) {
      const by = shift(c.start);
      if (by === 0) continue;
      if (c.locked) throw new LockedError();
      changes.push({ id: c.id, start: c.start - by });
    }
  }
  const marks: MarkerChange[] = [];
  for (const m of doc.markers) {
    const by = shift(m.position);
    if (by > 0) marks.push({ id: m.id, position: m.position - by });
  }
  const ops: Op[] = [];
  if (changes.length) ops.push({ op: "set_clips", changes });
  if (marks.length) ops.push({ op: "set_markers", changes: marks });
  if (ops.length === 0) return null;
  return {
    op: ops.length === 1 ? ops[0]! : { op: "batch", ops },
    spaces,
    removed: cuts.reduce((n, c) => n + c.len, 0),
    moved: changes.length,
  };
}
