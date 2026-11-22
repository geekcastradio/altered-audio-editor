// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Checks the TypeScript EQ graph, phase curve, band split, channel mix, Music Fit text and
// Files format rules against the same fixture the Rust tests read
// (shared-fixtures/panel-rules.json).
import { describe, expect, it } from "vitest";
import fx from "../../shared-fixtures/panel-rules.json";
import * as E from "../panels/voice-chain/eq-graph";
import * as P from "../panels/voice-chain/phase-curve";
import * as B from "../panels/properties/band-split";
import * as X from "../panels/properties/channel-mix";
import * as F from "../panels/editor/musicfit";
import * as V from "../panels/files/format";

const slot = (params: Record<string, number>) =>
  ({
    id: "s",
    effect: P.PHASE_CURVE,
    enabled: true,
    params,
    preset: "",
  }) as never;
const any = (c: unknown) => c as Record<string, any>;

describe("panel rules fixture", () => {
  it("EQ graph", () => {
    for (const raw of fx.eq) {
      const c = any(raw);
      if ("dir" in c)
        expect(c.dir === "octaves" ? E.octavesOf(c.q) : E.qOf(c.oct)).toBe(
          c.dir === "octaves" ? c.oct : c.q,
        );
      else if ("edges" in c) expect(E.widthEdges(c.f, c.q)).toEqual(c.edges);
      else if ("qe" in c) expect(E.qFromEdge(c.f, c.at)).toBe(c.qe);
      else if ("text" in c) expect(E.formatHz(c.hz)).toBe(c.text);
    }
  });
  it("phase curve", () => {
    for (const c of fx.phase) {
      const s = slot(c.params);
      expect([P.pointCount(s), P.rangeOf(s), P.curvePoints(s)]).toEqual([
        c.count,
        c.range,
        c.pts,
      ]);
    }
    for (const c of fx.curve)
      expect(P.curveAt(P.curvePoints(slot(c.params)), c.spline, c.hz)).toBe(
        c.out,
      );
    for (const c of fx.edit) {
      const s = slot(c.params);
      expect(P.addPoint(s, 1234, -33)).toEqual(c.add);
      expect(P.addPoint(s, 0, 0)).toEqual(c.adds);
      expect([0, 1, 2, 3, 7, 9].map((i) => P.removePoint(s, i))).toEqual(c.rm);
    }
    for (const c of fx.groups) {
      expect(P.groupShown(P.PHASE_CURVE, slot(c.params), c.g)).toBe(c.on);
      expect(P.groupShown("other", slot(c.params), c.g)).toBe(c.other);
    }
  });
  it("band split", () => {
    for (const c of fx.edges) expect(B.parseEdges(c.t)).toEqual(c.out);
    for (const c of fx.checks) expect(B.checkEdges(c.e)).toBe(c.out);
    for (const c of fx.bands) {
      let n = 0;
      expect(
        B.bandSlots(c.k, c.e, () => String(n++)).map((x) => [
          x.params.mode,
          x.params.freq,
        ]),
      ).toEqual(c.out);
    }
  });
  it("channel mix", () => {
    for (const c of fx.labels) expect(X.channelLabels(c.n)).toEqual(c.out);
    for (const c of fx.mixes) {
      expect(X.defaultMix(c.n).sends).toEqual(c.def);
      expect(
        X.mixOf({ sends: [{ left: 0.3, right: 0.7 }] }, c.n).sends,
      ).toEqual(c.filled);
      expect(X.mixOf(null, c.n).sends).toEqual(c.none);
    }
    for (const c of fx.presets) {
      const got = X.presets(c.n);
      expect(got).toEqual(c.out);
      expect(got.map((p) => X.presetFor(p.mix, c.n))).toEqual(c.match);
      expect(X.presetFor({ sends: [{ left: 0.9, right: 0.1 }] }, c.n)).toBe(
        c.odd,
      );
    }
  });
  it("Music Fit text", () => {
    for (const raw of fx.fits) {
      const c = any(raw);
      if ("same" in c) expect(F.sameLength(c.same[0], c.same[1])).toBe(c.out);
      else {
        expect(F.describePlan(c.p)).toBe(c.plan);
        expect(F.describeJoins(c.p, c.origin)).toBe(c.joins);
      }
    }
  });
  it("Files formats", () => {
    for (const raw of fx.files) {
      const c = any(raw);
      if ("dur" in c)
        expect(
          V.formatDuration({ length: c.dur[0], sampleRate: c.dur[1] } as never),
        ).toBe(c.out);
      else if ("hz" in c) expect(V.formatRate(c.hz)).toBe(c.out);
      else if ("ch" in c) expect(V.formatChannels(c.ch)).toBe(c.out);
      else if ("bytes" in c) expect(V.formatSize(c.bytes)).toBe(c.out);
      else expect(V.statusText({ status: c.st } as never)).toBe(c.out);
    }
  });
});
