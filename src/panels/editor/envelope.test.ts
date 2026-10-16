// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import type { EnvelopePointDoc } from "../../stores/session";
import { addPoint, dbToY, formatValue, levelAt, movePoint, sliceVolume, valueToY, yToDb, yToValue } from "./envelope";

const pt = (position: number, value: number): EnvelopePointDoc => ({ position, value, curve: "linear" });

describe("volume envelope", () => {
  const pts = [pt(100, 0), pt(300, -20)];

  it("holds before and after, and interpolates in dB", () => {
    expect(levelAt(pts, 0)).toBe(0);
    expect(levelAt(pts, 200)).toBe(-10);
    expect(levelAt(pts, 1_000)).toBe(-20);
    expect(levelAt([], 5)).toBe(0);
  });

  it("maps dB to the lane and back", () => {
    expect(dbToY(12, 10, 72)).toBe(10);
    expect(dbToY(-60, 10, 72)).toBe(82);
    expect(yToDb(dbToY(-6, 0, 100), 0, 100)).toBe(-6);
  });

  it("slices a piece and keeps its levels", () => {
    expect(sliceVolume(pts, 200, 400)).toEqual([pt(0, -10), pt(100, -20), pt(200, -20)]);
    expect(sliceVolume([], 0, 10)).toEqual([]);
  });

  it("adds and moves points", () => {
    const a = addPoint(pts, 200, -3);
    expect(a.index).toBe(1);
    expect(a.points.map((p) => p.position)).toEqual([100, 200, 300]);
    const m = movePoint(a.points, 1, 1_000, 99, 2_000);
    expect(m[1]).toEqual(pt(299, 12));
  });
});

describe("pan envelope", () => {
  it("draws full left at the top, center in the middle, full right at the bottom", () => {
    expect(valueToY("pan", -100, 10, 100)).toBe(10);
    expect(valueToY("pan", 0, 10, 100)).toBe(60);
    expect(valueToY("pan", 100, 10, 100)).toBe(110);
    expect(valueToY("pan", 400, 10, 100)).toBe(110);
  });

  it("reads y back to whole pan steps", () => {
    expect(yToValue("pan", 60, 10, 100)).toBe(0);
    expect(yToValue("pan", 35.4, 10, 100)).toBe(-49);
    expect(yToValue("pan", 999, 10, 100)).toBe(100);
  });

  it("volume mapping matches the dB helpers", () => {
    expect(valueToY("volume", -6, 0, 72)).toBeCloseTo(dbToY(-6, 0, 72));
    expect(yToValue("volume", 30, 0, 72)).toBeCloseTo(yToDb(30, 0, 72));
  });

  it("clamps moved points to the pan range", () => {
    const next = movePoint([pt(0, 0)], 0, 0, -300, 1000, "pan");
    expect(next[0]!.value).toBe(-100);
  });

  it("labels values", () => {
    expect(formatValue("pan", 0)).toBe("C");
    expect(formatValue("pan", -40)).toBe("L 40");
    expect(formatValue("pan", 25.4)).toBe("R 25");
    expect(formatValue("volume", 3)).toBe("+3.0 dB");
    expect(formatValue("volume", -6)).toBe("-6.0 dB");
  });
});
