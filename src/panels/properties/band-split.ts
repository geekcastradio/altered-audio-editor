// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * Split into Frequency Bands: a clip becomes two or three tracks, one per
 * band (Low, Mid, High), so each band can be processed on its own. Every
 * new track plays a copy of the clip through two 2nd-order Butterworth
 * Precision Filters at each edge, which makes 4th-order Linkwitz-Riley
 * crossovers: the bands add back to the original level. With two edges
 * they must be at least six times apart (about two and a half octaves),
 * or the bands add up to a dip of more than 0.4 dB near an edge. The
 * original clips are muted, not removed.
 */

import type { ClipChange, ClipSpec, EffectSlotDoc, Op, SessionDoc, TrackChanges } from "../../stores/session";
import type { Located } from "../editor/clip-edit";

export const BAND_MIN_HZ = 40;
export const BAND_MAX_HZ = 16_000;
/** Two edges closer than this ratio make the bands add up unevenly. */
export const BAND_MIN_RATIO = 6;
const NAMES = [["Low", "High"], ["Low", "Mid", "High"]] as const;

/** Reads "200, 2000" (commas, spaces or semicolons) into edges in Hz; null when it is not numbers. */
export function parseEdges(text: string): number[] | null {
  const parts = text.split(/[\s,;]+/).filter(Boolean);
  if (parts.length === 0) return null;
  const nums = parts.map((p) => Number(p.replace(/hz$/i, "")));
  return nums.every((n) => Number.isFinite(n)) ? nums : null;
}

/** What is wrong with the edges, or null when they can be used. */
export function checkEdges(edges: readonly number[]): string | null {
  if (edges.length < 1 || edges.length > 2) return "Give one frequency for two bands, or two for three bands.";
  if (edges.some((e) => e < BAND_MIN_HZ || e > BAND_MAX_HZ)) return `Each frequency must be between ${BAND_MIN_HZ} and ${BAND_MAX_HZ} Hz.`;
  if (edges.length === 2) {
    const [a, b] = edges as [number, number];
    if (b <= a) return "Give the lower frequency first.";
    if (b / a < BAND_MIN_RATIO) return `The two frequencies must be at least ${BAND_MIN_RATIO} times apart, or the bands will not add back evenly.`;
  }
  return null;
}

/** The Precision Filter slots of band `k` (0 is the lowest): a pair of filters at each edge it touches. */
export function bandSlots(k: number, edges: readonly number[], newId: () => string): EffectSlotDoc[] {
  const slot = (mode: number, freq: number): EffectSlotDoc => ({
    id: newId(),
    effect: "altaudio.precision_filter",
    enabled: true,
    // Butterworth, 2nd order. mode 0 is low-pass, 1 is high-pass.
    params: { design: 0, mode, order: 0, freq },
    preset: "Frequency band",
  });
  const out: EffectSlotDoc[] = [];
  if (k > 0) out.push(slot(1, edges[k - 1]!), slot(1, edges[k - 1]!));
  if (k < edges.length) out.push(slot(0, edges[k]!), slot(0, edges[k]!));
  return out;
}

/** Clips among `found` that can be split: not reversed (a reversed clip plays from a separate copy). */
export function bandSplittable(found: readonly Located[]): Located[] {
  return found.filter((f) => !f.clip.reversed && !f.clip.locked);
}

/**
 * The edit that puts each selected clip's bands on new tracks right under its
 * track, or null when no clip can be split.
 */
export function bandSplitOp(doc: SessionDoc, found: readonly Located[], edges: readonly number[], uuid: () => string): Op | null {
  const usable = bandSplittable(found);
  if (usable.length === 0 || checkEdges(edges) !== null) return null;
  const labels = NAMES[edges.length - 1]!;

  const byTrack = new Map<number, Located[]>();
  for (const f of usable) byTrack.set(f.trackIndex, [...(byTrack.get(f.trackIndex) ?? []), f]);
  const trackOrder = [...byTrack.keys()].sort((a, b) => a - b);

  const ops: Op[] = [];
  let added = 0;
  for (const ti of trackOrder) {
    const clips = byTrack.get(ti)!;
    const source = clips[0]!.track;
    const trackIds = labels.map(() => uuid());
    labels.forEach((label, k) => {
      ops.push({ op: "add_track", id: trackIds[k]!, name: `${source.name} ${label}`, stereo: true, index: ti + added + 1 + k });
    });
    added += labels.length;

    const specs: ClipSpec[][] = labels.map(() => []);
    const changes: ClipChange[] = [];
    for (const { clip } of clips) {
      labels.forEach((label, k) => {
        const id = uuid();
        specs[k]!.push({ id, name: `${clip.name} ${label}`, media_id: clip.media_id, start: clip.start, source_in: clip.source_in, source_out: clip.source_out });
        const change: ClipChange = { id, gain_db: clip.gain_db, fade_in: clip.fade_in, fade_out: clip.fade_out };
        if (clip.muted) change.muted = true;
        if (clip.audio_type) change.audio_type = clip.audio_type;
        if (clip.channel_mix) change.channel_mix = clip.channel_mix;
        if (clip.stretch) change.stretch = clip.stretch;
        if (clip.looped) change.looped = clip.looped;
        changes.push(change);
      });
    }
    specs.forEach((list, k) => {
      if (list.length > 0) ops.push({ op: "add_clips", track_id: trackIds[k]!, clips: list });
    });
    if (changes.length > 0) ops.push({ op: "set_clips", changes });

    labels.forEach((_, k) => {
      const t: TrackChanges = {};
      if (source.color) t.color = source.color;
      if (Object.keys(t).length > 0) ops.push({ op: "set_track", id: trackIds[k]!, changes: t });
      ops.push({
        op: "set_effects",
        target: { kind: "track", id: trackIds[k]! },
        chain: { enabled: true, input_gain_db: 0, output_gain_db: 0, mix: 1, slots: bandSlots(k, edges, uuid) },
      });
    });

    // The originals stay, muted, so nothing plays twice and nothing is lost.
    ops.push({ op: "set_clips", changes: clips.map((f) => ({ id: f.clip.id, muted: true })) });
  }
  // `doc` is read only to keep the signature the same as the other split edits.
  void doc;
  return ops.length === 1 ? ops[0]! : { op: "batch", ops };
}
