// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// The TypeScript analysis formats and scales must agree with
// shared-fixtures/analysis.json, the same file the Rust tests read
// (src-tauri/tests/analysis_fixture.rs).

import { describe, expect, it } from "vitest";
import fixture from "../../shared-fixtures/analysis.json";
import {
  binDb,
  dbY,
  fmtAt,
  fmtDb,
  fmtHz,
  fmtSample,
  FREQ_TICKS,
  histogramBars,
  humCutFix,
  logX,
} from "./analysis";

/** A value the fixture wrote as a number, or as the text of one that JSON cannot hold. */
const number = (v: unknown): number | null | undefined =>
  v === "NaN"
    ? NaN
    : v === "Infinity"
      ? Infinity
      : v === "-Infinity"
        ? -Infinity
        : v === "undefined"
          ? undefined
          : (v as number | null);

const close = (got: number, want: number) => expect(got).toBeCloseTo(want, 9);

describe("shared analysis fixture", () => {
  it("fmtDb", () => {
    for (const c of fixture.fmtDb)
      expect(fmtDb(number(c.v), c.floor), JSON.stringify(c)).toBe(
        c.want,
      );
  });
  it("text formats", () => {
    for (const c of fixture.fmtSample)
      expect(fmtSample(c.v), JSON.stringify(c)).toBe(c.want);
    for (const c of fixture.fmtHz)
      expect(fmtHz(c.hz), JSON.stringify(c)).toBe(c.want);
    for (const c of fixture.fmtAt)
      expect(fmtAt(c.seconds), JSON.stringify(c)).toBe(c.want);
  });
  it("scales", () => {
    for (const c of fixture.logX) close(logX(c.hz, c.low, c.high), c.want);
    for (const c of fixture.dbY) close(dbY(c.db, c.top, c.bottom), c.want);
    for (const c of fixture.binDb) close(binDb(c.i), c.want);
    expect(FREQ_TICKS).toEqual(fixture.freqTicks);
  });
  it("histogramBars", () => {
    for (const c of fixture.histogramBars) {
      const got = histogramBars(c.bins);
      expect(got.length).toBe(c.want.length);
      got.forEach((g, i) => close(g, c.want[i]!));
    }
  });
  it("humCutFix", () => {
    for (const c of fixture.humCutFix)
      expect(humCutFix(c.hum), JSON.stringify(c)).toEqual(c.want);
  });
});
