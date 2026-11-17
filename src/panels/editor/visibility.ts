// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * Track visibility: which tracks the Editor and Mixer list. Hidden tracks
 * still play and mix; only the lists skip them.
 */

interface ClipSpan {
  start: number;
  /** Length on the timeline, in samples. */
  length: number;
}

export interface VisTrack {
  id: string;
  hidden?: boolean;
  clips: { id: string }[];
}

/** The tracks the lists show. */
export function shownTracks<T extends { hidden?: boolean }>(
  tracks: readonly T[],
): T[] {
  return tracks.filter((t) => !t.hidden);
}

/** Ids of tracks whose hidden flag must change to leave only those `keep` accepts showing. */
export function showOnly<T extends VisTrack>(
  tracks: readonly T[],
  keep: (t: T) => boolean,
): { id: string; hidden: boolean }[] {
  const out: { id: string; hidden: boolean }[] = [];
  for (const t of tracks) {
    const hidden = !keep(t);
    if (hidden !== !!t.hidden) out.push({ id: t.id, hidden });
  }
  return out;
}

/** A saved set of hidden tracks, matched by track name so it works in other sessions too. */
export interface VisPreset {
  name: string;
  hidden: string[];
}

/** Most presets kept; the oldest is dropped when a new one is saved. */
export const MAX_VIS_PRESETS = 8;

const norm = (s: string): string => s.trim().toLowerCase();

/** Names of the tracks that are hidden now. */
export function hiddenNames(tracks: readonly { name: string; hidden?: boolean }[]): string[] {
  return tracks.filter((t) => t.hidden).map((t) => t.name);
}

/** The test for Load Visibility Preset: a track shows unless its name is in `hidden`. */
export function keepByPreset(hidden: readonly string[]): (t: { name: string }) => boolean {
  const gone = new Set(hidden.map(norm));
  return (t) => !gone.has(norm(t.name));
}

/** Adds or replaces a preset by name (case ignored) and keeps the newest MAX_VIS_PRESETS. */
export function withPreset(list: readonly VisPreset[], preset: VisPreset): VisPreset[] {
  const rest = list.filter((p) => norm(p.name) !== norm(preset.name));
  return [...rest, preset].slice(-MAX_VIS_PRESETS);
}

/** Reads saved presets back, dropping anything that is not a name with a list of names. */
export function sanitizePresets(raw: unknown): VisPreset[] {
  if (!Array.isArray(raw)) return [];
  const out: VisPreset[] = [];
  for (const p of raw) {
    if (!p || typeof p !== "object") continue;
    const { name, hidden } = p as { name?: unknown; hidden?: unknown };
    if (typeof name !== "string" || !name.trim() || !Array.isArray(hidden)) continue;
    out.push({ name: name.trim(), hidden: hidden.filter((h): h is string => typeof h === "string") });
  }
  return out.slice(-MAX_VIS_PRESETS);
}

/** True when any clip of the track overlaps [start, end). A point (start == end) uses start. */
export function hasClipIn(
  spans: readonly ClipSpan[],
  start: number,
  end: number,
): boolean {
  const hi = Math.max(end, start + 1);
  return spans.some((c) => c.start < hi && c.start + c.length > start);
}
