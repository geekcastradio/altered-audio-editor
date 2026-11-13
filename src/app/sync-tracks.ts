// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * Multitrack > Sync Tracks: what the measuring engine reports, how to say it in plain words, and
 * the edit that moves the second track into line. The measuring itself runs in the Rust core.
 */

import { LockedError } from "../panels/editor/clip-edit";
import type { ClipChange, Op, SessionDoc } from "../stores/session";

export interface TrackSync {
  /** Session samples the second track trails the first by (negative: it starts early). */
  offset: number;
  /** How much the gap grows, in parts per million of the first track's time. */
  driftPpm: number;
  /** 0 to 1. */
  confidence: number;
  /** The second track is upside down against the first. */
  inverted: boolean;
  windows: number;
}

/** Drift below this is left unmentioned: under 20 ms over an hour. */
export const DRIFT_QUIET_PPM = 5;

/** Moves every clip on the track so the offset closes. Null when there is nothing to move. */
export function moveTrack(doc: SessionDoc, trackId: string, offset: number): Op | "before-start" | "blocked" | null {
  const track = doc.tracks.find((t) => t.id === trackId);
  if (!track) return null;
  if ((track.midi_clips?.length ?? 0) > 0) return "blocked";
  const shift = Math.round(offset);
  if (shift === 0 || track.clips.length === 0) return null;
  const changes: ClipChange[] = [];
  for (const c of track.clips) {
    const start = c.start - shift;
    if (start < 0) return "before-start";
    if (c.locked) throw new LockedError();
    changes.push({ id: c.id, start });
  }
  return { op: "set_clips", changes };
}

const seconds = (samples: number, rate: number) => Math.abs(samples) / rate;

/** The result in plain words, one line each. */
export function describeSync(r: TrackSync, rate: number): string[] {
  const out: string[] = [];
  const secs = seconds(r.offset, rate);
  if (Math.round(r.offset) === 0) out.push("The tracks already line up.");
  else if (r.offset > 0) out.push(`The second track is ${secs.toFixed(3)} s late. Moving it earlier by that much lines it up.`);
  else out.push(`The second track starts ${secs.toFixed(3)} s early. Moving it later by that much lines it up.`);
  const perHour = Math.abs(r.driftPpm) * 3.6;
  if (Math.abs(r.driftPpm) >= DRIFT_QUIET_PPM) {
    out.push(
      `The gap ${r.driftPpm > 0 ? "grows" : "shrinks"} by about ${perHour.toFixed(0)} ms every hour (${Math.abs(r.driftPpm).toFixed(0)} ppm). Moving the track fixes the start; the two will slip apart over a long recording.`,
    );
  } else {
    out.push("The two clocks stay together, so a single move holds for the whole recording.");
  }
  if (r.inverted) out.push("The second track is also upside down against the first. Flip its polarity if they sound thin together.");
  return out;
}

/** How much to trust it, in a word. */
export function trust(r: TrackSync): "strong" | "fair" | "weak" {
  return r.confidence >= 0.5 ? "strong" : r.confidence >= 0.25 ? "fair" : "weak";
}
