// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// The TypeScript loudness matching rules must agree with
// shared-fixtures/loudness-match.json, the same file the Rust tests read
// (src-tauri/tests/loudness_match_fixture.rs). Values JSON cannot hold (NaN and
// the infinities) are written as text.

import { describe, expect, it } from "vitest";
import fixture from "../../shared-fixtures/loudness-match.json";
import type { AnalysisReport } from "./analysis";
import type { LoudnessReport, LoudnessTarget } from "./loudness";
import {
  BASIS_LEVEL,
  changeFor,
  DEFAULT_TOLERANCE,
  GAIN_MAX,
  GAIN_MIN,
  levelRow,
  levelsOf,
  matchRow,
  truePeakRow,
  type MatchRow,
} from "./loudness-match";

const num = (v: unknown): number =>
  v === "NaN"
    ? NaN
    : v === "Infinity"
      ? Infinity
      : v === "-Infinity"
        ? -Infinity
        : (v as number);
const decode = <T extends object>(o: T): T =>
  Object.fromEntries(
    Object.entries(o).map(([k, v]) => [
      k,
      typeof v === "string" && /^-?(Infinity|NaN)$/.test(v) ? num(v) : v,
    ]),
  ) as T;
const same = (got: number, want: unknown) => {
  const w = num(want);
  if (Number.isNaN(w)) expect(got).toBeNaN();
  else if (!Number.isFinite(w)) expect(got).toBe(w);
  else expect(got).toBeCloseTo(w, 9);
};
const report = (r: unknown) => decode(r as object) as unknown as LoudnessReport;
const analysis = (r: { left: object; right: object }) =>
  ({
    left: decode(r.left),
    right: decode(r.right),
  }) as unknown as AnalysisReport;
const checkRow = (got: MatchRow, want: Record<string, unknown>) => {
  expect(got.key).toBe(want.key);
  expect(got.clipIds).toEqual(want.clipIds);
  same(got.measured, want.measured);
  same(got.truePeak, want.truePeak);
  same(got.change, want.change);
  same(got.peakAfter, want.peakAfter);
  expect(got.measurable).toBe(want.measurable);
  expect(got.peakWarning).toBe(want.peakWarning);
};

describe("shared loudness match fixture", () => {
  it("constants", () => {
    expect([GAIN_MIN, GAIN_MAX, DEFAULT_TOLERANCE]).toEqual([
      fixture.constants.gainMin,
      fixture.constants.gainMax,
      fixture.constants.defaultTolerance,
    ]);
    expect(BASIS_LEVEL).toEqual(fixture.constants.basisLevel);
  });
  it("changeFor", () => {
    for (const c of fixture.changeFor)
      same(changeFor(c.goal, c.measured, c.tolerance), c.want);
  });
  it("matchRow", () => {
    for (const c of fixture.matchRow)
      checkRow(
        matchRow(
          fixture.item,
          report(c.report),
          c.target as LoudnessTarget,
          c.tolerance,
        ),
        c.want,
      );
  });
  it("levelsOf", () => {
    for (const c of fixture.levelsOf) {
      const got = levelsOf(analysis(c.report));
      same(got.peakDb, c.want.peakDb);
      same(got.rmsDb, c.want.rmsDb);
    }
  });
  it("levelRow", () => {
    for (const c of fixture.levelRow)
      checkRow(
        levelRow(
          fixture.item,
          c.basis as "peak" | "rms",
          c.level,
          analysis(c.report),
          c.tolerance,
        ),
        c.want,
      );
  });
  it("truePeakRow", () => {
    for (const c of fixture.truePeakRow)
      checkRow(
        truePeakRow(fixture.item, c.level, report(c.report), c.tolerance),
        c.want,
      );
  });
});
