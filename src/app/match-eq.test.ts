// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { matchEq, responseDb, type Spectrum } from "./match-eq";

/** A log-spaced grid from 20 Hz to 20 kHz. */
const grid = (n = 240): number[] => Array.from({ length: n }, (_, i) => 20 * Math.pow(1000, i / (n - 1)));
const flat = (hz: number[], db = -40): Spectrum => ({ hz, db: hz.map(() => db) });
const shaped = (hz: number[], f: (hz: number) => number, base = -40): Spectrum => ({ hz, db: hz.map((h) => base + f(h)) });

describe("filter responses", () => {
  it("peak at its centre equals its gain, and is flat far away", () => {
    expect(responseDb("peak", 48_000, 1000, 1, 6, 1000)).toBeCloseTo(6, 1);
    expect(Math.abs(responseDb("peak", 48_000, 1000, 1, 6, 30))).toBeLessThan(0.2);
  });
  it("shelves reach their gain on their own side", () => {
    expect(responseDb("lowShelf", 48_000, 200, 1, 6, 20)).toBeCloseTo(6, 0);
    expect(Math.abs(responseDb("lowShelf", 48_000, 200, 1, 6, 10_000))).toBeLessThan(0.2);
    expect(responseDb("highShelf", 48_000, 6000, 1, -6, 18_000)).toBeCloseTo(-6, 0);
  });
});

describe("match EQ", () => {
  it("does nothing for two voices that already match", () => {
    const hz = grid();
    const fit = matchEq(flat(hz), flat(hz))!;
    expect(fit.errorBefore).toBeLessThan(0.01);
    const gains = Object.entries(fit.params).filter(([k]) => k.endsWith("_gain"));
    expect(gains.every(([, v]) => v === 0)).toBe(true);
  });

  it("ignores an overall level difference", () => {
    const hz = grid();
    const fit = matchEq(flat(hz, -50), flat(hz, -30))!;
    expect(fit.errorBefore).toBeLessThan(0.01);
  });

  it("finds a presence boost and a bass cut and lowers the error", () => {
    const hz = grid();
    const src = flat(hz);
    const ref = shaped(hz, (h) => 5 * Math.exp(-Math.pow(Math.log2(h / 3000) / 0.6, 2)) - 5 / (1 + Math.pow(h / 150, 2)));
    const fit = matchEq(src, ref)!;
    expect(fit.errorBefore).toBeGreaterThan(1.5);
    expect(fit.errorAfter).toBeLessThan(fit.errorBefore * 0.55);
    // The fitted curve lifts the presence region and cuts low down.
    const at = (f: number) => fit.fitted[fit.hz.reduce((b, h, k) => (Math.abs(Math.log(h / f)) < Math.abs(Math.log(fit.hz[b]! / f)) ? k : b), 0)]!;
    expect(at(3000)).toBeGreaterThan(2);
    expect(at(100)).toBeLessThan(-1);
  });

  it("never asks for more than 12 dB", () => {
    const hz = grid();
    const fit = matchEq(flat(hz), shaped(hz, (h) => (h > 2000 ? 40 : 0)))!;
    for (const [k, v] of Object.entries(fit.params)) if (k.endsWith("_gain")) expect(Math.abs(v)).toBeLessThanOrEqual(12);
  });

  it("leaves the filters off and refuses a spectrum with no points", () => {
    const hz = grid();
    const fit = matchEq(flat(hz), shaped(hz, (h) => (h > 5000 ? 6 : 0)))!;
    expect(fit.params.hp_on).toBe(0);
    expect(fit.params.lp_on).toBe(0);
    expect(matchEq({ hz: [], db: [] }, flat(hz))).toBeNull();
  });

  it("reads a reference measured on a different grid", () => {
    const src = flat(grid(240));
    const ref = shaped(grid(120), (h) => (h > 4000 ? 6 : 0));
    const fit = matchEq(src, ref)!;
    expect(fit.params.hs_gain ?? 0).toBeGreaterThan(2);
  });
});
