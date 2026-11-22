// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// The TypeScript Check Loudness and recording rules must agree with
// shared-fixtures/loudness-recording.json, the same file the Rust tests read
// (src-tauri/tests/loudness_recording_fixture.rs). Non-finite numbers are text
// in the file.

import { describe, expect, it } from "vitest";
import fixture from "../../shared-fixtures/loudness-recording.json";
import * as L from "./loudness";
import * as R from "./recording";

const num = (v: unknown): number => (v === "Infinity" ? Infinity : v === "-Infinity" ? -Infinity : v === "NaN" ? NaN : (v as number));
const report = (r: { integrated: unknown; integratedMono: unknown; truePeak: unknown }): L.LoudnessReport => ({
  integrated: num(r.integrated),
  integratedMono: num(r.integratedMono),
  truePeak: num(r.truePeak),
  range: 0,
  samplePeak: 0,
  seconds: 1,
  cancelled: false,
});
const target = (t: L.LoudnessTarget): L.LoudnessTarget => t;
const nearly = (got: unknown, want: unknown) => {
  const g = num(got);
  const w = num(want);
  if (Number.isNaN(w)) expect(Number.isNaN(g)).toBe(true);
  else if (Number.isFinite(w)) expect(g).toBeCloseTo(w, 9);
  else expect(g).toBe(w);
};

describe("shared loudness and recording fixture", () => {
  it("regions and targets", () => {
    for (const c of fixture.regionForZone) expect(L.regionForZone(c.zone)).toBe(c.want);
    for (const c of fixture.targets) expect(L.targetsFor(c.region, c.custom)).toEqual(c.want);
  });
  it("verdicts", () => {
    for (const c of fixture.evaluate) {
      const got = L.evaluate(report(c.report), target(c.target));
      nearly(got.measured, c.want.measured);
      nearly(got.offset, c.want.offset);
      nearly(got.gainToTarget, c.want.gainToTarget);
      expect(got.loudness).toBe(c.want.loudness);
      expect(got.peak).toBe(c.want.peak);
      expect(got.gainIsSafe).toBe(c.want.gainIsSafe);
      expect(got.advice).toBe(c.want.advice);
    }
  });
  it("export predictions", () => {
    for (const c of fixture.predict) {
      const got = L.predictExport(report(c.report), c.target ? target(c.target) : null, c.mono);
      nearly(got.gain, c.want.gain);
      nearly(got.loudness, c.want.loudness);
      nearly(got.truePeak, c.want.truePeak);
      expect(got.limitedByPeak).toBe(c.want.limitedByPeak);
      if (c.want.offset === null) expect(got.offset).toBeNull();
      else nearly(got.offset, c.want.offset);
      expect(got.grade).toBe(c.want.grade);
      expect(got.summary).toBe(c.want.summary);
    }
  });
  it("recording", () => {
    for (const c of fixture.sanitize) expect(R.sanitizeRecordOptions(c.raw ?? undefined)).toEqual(c.want);
    for (const c of fixture.recordPlan) expect(R.recordPlan(c.playhead, c.sel, c.o, c.rate)).toEqual(c.want);
    for (const c of fixture.punchStop) expect(R.punchStopAfter(c.plan, c.rate)).toEqual(c.want);
    for (const c of fixture.placeTake) expect(R.placeTake(c.plan, c.take, c.sessionRate, c.offsetMs)).toEqual(c.want);
  });
});
