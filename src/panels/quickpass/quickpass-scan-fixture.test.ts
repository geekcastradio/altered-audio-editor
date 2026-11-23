// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Checks the QuickPass scanning rules in TypeScript against the fixture the Rust test reads too
// (shared-fixtures/quickpass-scan.json).
import { describe, expect, it } from "vitest";
import fx from "../../../shared-fixtures/quickpass-scan.json";
import * as Q from "./quickpass";

const J = (v: unknown) => JSON.parse(JSON.stringify(v ?? null));
const map = new Map(Object.entries(fx.tracks));
const lv = fx.lv as never[] as { db: number[]; peak: number[] }[];

describe("quickpass scan fixture", () => {
  it("settings", () => {
    for (const c of fx.sanitize) expect(J(Q.sanitizeSettings(c.raw))).toEqual(c.out);
  });
  it("scans", () => {
    for (const c of fx.scans) {
      const s = c.s as never;
      expect(J(Q.findAll(fx.lv as never, map, s, 44100))).toEqual(c.out);
      expect(J(Q.silentRuns(lv[0]!.db, 0.01, s))).toEqual(c.runs.silent);
      expect(J(Q.noiseRuns(lv[1]!.db, 0.02, s))).toEqual(c.runs.noise);
      expect(J(Q.soundRuns(lv[2]!.db, 0.01, s))).toEqual(c.runs.sound);
      expect(J(Q.clipRuns(lv[0]!.peak, s))).toEqual(c.runs.clip);
      expect(J(Q.spotsRequest(s))).toEqual(c.req);
    }
  });
  it("spots, levels and labels", () => {
    for (const c of fx.spotOut) expect(J(Q.spotFindings(fx.spots as never, map, c.kind as never))).toEqual(c.out);
    for (const c of fx.suggest) expect(J(Q.suggestLevels(c.db))).toEqual(c.out);
    for (const c of fx.labels) expect(Q.scanLabel(c.p)).toBe(c.out);
  });
});
