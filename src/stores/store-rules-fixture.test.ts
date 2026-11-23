// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Checks the store rules in TypeScript against the fixture the Rust test
// reads too (shared-fixtures/store-rules.json).
import { describe, expect, it } from "vitest";
import fx from "../../shared-fixtures/store-rules.json";
import { parseSettings } from "./editor";
import * as L from "./libraries";
import { fileName } from "./media-pool";
import { isBlank } from "./session";
import * as T from "./transport";

const J = (v: unknown) => JSON.parse(JSON.stringify(v ?? null));

describe("store rules fixture", () => {
  it("paths, tags and names", () => {
    for (const c of fx.abs) expect(L.isAbsolutePath(c.p)).toBe(c.out);
    for (const c of fx.join) expect(L.joinPath(c.dir, c.rel)).toBe(c.out);
    for (const c of fx.item) expect(L.itemPath(c.dir, { file: c.file } as never)).toBe(c.out);
    for (const c of fx.tags) expect(L.parseTags(c.raw)).toEqual(c.out);
    for (const c of fx.retag) expect(L.retag(c.tags, c.add, c.remove)).toEqual(c.out);
    for (const c of fx.folder) expect(L.folderName(c.n)).toBe(c.out);
    for (const c of fx.file) expect(fileName(c.p)).toBe(c.out);
  });
  it("meters", () => {
    for (const run of fx.meter) {
      T.setMeterSpeed(run.speed as never);
      let m = T.quiet();
      let hold = 0;
      for (const step of run.steps) {
        const f = T.foldMeter(m, step.r, hold);
        m = f.meter;
        hold = f.hold;
        expect(m.level).toBeCloseTo(step.meter.level, 9);
        expect(m.peak).toBeCloseTo(step.meter.peak, 9);
        expect(m.over).toBe(step.meter.over);
        expect(hold).toBe(step.hold);
      }
    }
    T.setMeterSpeed("fast");
    for (const c of fx.frac) expect(T.meterFraction(c.p, c.f)).toBeCloseTo(c.out, 9);
  });
  it("blank sessions and editor settings", () => {
    for (const c of fx.blank) expect(isBlank(c.doc as never)).toBe(c.out);
    for (const c of fx.editor) expect(J(parseSettings(c.raw, fx.base as never))).toEqual(c.out);
  });
});
