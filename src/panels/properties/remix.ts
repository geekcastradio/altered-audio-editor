// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * Properties > Remix, numbers only: how long a Fit Music plan plays and
 * how a length reads. The plan's pieces overlap by a short crossfade at
 * each join, the same way fitOps lays them on the track.
 */

import { FIT_XFADE, type FitPlan } from "../editor/musicfit";

/** Seconds the plan plays once applied. */
export function planDuration(p: FitPlan): number {
  if (p.segments.length === 0) return 0;
  const body = p.segments.reduce((sum, s) => sum + Math.max(0, s.end - s.start), 0);
  return Math.max(0, (body - (p.segments.length - 1) * FIT_XFADE) * p.stretch);
}

/** "42.500" under a minute, "1:05.250" from a minute up. */
export function formatSeconds(s: number): string {
  const safe = Number.isFinite(s) && s > 0 ? s : 0;
  const m = Math.floor(safe / 60);
  const rest = (safe - m * 60).toFixed(3).padStart(6, "0");
  return m > 0 ? `${m}:${rest}` : safe.toFixed(3);
}
