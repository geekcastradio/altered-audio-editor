// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { fracOf, freqAt, freqTicks, FULL_VIEW, MIN_HZ, MIN_SPAN, rasterize, shapesBounds, SPECTRUM_LUT, viewFrac, viewFreq, viewRange, zoomFreq } from "./spectrum";

describe("spectrum axis", () => {
  it("is log spaced and round-trips", () => {
    expect(freqAt(0, 24_000)).toBeCloseTo(MIN_HZ);
    expect(freqAt(1, 24_000)).toBeCloseTo(24_000);
    for (const hz of [50, 440, 1000, 8000]) expect(freqAt(fracOf(hz, 24_000), 24_000)).toBeCloseTo(hz, 3);
    expect(fracOf(10, 24_000)).toBe(0);
  });

  it("colors run from dark to light", () => {
    expect(SPECTRUM_LUT.length).toBe(768);
    const lum = (i: number) => SPECTRUM_LUT[i * 3]! + SPECTRUM_LUT[i * 3 + 1]! + SPECTRUM_LUT[i * 3 + 2]!;
    expect(lum(0)).toBeLessThan(lum(128));
    expect(lum(128)).toBeLessThan(lum(255));
  });
});

describe("spectral selection masks", () => {
  const at = (m: NonNullable<ReturnType<typeof rasterize>>, t: number, f: number) => {
    const c = Math.floor((t - m.start) / m.spc);
    const r = Math.floor((m.frac_top - f) / m.frac_per_row);
    return c < 0 || r < 0 || c >= m.cols || r >= m.rows ? 0 : m.data[r * m.cols + c]!;
  };

  it("fills a box", () => {
    const m = rasterize([{ kind: "box", start: 1000, end: 2000, bottom: 0.4, top: 0.6 }])!;
    expect(at(m, 1500, 0.5)).toBe(255);
    expect(m.start).toBe(1000);
    expect(m.frac_top).toBeCloseTo(0.6);
  });

  it("fills inside a lasso only", () => {
    const m = rasterize([{ kind: "lasso", points: [[0, 0.2], [1000, 0.2], [1000, 0.8], [500, 0.5], [0, 0.8]] }])!;
    expect(at(m, 100, 0.3)).toBe(255);
    expect(at(m, 500, 0.75)).toBe(0); // the notch
  });

  it("sweeps a brush along its stroke", () => {
    const m = rasterize([{ kind: "brush", points: [[1000, 0.5], [3000, 0.5]], rx: 100, ry: 0.05 }])!;
    expect(at(m, 2000, 0.5)).toBe(255);
    expect(at(m, 2000, 0.54)).toBe(255);
    expect(at(m, 2000, 0.4)).toBe(0);
  });

  it("is null for nothing", () => {
    expect(rasterize([])).toBeNull();
    expect(shapesBounds([{ kind: "lasso", points: [] }])).toBeNull();
  });
});

describe("the spectrum view's frequency axis", () => {
  const ny = 24_000;
  it("matches the edits' log axis when fully zoomed out", () => {
    for (const f of [0, 0.25, 0.5, 1]) expect(viewFreq(f, FULL_VIEW, ny)).toBeCloseTo(freqAt(f, ny), 3);
  });
  it("goes from frequency to place and back, linear or log", () => {
    for (const log of [0, 0.5, 1]) {
      const v = { lo: 100, hi: 8_000, log };
      expect(viewFrac(viewFreq(0.3, v, ny), v, ny)).toBeCloseTo(0.3, 4);
    }
  });
  it("zooms around the pointer, within 0 Hz and the Nyquist frequency", () => {
    const v = zoomFreq(FULL_VIEW, 1_000, 0.5, ny);
    const [lo, hi] = viewRange(v, ny);
    expect(lo).toBeGreaterThan(30);
    expect(hi).toBeLessThan(ny);
    // 1 kHz stays where it was.
    expect(viewFrac(1_000, v, ny)).toBeCloseTo(viewFrac(1_000, FULL_VIEW, ny), 2);
    // Never narrower than the minimum, never past the top.
    let w = v;
    for (let i = 0; i < 60; i++) w = zoomFreq(w, 1_000, 0.5, ny);
    const [a, b] = viewRange(w, ny);
    expect(b - a).toBeGreaterThanOrEqual(MIN_SPAN - 1e-6);
    let out = v;
    for (let i = 0; i < 60; i++) out = zoomFreq(out, 1_000, 2, ny);
    expect(out.hi).toBeNull();
  });
  it("labels the axis with spaced marks", () => {
    const t = freqTicks(FULL_VIEW, ny, 300);
    expect(t).toContain(1_000);
    expect(freqTicks({ lo: 400, hi: 600, log: 0 }, ny, 300).every((f) => f >= 400 && f <= 600)).toBe(true);
  });
});

describe("lasso fill", () => {
  // The old way: test every cell against the outline. Kept here as the reference.
  function slow(shapes: Parameters<typeof rasterize>[0]): number[] | null {
    const r = rasterize(shapes);
    if (!r) return null;
    const lasso = shapes[0]!;
    if (lasso.kind !== "lasso") return null;
    const pts = lasso.points;
    const b = shapesBounds(shapes)!;
    const out = new Array<number>(r.cols * r.rows).fill(0);
    for (let row = 0; row < r.rows; row++) {
      for (let c = 0; c < r.cols; c++) {
        const x = b.t0 + (c + 0.5) * r.spc;
        const y = b.f1 - (row + 0.5) * r.frac_per_row;
        let inside = false;
        for (let i = 0, j = pts.length - 1; i < pts.length; j = i++) {
          const [xi, yi] = pts[i]!;
          const [xj, yj] = pts[j]!;
          if (yi > y !== yj > y && x < ((xj - xi) * (y - yi)) / (yj - yi) + xi) inside = !inside;
        }
        if (inside) out[row * r.cols + c] = 255;
      }
    }
    return out;
  }

  it("matches testing every cell, on a wobbly freehand outline", () => {
    let seed = 7;
    const rnd = () => ((seed = (seed * 1664525 + 1013904223) % 4294967296) / 4294967296);
    const points: [number, number][] = [];
    for (let i = 0; i < 400; i++) {
      const a = (i / 400) * Math.PI * 2;
      const rad = 0.3 + 0.12 * Math.sin(a * 5) + 0.02 * rnd();
      points.push([100_000 + 60_000 * Math.cos(a) * (rad / 0.3), 0.5 + rad * Math.sin(a)]);
    }
    const shapes = [{ kind: "lasso", points }] as Parameters<typeof rasterize>[0];
    const fast = rasterize(shapes)!;
    const ref = slow(shapes)!;
    let diff = 0;
    for (let i = 0; i < ref.length; i++) if (fast.data[i] !== ref[i]) diff++;
    // Cells sitting exactly on an edge can differ; nothing else may.
    expect(diff).toBeLessThanOrEqual(ref.length * 0.0005);
    expect(fast.data.some((v) => v > 0)).toBe(true);
  });

  it("is quick for a large outline", () => {
    const points: [number, number][] = [];
    for (let i = 0; i < 2000; i++) {
      const a = (i / 2000) * Math.PI * 2;
      points.push([200_000 + 150_000 * Math.cos(a), 0.5 + 0.4 * Math.sin(a)]);
    }
    const t0 = performance.now();
    rasterize([{ kind: "lasso", points }] as Parameters<typeof rasterize>[0]);
    expect(performance.now() - t0).toBeLessThan(250);
  });
});
