// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { meterFraction } from "../../stores/transport";
import { METER_RANGES, meterPercent, sanitizeSettings, DEFAULT_SETTINGS } from "../../stores/settings";
import { meterPos, meterTicks, rangeLabel } from "./scale";

describe("meter scale", () => {
  it("starts each scale at its floor and ends it at 0 dB", () => {
    for (const r of METER_RANGES) {
      const t = meterTicks(r);
      expect(t[0]).toBe(-r);
      expect(t[t.length - 1]).toBe(0);
      expect(meterPos(-r, r)).toBe(0);
      expect(meterPos(0, r)).toBe(100);
    }
  });
  it("keeps the old -60 dB look by default", () => {
    expect(meterPos(-30, 60)).toBe(50);
    expect(meterTicks(60)).toEqual([-60, -48, -36, -24, -18, -12, -9, -6, -3, 0]);
    expect(meterTicks(1234)).toEqual(meterTicks(60));
  });
  it("shows a quiet noise floor on the deeper scales", () => {
    // -80 dB is off the bottom of a -60 dB meter but well up a -96 dB one.
    expect(meterFraction(10 ** (-80 / 20), -60)).toBe(0);
    expect(meterFraction(10 ** (-80 / 20), -96)).toBeCloseTo(16 / 96, 6);
    expect(meterPercent(-12, -96)).toBeCloseTo((84 / 96) * 100, 6);
  });
  it("only accepts the offered depths", () => {
    expect(sanitizeSettings({ meterRangeDb: 96 }).meterRangeDb).toBe(96);
    expect(sanitizeSettings({ meterRangeDb: 50 }).meterRangeDb).toBe(DEFAULT_SETTINGS.meterRangeDb);
    expect(sanitizeSettings({ meterRangeDb: "96" }).meterRangeDb).toBe(60);
    expect(rangeLabel(96)).toBe("-96 dB");
  });
});
