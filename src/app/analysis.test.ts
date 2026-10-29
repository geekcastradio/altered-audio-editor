// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { binDb, dbY, fmtDb, fmtHz, histogramBars, logX } from "./analysis";

describe("analysis helpers", () => {
  it("formats levels and silence", () => {
    expect(fmtDb(-6.02)).toBe("-6.0 dB");
    expect(fmtDb(-200)).toBe("-inf dB");
    expect(fmtDb(Number.NaN)).toBe("-inf dB");
  });

  it("places frequencies on a log axis", () => {
    expect(logX(20)).toBe(0);
    expect(logX(20000)).toBe(1);
    expect(logX(1000)).toBeGreaterThan(0.5);
    expect(logX(1000)).toBeLessThan(0.7);
    expect(logX(5)).toBe(0);
  });

  it("maps levels to a vertical span", () => {
    expect(dbY(0, 0, -100)).toBe(0);
    expect(dbY(-50, 0, -100)).toBe(0.5);
    expect(dbY(-300, 0, -100)).toBe(1);
  });

  it("scales histogram bars to the biggest bin", () => {
    const bars = histogramBars([0, 9, 99]);
    expect(bars[0]).toBe(0);
    expect(bars[2]).toBe(1);
    expect(bars[1]).toBeGreaterThan(0.4);
    expect(histogramBars([0, 0])).toEqual([0, 0]);
  });

  it("names axis ticks", () => {
    expect(fmtHz(500)).toBe("500");
    expect(fmtHz(2000)).toBe("2k");
    expect(binDb(12)).toBe(-12);
  });
});
