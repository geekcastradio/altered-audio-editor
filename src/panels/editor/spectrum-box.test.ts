// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { adjustBox, bandBox, boxFromValues, boxValues, hitBox, rangeText, type BoxShape } from "./spectrum-box";

const box: BoxShape = { kind: "box", start: 1000, end: 3000, bottom: 0.2, top: 0.6 };
const px = { left: 100, right: 300, top: 50, bottom: 150 };

describe("hitBox", () => {
  it("finds corners, edges, the inside and nothing", () => {
    expect(hitBox(px, 100, 50)).toBe("nw");
    expect(hitBox(px, 300, 150)).toBe("se");
    expect(hitBox(px, 300, 50)).toBe("ne");
    expect(hitBox(px, 100, 150)).toBe("sw");
    expect(hitBox(px, 200, 50)).toBe("n");
    expect(hitBox(px, 200, 150)).toBe("s");
    expect(hitBox(px, 100, 100)).toBe("w");
    expect(hitBox(px, 300, 100)).toBe("e");
    expect(hitBox(px, 200, 100)).toBe("move");
    expect(hitBox(px, 10, 10)).toBeNull();
  });
  it("still lets a very thin box be resized", () => {
    const thin = { left: 100, right: 300, top: 100, bottom: 102 };
    expect(hitBox(thin, 200, 100)).toBe("n");
    expect(hitBox(thin, 200, 102)).toBe("s");
  });
});

describe("adjustBox", () => {
  it("moves the whole box and keeps its size", () => {
    const m = adjustBox(box, "move", 500, 0.1, 10000);
    expect([m.start, m.end]).toEqual([1500, 3500]);
    expect(m.top - m.bottom).toBeCloseTo(0.4);
  });
  it("stops a move at the edges of the file and the axis", () => {
    const m = adjustBox(box, "move", -5000, 0.9, 10000);
    expect([m.start, m.end]).toEqual([0, 2000]);
    expect(m.top).toBeCloseTo(1);
    expect(m.bottom).toBeCloseTo(0.6);
  });
  it("resizes only the grabbed sides", () => {
    const m = adjustBox(box, "e", 1000, 0.5, 10000);
    expect([m.start, m.end, m.bottom, m.top]).toEqual([1000, 4000, 0.2, 0.6]);
    const n = adjustBox(box, "nw", -200, 0.1, 10000);
    expect([n.start, n.end, n.bottom]).toEqual([800, 3000, 0.2]);
    expect(n.top).toBeCloseTo(0.7);
  });
  it("cannot turn a box inside out", () => {
    const m = adjustBox(box, "w", 99999, 0, 10000);
    expect(m.start).toBeLessThan(m.end);
    const n = adjustBox(box, "n", 0, -5, 10000);
    expect(n.top).toBeGreaterThan(n.bottom);
  });
});

describe("typed values", () => {
  const rate = 48000;
  const nyq = 24000;
  it("reads a box back as seconds and hertz, and typing them gives the box", () => {
    const b = boxFromValues({ start: 0.5, end: 2, low: 100, high: 1000 }, rate, 480000, nyq)!;
    const v = boxValues(b, rate, nyq);
    expect(v.start).toBeCloseTo(0.5, 4);
    expect(v.end).toBeCloseTo(2, 4);
    expect(v.low).toBeCloseTo(100, 0);
    expect(v.high).toBeCloseTo(1000, 0);
  });
  it("accepts the ends in either order and holds them inside the file", () => {
    const b = boxFromValues({ start: 99, end: -3, low: 5000, high: 50 }, rate, 48000, nyq)!;
    expect(b.start).toBe(0);
    expect(b.end).toBe(48000);
    expect(b.bottom).toBeLessThan(b.top);
  });
  it("refuses numbers that are not numbers", () => {
    expect(boxFromValues({ start: NaN, end: 1, low: 1, high: 2 }, rate, 48000, nyq)).toBeNull();
  });
  it("describes the range", () => {
    expect(rangeText({ start: 0, end: 1, low: 20, high: 180 })).toBe("20 Hz to 180 Hz, range 160 Hz");
    expect(rangeText({ start: 0, end: 1, low: 500, high: 2500 })).toBe("500 Hz to 2.5 kHz, range 2 kHz");
  });
});

describe("bandBox", () => {
  it("runs a band over the whole file, or over the time selection", () => {
    const all = bandBox(0.7, 0.2, null, 9000);
    expect([all.start, all.end, all.bottom, all.top]).toEqual([0, 9000, 0.2, 0.7]);
    const part = bandBox(0.1, 0.3, { start: 4000, end: 1000 }, 9000);
    expect([part.start, part.end]).toEqual([1000, 4000]);
  });
  it("makes a click on the scale a thin but valid band", () => {
    const b = bandBox(0.5, 0.5, null, 9000);
    expect(b.top).toBeGreaterThan(b.bottom);
  });
});
