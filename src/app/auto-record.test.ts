// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { Take, thin, toleranceFor } from "./auto-record";
import type { EnvelopePointDoc } from "../stores/session";

const P = (position: number, value: number): EnvelopePointDoc => ({ position, value, curve: "linear" });
const RATE = 1000; // glide is 20 samples

describe("thin", () => {
  it("keeps the ends and the corners, drops the points on the line", () => {
    const line = [P(0, 0), P(10, 1), P(20, 2), P(30, 3), P(40, 0)];
    expect(thin(line, 0.1).map((p) => p.position)).toEqual([0, 30, 40]);
  });
  it("keeps everything at zero tolerance", () => {
    const line = [P(0, 0), P(10, 1), P(20, 2)];
    expect(thin(line, 0)).toHaveLength(3);
  });
  it("derives the tolerance from the lane's range", () => {
    expect(toleranceFor({ top: 12, bottom: -60 }, 0.003)).toBeCloseTo(0.216, 3);
    expect(toleranceFor({ top: 12, bottom: -60 }, 0)).toBe(0);
  });
});

describe("Touch", () => {
  it("writes while held, then the lane comes back", () => {
    const base = [P(0, -6), P(1000, -6)];
    const t = new Take(RATE);
    t.arm("v", "touch", base, 0, 0, 0, -6);
    expect(t.writing("v")).toBe(false);
    t.set("v", 100, -20); // not held: ignored
    t.grab("v", 200, -6);
    t.set("v", 300, -12);
    t.set("v", 400, -18);
    t.release("v", 500);
    const out = t.finish(1000).get("v")!;
    // Held span rewritten, the lane's -6 before it and after the glide.
    expect(out.find((p) => p.position === 199)!.value).toBe(-6);
    expect(out.find((p) => p.position === 400)!.value).toBe(-18);
    expect(out.find((p) => p.position === 500)!.value).toBe(-18);
    expect(out.find((p) => p.position === 520)!.value).toBe(-6);
    expect(out[out.length - 1]).toEqual(P(1000, -6));
    expect(out.every((p, i) => i === 0 || p.position > out[i - 1]!.position)).toBe(true);
  });

  it("leaves a lane alone that was never held", () => {
    const t = new Take(RATE);
    t.arm("v", "touch", [P(0, 0)], 0, 0, 0, 0);
    t.set("v", 50, -10);
    expect(t.finish(100).size).toBe(0);
  });

  it("writes two separate touches", () => {
    const t = new Take(RATE);
    t.arm("v", "touch", [], 0, 0, 0, 0);
    t.grab("v", 100, -3);
    t.set("v", 150, -4);
    t.release("v", 200);
    t.grab("v", 600, -9);
    t.set("v", 650, -10);
    t.release("v", 700);
    const out = t.finish(1000).get("v")!;
    expect(out.map((p) => p.position)).toEqual([100, 150, 200, 600, 650, 700]);
  });
});

describe("Latch", () => {
  it("starts at the first touch and writes to the stop, even after release", () => {
    const base = [P(0, 0), P(2000, 0)];
    const t = new Take(RATE);
    t.arm("p", "latch", base, 0, 0, 0, 0);
    t.grab("p", 300, 10);
    t.set("p", 400, 20);
    t.release("p", 450); // latch keeps writing
    expect(t.writing("p")).toBe(true);
    const out = t.finish(1000).get("p")!;
    expect(out.find((p) => p.position === 1000)!.value).toBe(20);
    // The lane's own points after the stop come back after the glide.
    expect(out.find((p) => p.position === 1020)!.value).toBe(0);
    expect(out[out.length - 1]).toEqual(P(2000, 0));
  });
});

describe("Write", () => {
  it("rewrites a lane that exists from the start of the take", () => {
    const base = [P(0, -3), P(500, -9), P(1000, -3)];
    const t = new Take(RATE);
    t.arm("v", "write", base, 0, 0, 0, -3);
    expect(t.writing("v")).toBe(true);
    t.set("v", 500, -3);
    const out = t.finish(1000).get("v")!;
    expect(out.map((p) => p.value)).toEqual([-3, -3, -3]);
  });

  it("starts at the first move when there is no lane yet", () => {
    const t = new Take(RATE);
    t.arm("v", "write", [], 0, 0, 0, 0);
    expect(t.writing("v")).toBe(false);
    t.set("v", 250, -5);
    const out = t.finish(1000).get("v")!;
    expect(out[0]).toEqual(P(250, -5));
    expect(out[out.length - 1]).toEqual(P(1000, -5));
  });

  it("thins what it wrote", () => {
    const t = new Take(RATE);
    t.arm("v", "write", [], 0, 0.5, 0, 0);
    for (let i = 0; i <= 100; i++) t.set("v", i * 10, -i * 0.1);
    const out = t.finish(1000).get("v")!;
    expect(out.length).toBeLessThan(5);
  });
});
