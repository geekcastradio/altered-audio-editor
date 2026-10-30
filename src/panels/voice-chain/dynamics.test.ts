// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { curvePoints, ratioFromSlope, transfer } from "./dynamics";

const c = { threshold: -20, ratio: 4, knee: 0, makeup: 0 };

describe("Dynamics transfer curve", () => {
  it("passes quiet sound through unchanged", () => {
    expect(transfer(-40, c)).toBe(-40);
  });
  it("turns the part over the threshold down by the ratio", () => {
    expect(transfer(0, c)).toBeCloseTo(-15);
  });
  it("adds make-up gain everywhere", () => {
    expect(transfer(-40, { ...c, makeup: 6 })).toBe(-34);
  });
  it("is continuous through a soft knee", () => {
    const k = { ...c, knee: 10 };
    expect(transfer(-25, k)).toBeCloseTo(-25);
    expect(transfer(-15, k)).toBeCloseTo(-20 + 5 / 4);
    expect(Math.abs(transfer(-20.01, k) - transfer(-19.99, k))).toBeLessThan(0.02);
  });
  it("never rises faster than the input", () => {
    const pts = curvePoints({ ...c, knee: 8 });
    for (let i = 1; i < pts.length; i++) expect(pts[i]![1] - pts[i - 1]![1]).toBeLessThanOrEqual(pts[i]![0] - pts[i - 1]![0] + 1e-9);
  });
  it("turns a slope into a ratio within the control's range", () => {
    expect(ratioFromSlope(0.25)).toBe(4);
    expect(ratioFromSlope(0)).toBe(20);
    expect(ratioFromSlope(2)).toBe(1);
  });
});
