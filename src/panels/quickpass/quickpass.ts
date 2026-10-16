// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * QuickPass logic with no screen in it: turning measured levels into
 * silence findings, suggesting levels from a clip, and building the
 * edits (remove, split, mark) that act on findings.
 */

import { clipEnd, mediaOf, previewTracks, rippleDeleteRange, splitClips } from "../editor/clip-edit";
import { newMarker } from "../markers/markers";
import type { ClipLevels } from "../../app/core";
import type { MarkerDoc, Op, SessionDoc } from "../../stores/session";

export interface QuickPassSettings {
  /** Audio below this level counts as quiet. */
  quietDb: number;
  /** Quiet must last this long (seconds) to be a silence. */
  minQuietSec: number;
  /** Audio at or above this level counts as speech. */
  contentDb: number;
  /** Speech shorter than this (seconds) does not break a silence. */
  minContentSec: number;
  /** What fixing a silence does. */
  action: FixAction;
  /** Remove leaves this much of each silence (milliseconds) so speech does not run together. */
  keepMs: number;
  /** Where Remove closes the gap. */
  ripple: "track" | "all";
}

export type FixAction = "remove" | "split" | "mark";

export const DEFAULTS: QuickPassSettings = { quietDb: -60, minQuietSec: 0.5, contentDb: -50, minContentSec: 0.05, action: "remove", keepMs: 100, ripple: "track" };

export interface Preset {
  name: string;
  settings: QuickPassSettings;
}

/** Starting points. Each is a full set of settings the user can then adjust. */
export const BUILTIN_PRESETS: readonly Preset[] = [
  { name: "Default", settings: DEFAULTS },
  { name: "Trim Long Silent Pauses", settings: { ...DEFAULTS, quietDb: -74, minQuietSec: 1, contentDb: -63, minContentSec: 0.05, action: "remove", keepMs: 100 } },
  { name: "Tighten Pauses Between Speakers", settings: { ...DEFAULTS, quietDb: -55, minQuietSec: 0.8, contentDb: -45, minContentSec: 0.1, action: "remove", keepMs: 300 } },
  { name: "Remove Digital Dropouts", settings: { ...DEFAULTS, quietDb: -100, minQuietSec: 0.01, contentDb: -80, minContentSec: 0.02, action: "remove", keepMs: 0 } },
  { name: "Mark Long Pauses", settings: { ...DEFAULTS, quietDb: -60, minQuietSec: 2, contentDb: -50, minContentSec: 0.1, action: "mark", keepMs: 0 } },
  { name: "Split at Pauses", settings: { ...DEFAULTS, quietDb: -60, minQuietSec: 1, contentDb: -50, minContentSec: 0.1, action: "split", keepMs: 0 } },
];

const num = (v: unknown, lo: number, hi: number, fallback: number) => (typeof v === "number" && Number.isFinite(v) ? Math.min(hi, Math.max(lo, v)) : fallback);

/** Saved settings with anything missing or out of range put right. */
export function sanitizeSettings(raw: unknown): QuickPassSettings {
  const r = (raw && typeof raw === "object" ? raw : {}) as Record<string, unknown>;
  const quietDb = num(r.quietDb, -100, -1, DEFAULTS.quietDb);
  return {
    quietDb,
    minQuietSec: num(r.minQuietSec, 0.005, 60, DEFAULTS.minQuietSec),
    contentDb: Math.max(quietDb, num(r.contentDb, -100, 0, DEFAULTS.contentDb)),
    minContentSec: num(r.minContentSec, 0.01, 10, DEFAULTS.minContentSec),
    action: r.action === "split" || r.action === "mark" ? r.action : "remove",
    keepMs: num(r.keepMs, 0, 5000, DEFAULTS.keepMs),
    ripple: r.ripple === "all" ? "all" : "track",
  };
}

export interface Finding {
  id: string;
  trackId: string;
  clipName: string;
  /** Session samples. */
  start: number;
  end: number;
}

/** Runs of windows [from, to) that are silence under the settings. */
export function silentRuns(db: readonly number[], windowSec: number, s: QuickPassSettings): [number, number][] {
  const minQuiet = Math.max(1, Math.ceil(s.minQuietSec / windowSec));
  const minContent = Math.max(1, Math.ceil(s.minContentSec / windowSec));
  // Quiet runs, then join two that only a short burst of speech separates.
  const quiet: [number, number][] = [];
  let from = -1;
  db.forEach((v, i) => {
    if (v < s.quietDb) {
      if (from < 0) from = i;
    } else if (from >= 0) {
      quiet.push([from, i]);
      from = -1;
    }
  });
  if (from >= 0) quiet.push([from, db.length]);
  const joined: [number, number][] = [];
  for (const run of quiet) {
    const last = joined[joined.length - 1];
    if (last) {
      const between = db.slice(last[1], run[0]);
      const loud = between.filter((v) => v >= s.contentDb).length;
      if (between.length < minContent || loud < minContent) {
        last[1] = run[1];
        continue;
      }
    }
    joined.push([...run]);
  }
  return joined.filter(([a, b]) => b - a >= minQuiet);
}

/** Findings for the measured clips, in timeline order per track. */
export function findSilences(
  levels: readonly ClipLevels[],
  tracks: ReadonlyMap<string, { trackId: string; name: string }>,
  s: QuickPassSettings,
  rate: number,
): Finding[] {
  const out: Finding[] = [];
  for (const l of levels) {
    const info = tracks.get(l.clipId);
    if (!info) continue;
    const windowSec = l.window / rate;
    const clipStart = l.start;
    const clipEndAt = l.start + l.db.length * l.window;
    for (const [a, b] of silentRuns(l.db, windowSec, s)) {
      const start = clipStart + a * l.window;
      const end = Math.min(clipEndAt, clipStart + b * l.window);
      out.push({ id: `${l.clipId}:${start}`, trackId: info.trackId, clipName: info.name, start, end });
    }
  }
  return out.sort((x, y) => x.start - y.start || x.trackId.localeCompare(y.trackId));
}

/**
 * Starting levels from what the clip sounds like: the quiet level sits
 * a little above its noise floor, the speech level halfway to its
 * normal speaking level. Null when there is too little to judge.
 */
export function suggestLevels(db: readonly number[]): { quietDb: number; contentDb: number } | null {
  const v = db.filter((x) => x > -120).sort((a, b) => a - b);
  if (v.length < 20) return null;
  const at = (p: number) => v[Math.min(v.length - 1, Math.floor(p * v.length))]!;
  const floor = at(0.1);
  const speech = at(0.9);
  if (speech - floor < 6) return null;
  const quietDb = Math.round(floor + (speech - floor) * 0.2);
  const contentDb = Math.max(quietDb, Math.round(floor + (speech - floor) * 0.4));
  return { quietDb, contentDb };
}

/** What an edit did to the clips, so several can be built one after another. */
export function applyLocal(doc: SessionDoc, op: Op): SessionDoc {
  switch (op.op) {
    case "batch":
      return op.ops.reduce(applyLocal, doc);
    case "remove_clips": {
      const gone = new Set(op.ids);
      return { ...doc, tracks: doc.tracks.map((t) => ({ ...t, clips: t.clips.filter((c) => !gone.has(c.id)) })) };
    }
    case "set_clips":
      return { ...doc, tracks: previewTracks(doc, op.changes) };
    case "restore_clips": {
      const by = new Map<string, typeof op.clips>();
      for (const e of op.clips) by.set(e[0], [...(by.get(e[0]) ?? []), e]);
      return { ...doc, tracks: doc.tracks.map((t) => ({ ...t, clips: [...t.clips, ...(by.get(t.id) ?? []).map((e) => e[1])] })) };
    }
    default:
      return doc;
  }
}

/** Removes the silences and closes the gaps, last one first so earlier ones keep their place. */
export function cutRange(f: Pick<Finding, "start" | "end">, keepMs: number, rate: number): [number, number] | null {
  const half = Math.round((keepMs / 1000) * rate * 0.5);
  const a = f.start + half;
  const b = f.end - half;
  return b > a ? [a, b] : null;
}

export function removeOp(doc: SessionDoc, found: readonly Finding[], s: Pick<QuickPassSettings, "ripple" | "keepMs">, newId: () => string): Op | null {
  const ops: Op[] = [];
  let cur = doc;
  for (const f of [...found].sort((a, b) => b.start - a.start)) {
    const cut = cutRange(f, s.keepMs, doc.sample_rate);
    if (!cut) continue;
    const tracks = s.ripple === "all" ? cur.tracks.map((t) => t.id) : [f.trackId];
    const op = rippleDeleteRange(cur, tracks, cut[0], cut[1], newId);
    if (!op) continue;
    ops.push(op);
    cur = applyLocal(cur, op);
  }
  return ops.length ? { op: "batch", ops } : null;
}

/** Splits each track at both edges of every silence. Locked clips are left alone. */
export function splitOp(doc: SessionDoc, found: readonly Finding[], newId: () => string): Op | null {
  const ops: Op[] = [];
  let cur = doc;
  for (const f of found) {
    const t = cur.tracks.find((x) => x.id === f.trackId);
    if (!t) continue;
    for (const at of [f.end, f.start]) {
      const op = splitClips(cur, t.clips.map((c) => c.id), at, newId);
      if (!op) continue;
      ops.push(op);
      cur = applyLocal(cur, op);
    }
  }
  return ops.length ? { op: "batch", ops } : null;
}

/** The clips that now sit exactly on the findings, to leave selected after a split. */
export function clipsOn(doc: SessionDoc, found: readonly Finding[]): string[] {
  const media = mediaOf(doc);
  const ids: string[] = [];
  for (const f of found) {
    const t = doc.tracks.find((x) => x.id === f.trackId);
    for (const c of t?.clips ?? []) if (c.start === f.start && clipEnd(c, media, doc.sample_rate) === f.end) ids.push(c.id);
  }
  return ids;
}

/** One range marker per silence. */
export function markersFor(doc: SessionDoc, found: readonly Finding[], newId: () => string): MarkerDoc[] {
  const taken = doc.markers.filter((m) => /^Silence \d+$/.test(m.name)).length;
  return [...found]
    .sort((a, b) => a.start - b.start)
    .map((f, i) => newMarker(newId(), `Silence ${taken + i + 1}`, f.start, f.end - f.start));
}

/** After `gone` is removed, where the other findings sit. Those inside it are dropped. */
export function shiftFindings(rest: readonly Finding[], gone: Finding, ripple: "track" | "all", keepMs = 0, rate = 48000): Finding[] {
  const cut = cutRange(gone, keepMs, rate) ?? [gone.start, gone.start];
  const len = cut[1] - cut[0];
  gone = { ...gone, start: cut[0], end: cut[1] };
  return rest
    .filter((f) => f.id !== gone.id && !((ripple === "all" || f.trackId === gone.trackId) && f.start < gone.end && f.end > gone.start))
    .map((f) => (f.start >= gone.end && (ripple === "all" || f.trackId === gone.trackId) ? { ...f, start: f.start - len, end: f.end - len } : f));
}
