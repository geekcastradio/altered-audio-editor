// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * Isolated short clips: after QuickPass, a stray cough, bump or breath can survive as a very
 * short clip with nothing around it on its own track. This lists them so the editor can hear each
 * one and decide. Nothing is removed here; the dialog asks first.
 *
 * Values are session samples.
 */

import { clipEnd, mediaOf } from "../editor/clip-edit";
import type { Op, SessionDoc } from "../../stores/session";

export interface IsolatedOptions {
  /** A clip shorter than this counts as short. */
  maxLen: number;
  /** A short clip counts as isolated when the clips next to it on its track are at least this far away. */
  gap: number;
}

export interface IsolatedClip {
  id: string;
  trackId: string;
  trackName: string;
  clipName: string;
  start: number;
  end: number;
  /** Another track has audio under it, so it may be bleed or an interjection rather than a stray noise. */
  overlapsOthers: boolean;
}

/** Short clips with room on both sides of their own track, in timeline order. Locked and muted clips are skipped. */
export function isolatedClips(doc: SessionDoc, o: IsolatedOptions): IsolatedClip[] {
  const media = mediaOf(doc);
  const rate = doc.sample_rate;
  const spans = doc.tracks.map((t) => t.clips.map((c) => ({ c, start: c.start, end: clipEnd(c, media, rate) })).sort((a, b) => a.start - b.start));
  const out: IsolatedClip[] = [];
  doc.tracks.forEach((t, ti) => {
    const mine = spans[ti]!;
    mine.forEach((s, i) => {
      if (s.c.locked || s.c.muted) return;
      if (s.end - s.start >= o.maxLen) return;
      const before = i > 0 ? Math.max(...mine.slice(0, i).map((x) => x.end)) : -Infinity;
      const after = i < mine.length - 1 ? mine[i + 1]!.start : Infinity;
      if (s.start - before < o.gap || after - s.end < o.gap) return;
      const overlapsOthers = spans.some((other, oi) => oi !== ti && other.some((x) => x.start < s.end && x.end > s.start));
      out.push({ id: s.c.id, trackId: t.id, trackName: t.name, clipName: s.c.name, start: s.start, end: s.end, overlapsOthers });
    });
  });
  return out.sort((a, b) => a.start - b.start || a.trackName.localeCompare(b.trackName));
}

/** One op that removes the chosen clips and leaves the gaps; null when none are left to remove. Locked clips stay. */
export function removeIsolated(doc: SessionDoc, ids: Iterable<string>): Op | null {
  const want = new Set(ids);
  const gone = doc.tracks.flatMap((t) => t.clips.filter((c) => want.has(c.id) && !c.locked).map((c) => c.id));
  return gone.length ? { op: "remove_clips", ids: gone } : null;
}
