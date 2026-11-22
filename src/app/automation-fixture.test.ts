// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Checks the TypeScript automation lane rules against the same fixture the Rust test reads
// (shared-fixtures/automation.json).
import { describe, expect, it } from "vitest";
import fx from "../../shared-fixtures/automation.json";
import * as A from "../panels/editor/automation";
import { thin } from "./auto-record";
import type { EnvelopePointDoc } from "../stores/session";

const T = (t: unknown) => t as A.AutoTarget;
const P = (p: unknown) => p as EnvelopePointDoc[];
const owner = { volume_db: 0, pan: 0 } as never;

describe("automation fixture", () => {
  it("lanes, ranges and keys", () => {
    for (const c of fx.lane)
      expect(A.laneHeight(c.h === null ? {} : { height: c.h })).toBe(c.out);
    for (const c of fx.keys) expect(A.targetKey(T(c.t))).toBe(c.out);
    for (const c of fx.ranges)
      expect(
        A.autoRange(owner, [], { type: "space", param: c.p as never }),
      ).toEqual(c.out);
  });
  it("pixels, values, snapping and text", () => {
    for (const c of fx.ymap)
      expect(
        c.dir === "toY"
          ? A.autoY(c.v!, c.r, c.top, c.h)
          : A.autoValue(c.y!, c.r, c.top, c.h),
      ).toBe(c.out);
    for (const c of fx.snap) expect(A.autoSnap(T(c.t), c.v)).toBe(c.out);
    for (const c of fx.format)
      expect(A.autoFormat(T(c.t), c.v, c.r)).toBe(c.out);
  });
  it("values and points", () => {
    for (const c of fx.at)
      expect(A.autoAt(P(c.pts), c.pos, c.rest)).toBe(c.out);
    for (const c of fx.add)
      expect(A.autoAdd(P(c.pts), c.pos, c.v)).toEqual(c.out);
    for (const c of fx.move)
      expect(A.autoMove(P(c.pts), c.i, c.pos, c.v)).toEqual(c.out);
    for (const c of fx.many)
      expect(
        A.autoMoveMany(P(c.pts), new Set(c.sel), c.d, (p) => p.value + c.dv),
      ).toEqual(c.out);
    for (const c of fx.thin) expect(thin(P(c.pts), c.tol)).toEqual(c.out);
  });
});
