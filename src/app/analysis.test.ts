// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { binDb, dbY, findings, fmtAt, fmtDb, fmtHz, histogramBars, logX, type AnalysisReport } from "./analysis";

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

  it("reports clicks, flattened peaks, hum and DC offset in plain words", () => {
    const side = { dcOffset: 0, peakDb: -3 };
    const clean = { left: side, right: side, problems: { clicks: 0, clickTimes: [], clipRegions: 0, clipSamples: 0, clipTimes: [], hum: null } } as unknown as AnalysisReport;
    expect(findings(clean).map((f) => f.found)).toEqual([false, false, false, false]);
    const bad = {
      left: { ...side, dcOffset: 0.02 },
      right: side,
      problems: {
        clicks: 1,
        clickTimes: [62.5],
        clipRegions: 120,
        clipSamples: 400,
        clipTimes: [1.2, 3],
        hum: { baseHz: 60, strengthDb: 24.4, tones: [{ hz: 60, excessDb: 24.4 }, { hz: 120, excessDb: 3 }, { hz: 180, excessDb: 12 }] },
      },
    } as unknown as AnalysisReport;
    const f = findings(bad);
    expect(f.every((x) => x.found)).toBe(true);
    expect(f[0]).toMatchObject({ text: "1 click found.", at: ["1:02.500"] });
    expect(f[1]?.text).toBe("120 flattened peaks (400 samples at full scale), the first 2 listed.");
    expect(f[2]?.text).toBe("Possible 60 Hz mains hum: tones at 60 Hz, 180 Hz, the strongest 24 dB above the sound around it.");
    expect(f[3]?.text).toContain("0.0200");
    expect(fmtAt(0.0004)).toBe("0:00.000");
    expect(fmtAt(3725.0126)).toBe("62:05.013");
  });
});
