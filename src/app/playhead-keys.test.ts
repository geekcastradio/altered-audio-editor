// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { adjustSelection, headTarget, jumpTarget, type HeadContext } from "./playhead-keys";

const ctx = (o: Partial<HeadContext> = {}): HeadContext => ({ pos: 10_000, end: 100_000, spp: 100, width: 500, sel: null, ...o });

describe("playhead keys", () => {
  it("steps one pixel at the current zoom", () => {
    expect(headTarget("step-left", ctx())).toBe(9_900);
    expect(headTarget("step-right", ctx())).toBe(10_100);
    expect(headTarget("step-right", ctx({ spp: 0.2 }))).toBe(10_001);
  });

  it("pages by the width of the lane", () => {
    expect(headTarget("page-right", ctx())).toBe(60_000);
    expect(headTarget("page-left", ctx({ pos: 70_000 }))).toBe(20_000);
  });

  it("stays between zero and the end", () => {
    expect(headTarget("step-left", ctx({ pos: 30 }))).toBe(0);
    expect(headTarget("page-right", ctx({ pos: 90_000 }))).toBe(100_000);
    expect(headTarget("page-left", ctx({ pos: 100 }))).toBe(0);
  });

  it("goes to the In and Out points, and nowhere without a selection", () => {
    const sel = { start: 5_000, end: 8_000 };
    expect(headTarget("to-in", ctx({ sel }))).toBe(5_000);
    expect(headTarget("to-out", ctx({ sel }))).toBe(8_000);
    expect(headTarget("to-in", ctx())).toBeNull();
    expect(headTarget("to-out", ctx({ sel: { start: 4, end: 4 } }))).toBeNull();
  });

  it("reads a typed time and keeps it inside the content", () => {
    expect(jumpTarget("1:05.250", 48_000, "time", 30, 10_000_000)).toBe(3_132_000);
    expect(jumpTarget("65.25", 48_000, "time", 30, 10_000_000)).toBe(3_132_000);
    expect(jumpTarget("99:00", 48_000, "time", 30, 1_000_000)).toBe(1_000_000);
    expect(jumpTarget("soon", 48_000, "time", 30, 1_000_000)).toBeNull();
  });
});

describe("adjustSelection", () => {
  const sel = { start: 1_000, end: 5_000 };
  it("moves each border by the step", () => {
    expect(adjustSelection(sel, "start-left", 100, 10_000)).toEqual({ start: 900, end: 5_000 });
    expect(adjustSelection(sel, "start-right", 100, 10_000)).toEqual({ start: 1_100, end: 5_000 });
    expect(adjustSelection(sel, "end-left", 100, 10_000)).toEqual({ start: 1_000, end: 4_900 });
    expect(adjustSelection(sel, "end-right", 100, 10_000)).toEqual({ start: 1_000, end: 5_100 });
  });
  it("never crosses the borders or leaves the content", () => {
    expect(adjustSelection(sel, "start-right", 9_999, 10_000)).toEqual({ start: 4_999, end: 5_000 });
    expect(adjustSelection(sel, "end-left", 9_999, 10_000)).toEqual({ start: 1_000, end: 1_001 });
    expect(adjustSelection(sel, "start-left", 9_999, 10_000)).toEqual({ start: 0, end: 5_000 });
    expect(adjustSelection(sel, "end-right", 9_999, 10_000)).toEqual({ start: 1_000, end: 10_000 });
  });
  it("does nothing without a selection", () => {
    expect(adjustSelection(null, "end-right", 100, 10_000)).toBeNull();
    expect(adjustSelection({ start: 5, end: 5 }, "end-right", 100, 10_000)).toBeNull();
  });
});
