// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * Match EQ: moves the tone of one voice toward another.
 *
 * Both voices are measured as their long-term average spectrum (the
 * Analysis report already has it). The difference between them, smoothed
 * and levelled so the overall volume does not change, is the curve to
 * add. That curve is then fitted with what the Parametric EQ has: a low
 * shelf, a high shelf and three bands, so the result is an ordinary,
 * editable EQ and not a black box.
 *
 * The method is a greedy least-squares fit using the same filter shapes
 * the EQ runs (the published audio EQ cookbook formulas).
 */

export interface Spectrum {
  hz: readonly number[];
  db: readonly number[];
}

/** Where the match is trusted. Outside, the curve is held flat. */
export const MATCH_LOW_HZ = 80;
export const MATCH_HIGH_HZ = 12_000;
/** Largest boost or cut asked of any band, dB. */
export const MAX_GAIN_DB = 12;
/** Bands smaller than this are left at zero, dB. */
const MIN_BAND_DB = 0.5;

type Shape = "peak" | "lowShelf" | "highShelf";

/** Magnitude in dB at `f` of one filter, from the cookbook coefficients (shelf slope 1 as the EQ uses). */
export function responseDb(shape: Shape, rate: number, f0: number, q: number, gainDb: number, f: number): number {
  const w0 = (2 * Math.PI * Math.min(f0, rate * 0.49)) / rate;
  const sw = Math.sin(w0);
  const cw = Math.cos(w0);
  const a = Math.pow(10, gainDb / 40);
  let b: [number, number, number];
  let d: [number, number, number];
  if (shape === "peak") {
    const alpha = sw / (2 * Math.max(q, 0.01));
    b = [1 + alpha * a, -2 * cw, 1 - alpha * a];
    d = [1 + alpha / a, -2 * cw, 1 - alpha / a];
  } else {
    const alpha = (sw / 2) * Math.sqrt(2);
    const k = 2 * Math.sqrt(a) * alpha;
    if (shape === "lowShelf") {
      b = [a * (a + 1 - (a - 1) * cw + k), 2 * a * (a - 1 - (a + 1) * cw), a * (a + 1 - (a - 1) * cw - k)];
      d = [a + 1 + (a - 1) * cw + k, -2 * (a - 1 + (a + 1) * cw), a + 1 + (a - 1) * cw - k];
    } else {
      b = [a * (a + 1 + (a - 1) * cw + k), -2 * a * (a - 1 + (a + 1) * cw), a * (a + 1 + (a - 1) * cw - k)];
      d = [a + 1 - (a - 1) * cw + k, 2 * (a - 1 - (a + 1) * cw), a + 1 - (a - 1) * cw - k];
    }
  }
  const w = (2 * Math.PI * f) / rate;
  const c1 = Math.cos(w);
  const s1 = Math.sin(w);
  const c2 = Math.cos(2 * w);
  const s2 = Math.sin(2 * w);
  const nr = b[0] + b[1] * c1 + b[2] * c2;
  const ni = -(b[1] * s1 + b[2] * s2);
  const dr = d[0] + d[1] * c1 + d[2] * c2;
  const di = -(d[1] * s1 + d[2] * s2);
  return 10 * Math.log10(Math.max((nr * nr + ni * ni) / (dr * dr + di * di), 1e-24));
}

const RATE = 48_000;

/** Reference interpolated (in dB, on a log axis) onto `hz`. */
function resample(from: Spectrum, hz: readonly number[]): number[] {
  const out: number[] = [];
  let j = 0;
  for (const f of hz) {
    while (j < from.hz.length - 2 && from.hz[j + 1]! < f) j++;
    const f0 = from.hz[j]!;
    const f1 = from.hz[j + 1] ?? f0;
    const v0 = from.db[j]!;
    const v1 = from.db[j + 1] ?? v0;
    const t = f1 > f0 ? Math.min(1, Math.max(0, Math.log(f / f0) / Math.log(f1 / f0))) : 0;
    out.push(v0 + (v1 - v0) * t);
  }
  return out;
}

/** Moving average over +/- `oct` octaves on the log axis. */
function smooth(hz: readonly number[], v: readonly number[], oct: number): number[] {
  return v.map((_, i) => {
    const lo = hz[i]! / Math.pow(2, oct);
    const hi = hz[i]! * Math.pow(2, oct);
    let s = 0;
    let n = 0;
    for (let k = 0; k < hz.length; k++) {
      if (hz[k]! >= lo && hz[k]! <= hi) {
        s += v[k]!;
        n++;
      }
    }
    return n ? s / n : v[i]!;
  });
}

export interface MatchFit {
  /** Parametric EQ settings (keys of altaudio.parametric_eq). */
  params: Record<string, number>;
  /** The curve asked for on the source's grid, dB. */
  target: number[];
  /** What the EQ settings give on that grid, dB. */
  fitted: number[];
  hz: number[];
  /** RMS difference from the wanted curve inside the trusted range: before and after the fit, dB. */
  errorBefore: number;
  errorAfter: number;
}

const BAND_RANGES: [number, number][] = [
  [120, 700],
  [500, 3500],
  [2000, 10_000],
];
const Q_CHOICES = [0.7, 1.2, 2.2];

/** The Parametric EQ settings that move `source` toward `reference`. */
export function matchEq(source: Spectrum, reference: Spectrum): MatchFit | null {
  const hz = [...source.hz];
  if (hz.length < 8 || source.db.length !== hz.length) return null;
  const ref = resample(reference, hz);
  const inRange = (f: number) => f >= MATCH_LOW_HZ && f <= MATCH_HIGH_HZ;
  // The wanted curve: what the reference has that the source lacks.
  const nearest = (f: number) => hz.reduce((best, h, k) => (Math.abs(Math.log(h / f)) < Math.abs(Math.log(hz[best]! / f)) ? k : best), 0);
  const raw = hz.map((f) => {
    // Outside the trusted range the curve holds the value at the edge.
    const j = nearest(Math.min(Math.max(f, MATCH_LOW_HZ), MATCH_HIGH_HZ));
    return ref[j]! - source.db[j]!;
  });
  let target = smooth(hz, raw, 1 / 6);
  // Keep the overall level: the speech band (200 Hz to 5 kHz) averages zero.
  const band = hz.map((f, i) => (f >= 200 && f <= 5000 ? i : -1)).filter((i) => i >= 0);
  const mean = band.length ? band.reduce((s, i) => s + target[i]!, 0) / band.length : 0;
  target = target.map((v) => Math.max(-MAX_GAIN_DB, Math.min(MAX_GAIN_DB, v - mean)));

  const weight = hz.map((f) => (inRange(f) ? 1 : 0));
  const rms = (r: readonly number[]) => {
    let s = 0;
    let n = 0;
    r.forEach((v, i) => {
      s += weight[i]! * v * v;
      n += weight[i]!;
    });
    return n ? Math.sqrt(s / n) : 0;
  };
  const errorBefore = rms(target);
  const residual = [...target];
  const fitted = hz.map(() => 0);
  const take = (shape: Shape, f0: number, q: number, gain: number) => {
    hz.forEach((f, i) => {
      const r = responseDb(shape, RATE, f0, q, gain, f);
      fitted[i]! += r;
      residual[i]! -= r;
    });
  };

  const params: Record<string, number> = { hp_on: 0, lp_on: 0 };
  // The shelves, one at a time: least squares on the unit-gain shape.
  const shelf = (shape: Shape, f0: number, key: string) => {
    const unit = hz.map((f) => responseDb(shape, RATE, f0, 1, 6, f) / 6);
    let num = 0;
    let den = 0;
    hz.forEach((_, i) => {
      num += weight[i]! * residual[i]! * unit[i]!;
      den += weight[i]! * unit[i]! * unit[i]!;
    });
    const g = den > 0 ? Math.max(-MAX_GAIN_DB, Math.min(MAX_GAIN_DB, num / den)) : 0;
    const gain = Math.abs(g) < MIN_BAND_DB ? 0 : Math.round(g * 10) / 10;
    params[`${key}_freq`] = f0;
    params[`${key}_gain`] = gain;
    if (gain !== 0) take(shape, f0, 1, gain);
  };
  shelf("lowShelf", 150, "ls");
  shelf("highShelf", 7000, "hs");

  // The three bands: each lands on the biggest remaining difference in its range.
  BAND_RANGES.forEach(([lo, hi], n) => {
    const key = `b${n + 1}`;
    const sm = smooth(hz, residual, 1 / 8);
    let best = -1;
    hz.forEach((f, i) => {
      if (f >= lo && f <= hi && (best < 0 || Math.abs(sm[i]!) > Math.abs(sm[best]!))) best = i;
    });
    if (best < 0 || Math.abs(sm[best]!) < MIN_BAND_DB) {
      params[`${key}_gain`] = 0;
      return;
    }
    const f0 = hz[best]!;
    const gain = Math.round(Math.max(-MAX_GAIN_DB, Math.min(MAX_GAIN_DB, sm[best]! * 1.05)) * 10) / 10;
    // Pick the width that leaves the least error.
    let bestQ = Q_CHOICES[0]!;
    let bestErr = Infinity;
    for (const q of Q_CHOICES) {
      const trial = residual.map((r, i) => r - responseDb("peak", RATE, f0, q, gain, hz[i]!));
      const e = rms(trial);
      if (e < bestErr) {
        bestErr = e;
        bestQ = q;
      }
    }
    params[`${key}_freq`] = Math.round(f0);
    params[`${key}_gain`] = gain;
    params[`${key}_q`] = bestQ;
    take("peak", f0, bestQ, gain);
  });

  return { params, target, fitted, hz, errorBefore, errorAfter: rms(residual) };
}
