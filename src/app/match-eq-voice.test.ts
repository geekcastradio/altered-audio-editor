// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { VOICE_TARGET_OPTS, voiceTarget, voiceTargetEq, type Spectrum } from "./match-eq";

const grid = (n = 240): number[] => Array.from({ length: n }, (_, i) => 20 * Math.pow(1000, i / (n - 1)));
const onGrid = (hz: number[], f: (h: number) => number): Spectrum => ({ hz, db: hz.map((h) => -40 + f(h)) });

/** A voice that sits exactly on the target, plus `extra` dB. */
function onTarget(hz: number[], extra: (h: number) => number): Spectrum {
  const t = voiceTarget();
  return onGrid(hz, (h) => {
    let j = 0;
    while (j < t.hz.length - 2 && t.hz[j + 1]! < h) j++;
    const k = Math.log(h / t.hz[j]!) / Math.log(t.hz[j + 1]! / t.hz[j]!);
    return t.db[j]! + (t.db[j + 1]! - t.db[j]!) * Math.min(1, Math.max(0, k)) + extra(h);
  });
}

describe("Voice AutoEQ", () => {
  it("the target covers 50 Hz to 20 kHz and rolls off the top", () => {
    const t = voiceTarget();
    expect(t.hz[0]).toBeCloseTo(50, 3);
    expect(t.hz[t.hz.length - 1]).toBeCloseTo(20_000, 0);
    expect(t.db[t.db.length - 1]!).toBeLessThan(t.db[Math.floor(t.db.length / 2)]!);
  });

  it("a voice that already sits on the target needs no EQ", () => {
    const fit = voiceTargetEq(onTarget(grid(), () => 0))!;
    for (const [k, v] of Object.entries(fit.params)) if (k.endsWith("_gain")) expect(Math.abs(v)).toBeLessThan(1.5);
  });

  it("never asks for more than the cap, however far off the voice is", () => {
    const hz = grid();
    const fit = voiceTargetEq(onGrid(hz, (h) => (h > 3000 ? 30 : 0)))!;
    for (const [k, v] of Object.entries(fit.params)) if (k.endsWith("_gain")) expect(Math.abs(v)).toBeLessThanOrEqual(VOICE_TARGET_OPTS.capDb);
  });

  it("is not changed by how loud the recording is", () => {
    const hz = grid();
    const shape = (h: number) => 4 * Math.exp(-Math.pow(Math.log2(h / 300) / 0.8, 2));
    const a = voiceTargetEq({ hz, db: hz.map((h) => -50 + shape(h)) })!;
    const b = voiceTargetEq({ hz, db: hz.map((h) => -20 + shape(h)) })!;
    expect(a.params).toEqual(b.params);
  });

  it("cuts a boomy voice in the low mids", () => {
    const fit = voiceTargetEq(onTarget(grid(), (h) => 7 * Math.exp(-Math.pow(Math.log2(h / 250) / 0.5, 2))))!;
    const at = (f: number) => fit.fitted[fit.hz.reduce((b, h, k) => (Math.abs(Math.log(h / f)) < Math.abs(Math.log(fit.hz[b]! / f)) ? k : b), 0)]!;
    expect(at(250)).toBeLessThan(-1);
  });
});
