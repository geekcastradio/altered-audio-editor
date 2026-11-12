// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * Multitrack > Auto Mic Mix and Talk Time: the shapes the analysis sends
 * back, the one undoable edit that writes the volume lanes, and the plain
 * words for the result. The analysis itself runs in the backend.
 */

import { newLane, putLane, targetKey, type AutoOwner } from "../panels/editor/automation";
import type { EnvelopeDoc, EnvelopePointDoc, Op, SessionDoc, TrackDoc } from "../stores/session";

export interface MixOptions {
  openDb?: number;
  marginDb?: number;
  reduceDb?: number;
  fadeMs?: number;
  attackMs?: number;
  holdMs?: number;
}

export interface TrackMix {
  track: string;
  name: string;
  points: { position: number; value: number; curve: "linear" | "hold" | "spline" }[];
  talkSeconds: number;
  share: number;
}

export interface MixAnalysis {
  tracks: TrackMix[];
  totalSeconds: number;
}

/** What the dialog offers for how far a quiet mic is turned down. */
export const REDUCE_CHOICES: readonly { id: string; label: string; db: number }[] = [
  { id: "gentle", label: "Gentle (8 dB)", db: 8 },
  { id: "standard", label: "Standard (12 dB)", db: 12 },
  { id: "firm", label: "Firm (18 dB)", db: 18 },
];

/** Audio tracks that have something on them: the candidates for a mic. */
export function micTracks(doc: SessionDoc): TrackDoc[] {
  return doc.tracks.filter((t) => t.kind !== "midi" && t.clips.length > 0);
}

/** Minutes and seconds, or hours when long. */
export function formatDuration(seconds: number): string {
  const s = Math.max(0, Math.round(seconds));
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const r = s % 60;
  const two = (n: number) => String(n).padStart(2, "0");
  return h > 0 ? `${h}:${two(m)}:${two(r)}` : `${m}:${two(r)}`;
}

/** One line per voice for the result. */
export function describeTalk(a: MixAnalysis): string[] {
  return [...a.tracks]
    .sort((x, y) => y.talkSeconds - x.talkSeconds)
    .map((t) => `${t.name}: ${formatDuration(t.talkSeconds)} (${Math.round(t.share * 100)}%)`);
}

/** The volume lane for a track: points from the analysis, enabled, replacing any volume lane there. */
function withVolume(track: TrackDoc, points: EnvelopePointDoc[]): EnvelopeDoc[] {
  const owner = track as unknown as AutoOwner;
  const existing = (track.envelopes ?? []).find((e) => targetKey(e.target) === targetKey({ type: "volume" }));
  return putLane(owner, { ...(existing ?? newLane({ type: "volume" })), enabled: true, points });
}

/**
 * Writes each analysed track's gain curve to its volume lane, leaving other
 * lanes alone. One edit, so a single undo puts every fader lane back.
 */
export function mixOps(doc: SessionDoc, a: MixAnalysis): Op | null {
  const ops: Op[] = [];
  for (const m of a.tracks) {
    const track = doc.tracks.find((t) => t.id === m.track);
    if (!track || m.points.length === 0) continue;
    ops.push({ op: "set_envelopes", owner: track.id, envelopes: withVolume(track, m.points) });
  }
  if (ops.length === 0) return null;
  return ops.length === 1 ? ops[0]! : { op: "batch", ops };
}

/** One line for the notice after the mix is written. */
export function describeMix(a: MixAnalysis): string {
  const n = a.tracks.length;
  return `Wrote volume lanes for ${n} tracks. Quiet mics drop while someone else talks; undo takes it all back. Adjust the lanes by hand where a word sounds off.`;
}
