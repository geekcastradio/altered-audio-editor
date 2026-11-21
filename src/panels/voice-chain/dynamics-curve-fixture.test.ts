// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// The TypeScript Dynamics curve must agree with shared-fixtures/dynamics-curve.json,
// the same file the Rust tests read (src-tauri/tests/dynamics_curve_fixture.rs).

import { describe, expect, it } from "vitest";
import fixture from "../../../shared-fixtures/dynamics-curve.json";
import { curvePoints, ratioFromSlope, transfer } from "./dynamics";

const points = (got: [number, number][], want: number[][]) => {
  expect(got.length).toBe(want.length);
  got.forEach((p, i) => {
    expect(p[0]).toBeCloseTo(want[i]![0]!, 9);
    expect(p[1]).toBeCloseTo(want[i]![1]!, 9);
  });
};

describe("shared dynamics curve fixture", () => {
  it.each(fixture.transfer)("transfer: threshold $threshold ratio $ratio knee $knee makeup $makeup at $inDb", (c) => {
    expect(transfer(c.inDb, c)).toBeCloseTo(c.want, 9);
  });
  it.each(fixture.curvePoints)("curvePoints: $lo to $hi in $steps steps", (c) => {
    points(curvePoints(c, c.lo, c.hi, c.steps), c.want);
  });
  it.each(fixture.curvePointsDefault)("curvePoints with the default range", (c) => {
    points(curvePoints(c), c.want);
  });
  it.each(fixture.ratioFromSlope)("ratioFromSlope: $slope", (c) => {
    const got = "min" in c ? ratioFromSlope(c.slope, c.min, c.max) : ratioFromSlope(c.slope);
    expect(got).toBeCloseTo(c.want, 9);
  });
});
