// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Checks the TypeScript band split, Phase Curve editor and Fit Music to Length rules against the
// same fixture the Rust test reads (shared-fixtures/band-phase-fit.json).
import { describe, expect, it } from "vitest";
import fx from "../../shared-fixtures/band-phase-fit.json";
import * as B from "../panels/properties/band-split";
import * as P from "../panels/voice-chain/phase-curve";
import * as F from "../panels/editor/musicfit";

const J = (v: unknown) => JSON.parse(JSON.stringify(v));
const counter = () => {
  let n = 0;
  return () => `u${++n}`;
};
const nums = (a: (number | null)[]) => a.map((n) => (n === null ? NaN : n));

describe("band split, phase curve and music fit fixture", () => {
  it("band split", () => {
    expect([B.BAND_MIN_HZ, B.BAND_MAX_HZ, B.BAND_MIN_RATIO]).toEqual([fx.consts.min, fx.consts.max, fx.consts.ratio]);
    for (const c of fx.parse) expect(J(B.parseEdges(c.t))).toEqual(c.out);
    for (const c of fx.check) expect(B.checkEdges(nums(c.e))).toEqual(c.out);
    for (const c of fx.slots) expect(J(B.bandSlots(c.k, c.e, counter()))).toEqual(c.out);
    for (const c of fx.splitCases) {
      const found = c.sel.map((s) => {
        const track = c.tracks[s.ti]!;
        return { trackIndex: s.ti, track, clip: track.clips.find((k) => k.id === s.id)! };
      });
      expect(J(B.bandSplitOp({} as never, found as never, c.edges, counter()))).toEqual(c.out);
    }
  });
  it("phase curve", () => {
    for (const c of fx.phase) {
      const slot = c.slot as never;
      expect(P.pointCount(slot)).toBe(c.count);
      expect(P.rangeOf(slot)).toBe(c.range);
      const pts = P.curvePoints(slot);
      expect(J(pts)).toEqual(c.pts);
      for (const a of c.at) expect(P.curveAt(pts, a.sp, a.hz)).toBe(a.out);
      expect(J(P.addPoint(slot, 123, -45))).toEqual(c.add);
      c.rem.forEach((w, i) => expect(J(P.removePoint(slot, i))).toEqual(w));
      for (const s of c.shown) expect(P.groupShown(s.effect, slot, s.g)).toBe(s.out);
    }
    expect(P.curveAt([], true, 100)).toBe(fx.emptyAt);
  });
  it("fit music", () => {
    expect(F.FIT_XFADE).toBe(fx.consts.xfade);
    for (const c of fx.fit) expect(J(F.fitOps({ sample_rate: 48000 } as never, "tr", c.clip as never, c.media as never, c.plan, counter()))).toEqual(c.out);
    for (const c of fx.desc) {
      expect(F.describePlan(c.p)).toBe(c.plan);
      expect(F.describeJoins(c.p, c.origin)).toBe(c.joins);
    }
    for (const c of fx.same) expect(F.sameLength(c.a, c.b)).toBe(c.out);
  });
});
