// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * The spectrum view's frequency axis and colors. The axis is log spaced
 * from MIN_HZ to the Nyquist frequency and must match
 * `src-tauri/src/dsp/spectral.rs`.
 */

export const MIN_HZ = 30;

/** Frequency at `frac` up the axis (0 bottom, 1 top). */
export function freqAt(frac: number, nyquist: number): number {
  const top = Math.max(nyquist, MIN_HZ * 2);
  return MIN_HZ * Math.pow(top / MIN_HZ, Math.min(1, Math.max(0, frac)));
}

/** How far up the axis `hz` sits, 0 to 1. */
export function fracOf(hz: number, nyquist: number): number {
  const top = Math.max(nyquist, MIN_HZ * 2);
  if (hz <= MIN_HZ) return 0;
  return Math.min(1, Math.log(hz / MIN_HZ) / Math.log(top / MIN_HZ));
}

/** Color stops from silent to loud: near black, indigo, teal, amber, warm white. */
const STOPS: [number, [number, number, number]][] = [
  [0, [8, 8, 13]],
  [0.3, [42, 26, 94]],
  [0.55, [38, 128, 140]],
  [0.8, [222, 178, 70]],
  [1, [255, 250, 228]],
];

/** 256 RGB triples, flattened. */
export const SPECTRUM_LUT: Uint8Array = (() => {
  const out = new Uint8Array(256 * 3);
  for (let i = 0; i < 256; i++) {
    const t = i / 255;
    let k = 0;
    while (k < STOPS.length - 2 && t > STOPS[k + 1]![0]) k++;
    const [t0, c0] = STOPS[k]!;
    const [t1, c1] = STOPS[k + 1]!;
    const u = (t - t0) / (t1 - t0);
    for (let j = 0; j < 3; j++) out[i * 3 + j] = Math.round(c0[j]! + (c1[j]! - c0[j]!) * u);
  }
  return out;
})();

/** One part of a spectral selection. Times are samples; heights are axis positions (0 bottom, 1 top). */
export type SpectralShape =
  | { kind: "box"; start: number; end: number; bottom: number; top: number }
  | { kind: "lasso"; points: [number, number][] }
  | { kind: "brush"; points: [number, number][]; rx: number; ry: number };

/** What the core's spectral edit takes: weights 0 to 255 over time and the axis. */
export interface SpectralMask {
  start: number;
  spc: number;
  cols: number;
  rows: number;
  frac_top: number;
  frac_per_row: number;
  data: number[];
}

/** Time and axis extent of shapes. */
export function shapesBounds(shapes: readonly SpectralShape[]): { t0: number; t1: number; f0: number; f1: number } | null {
  let t0 = Infinity;
  let t1 = -Infinity;
  let f0 = Infinity;
  let f1 = -Infinity;
  for (const s of shapes) {
    if (s.kind === "box") {
      t0 = Math.min(t0, s.start);
      t1 = Math.max(t1, s.end);
      f0 = Math.min(f0, s.bottom);
      f1 = Math.max(f1, s.top);
    } else {
      const rx = s.kind === "brush" ? s.rx : 0;
      const ry = s.kind === "brush" ? s.ry : 0;
      for (const [t, f] of s.points) {
        t0 = Math.min(t0, t - rx);
        t1 = Math.max(t1, t + rx);
        f0 = Math.min(f0, f - ry);
        f1 = Math.max(f1, f + ry);
      }
    }
  }
  if (!(t1 > t0) || !(f1 > f0)) return null;
  return { t0: Math.max(0, t0), t1, f0: Math.max(0, f0), f1: Math.min(1, f1) };
}

function inPolygon(x: number, y: number, pts: readonly [number, number][]): boolean {
  let inside = false;
  for (let i = 0, j = pts.length - 1; i < pts.length; j = i++) {
    const [xi, yi] = pts[i]!;
    const [xj, yj] = pts[j]!;
    if (yi > y !== yj > y && x < ((xj - xi) * (y - yi)) / (yj - yi) + xi) inside = !inside;
  }
  return inside;
}

/** Turns shapes into a weight grid for the core. Null when nothing is selected. */
export function rasterize(shapes: readonly SpectralShape[], maxCols = 800, maxRows = 300): SpectralMask | null {
  const b = shapesBounds(shapes);
  if (!b) return null;
  const cols = Math.max(1, Math.min(maxCols, Math.ceil(b.t1 - b.t0)));
  const spc = (b.t1 - b.t0) / cols;
  const rows = Math.max(1, Math.min(maxRows, Math.ceil((b.f1 - b.f0) * 1000)));
  const fpr = (b.f1 - b.f0) / rows;
  const data = new Array<number>(cols * rows).fill(0);
  const tAt = (c: number) => b.t0 + (c + 0.5) * spc;
  const fAt = (r: number) => b.f1 - (r + 0.5) * fpr;
  for (const s of shapes) {
    if (s.kind === "box") {
      const c0 = Math.max(0, Math.floor((s.start - b.t0) / spc));
      const c1 = Math.min(cols - 1, Math.ceil((s.end - b.t0) / spc) - 1);
      const r0 = Math.max(0, Math.floor((b.f1 - s.top) / fpr));
      const r1 = Math.min(rows - 1, Math.ceil((b.f1 - s.bottom) / fpr) - 1);
      for (let r = r0; r <= r1; r++) for (let c = c0; c <= c1; c++) data[r * cols + c] = 255;
    } else if (s.kind === "lasso") {
      if (s.points.length < 3) continue;
      for (let r = 0; r < rows; r++) for (let c = 0; c < cols; c++) if (inPolygon(tAt(c), fAt(r), s.points)) data[r * cols + c] = 255;
    } else {
      // Brush: an ellipse swept along each segment.
      const pts = s.points.length === 1 ? [s.points[0]!, s.points[0]!] : s.points;
      for (let i = 0; i + 1 < pts.length; i++) {
        const [ax, ay] = pts[i]!;
        const [bx, by] = pts[i + 1]!;
        const c0 = Math.max(0, Math.floor((Math.min(ax, bx) - s.rx - b.t0) / spc));
        const c1 = Math.min(cols - 1, Math.ceil((Math.max(ax, bx) + s.rx - b.t0) / spc));
        const r0 = Math.max(0, Math.floor((b.f1 - Math.max(ay, by) - s.ry) / fpr));
        const r1 = Math.min(rows - 1, Math.ceil((b.f1 - Math.min(ay, by) + s.ry) / fpr));
        // Work in units where the brush is a unit circle.
        const ux = (bx - ax) / s.rx;
        const uy = (by - ay) / s.ry;
        const len2 = ux * ux + uy * uy;
        for (let r = r0; r <= r1; r++) {
          for (let c = c0; c <= c1; c++) {
            const px = (tAt(c) - ax) / s.rx;
            const py = (fAt(r) - ay) / s.ry;
            const t = len2 > 0 ? Math.max(0, Math.min(1, (px * ux + py * uy) / len2)) : 0;
            const dx = px - t * ux;
            const dy = py - t * uy;
            if (dx * dx + dy * dy <= 1) data[r * cols + c] = 255;
          }
        }
      }
    }
  }
  if (!data.some((v) => v > 0)) return null;
  return { start: b.t0, spc, cols, rows, frac_top: b.f1, frac_per_row: fpr, data };
}
