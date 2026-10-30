// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/** The Dynamics window's graph: output level against input level for the compressor's settings. */

export interface Curve {
  threshold: number;
  ratio: number;
  knee: number;
  makeup: number;
}

/** Output level (dB) for an input level (dB), with a soft knee centred on the threshold. */
export function transfer(inDb: number, c: Curve): number {
  const { threshold: t, ratio: r, knee: k, makeup: m } = c;
  const over = inDb - t;
  let out: number;
  if (k > 0 && Math.abs(over) <= k / 2) {
    const x = over + k / 2;
    out = inDb + ((1 / r - 1) * x * x) / (2 * k);
  } else if (over > 0) {
    out = t + over / r;
  } else {
    out = inDb;
  }
  return out + m;
}

/** Points along the curve from `lo` to `hi` dB in, for drawing. */
export function curvePoints(c: Curve, lo = -60, hi = 0, steps = 60): [number, number][] {
  return Array.from({ length: steps + 1 }, (_, i) => {
    const x = lo + ((hi - lo) * i) / steps;
    return [x, transfer(x, c)];
  });
}

/** The ratio that gives a stated threshold-to-output slope, clamped to the control's range. */
export const ratioFromSlope = (slope: number, min = 1, max = 20): number => Math.min(max, Math.max(min, slope <= 0.0001 ? max : 1 / slope));
