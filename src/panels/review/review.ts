// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/** The marks a host drops while recording (Cut this, Retake, Great bit) and moving through them in time order. */

import type { MarkerDoc } from "../../stores/session";

export type ReviewKind = "cut" | "retake" | "great";
export type ReviewFilter = ReviewKind | "all";

export const REVIEW_LABELS: Record<ReviewKind, string> = {
  cut: "Cut this",
  retake: "Retake",
  great: "Great bit",
};

/** Which review mark a marker is, by its name ("Cut this 3" is a cut), or null for any other marker. */
export function reviewKind(m: Pick<MarkerDoc, "name">): ReviewKind | null {
  const n = m.name.trim().toLowerCase();
  if (n.startsWith("cut this")) return "cut";
  if (n.startsWith("retake")) return "retake";
  if (n.startsWith("great bit")) return "great";
  return null;
}

/** The review marks that pass the filter, in time order. */
export function reviewMarks(markers: readonly MarkerDoc[], filter: ReviewFilter = "all"): MarkerDoc[] {
  return markers
    .filter((m) => {
      const k = reviewKind(m);
      return k !== null && (filter === "all" || filter === k);
    })
    .sort((a, b) => a.position - b.position);
}

/** How many review marks of each kind there are. */
export function reviewCounts(markers: readonly MarkerDoc[]): Record<ReviewKind, number> {
  const out: Record<ReviewKind, number> = { cut: 0, retake: 0, great: 0 };
  for (const m of markers) {
    const k = reviewKind(m);
    if (k) out[k]++;
  }
  return out;
}

/**
 * The mark after (or before) a spot, skipping any at the spot itself. `marks` must be in time order.
 * Null when there is none that way.
 */
export function stepMark(marks: readonly MarkerDoc[], at: number, dir: 1 | -1): MarkerDoc | null {
  if (dir === 1) return marks.find((m) => m.position > at) ?? null;
  for (let i = marks.length - 1; i >= 0; i--) if (marks[i]!.position < at) return marks[i]!;
  return null;
}
