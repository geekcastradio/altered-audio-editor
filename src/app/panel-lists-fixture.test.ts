// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Checks the TypeScript Metadata panel, soundboard pad and library bin rules against the same
// fixture the Rust test reads (shared-fixtures/panel-lists.json).
import { describe, expect, it } from "vitest";
import fx from "../../shared-fixtures/panel-lists.json";
import * as M from "../panels/metadata/metadata";
import * as S from "../panels/soundboard/soundboard";
import * as B from "../panels/libraries/bins";

const J = (v: unknown) => JSON.parse(JSON.stringify(v));

describe("panel lists fixture", () => {
  it("tag sets", () => {
    for (const c of fx.sanitize) expect(J(M.sanitizeSets(c.raw))).toEqual(c.out);
    let sets: M.TagSet[] = [];
    for (const c of fx.upserts) {
      sets = M.upsertSet(sets, c.name, c.values as never);
      expect(J(sets)).toEqual(c.out);
    }
    for (const c of fx.removes) expect(J(M.removeSet(sets, c.name!))).toEqual(c.out);
    let big: M.TagSet[] = [];
    for (let i = 0; i < 105; i++) big = M.upsertSet(big, `Item ${(i * 13) % 105}`, { title: "t" });
    expect(J(big)).toEqual(fx.bigRun);
  });
  it("checks and lists", () => {
    for (const c of fx.checks.year) expect(M.checkYear(c.v)).toBe(c.out);
    for (const c of fx.checks.track) expect(M.checkTrack(c.v)).toBe(c.out);
    for (const c of fx.checks.problems) expect(M.problems({ year: c.y!, track: c.t! })).toEqual(c.out);
    for (const c of fx.baseNames) expect(M.baseName(c.p)).toBe(c.out);
    for (const c of fx.filter) expect(M.filterEntries(fx.entries as never, c.q).map((e) => e.path)).toEqual(c.out);
    for (const c of fx.overlays) expect(J(M.overlay(c.tags as never, c.set as never))).toEqual(c.out);
    for (const c of fx.adds) expect(M.addPaths(c.have as string[], c.more as string[])).toEqual(c.out);
  });
  it("pads", () => {
    for (const c of fx.pads) expect(J(S.sanitizePads(c.raw))).toEqual(c.out);
    for (const c of fx.labels) expect(S.padLabel(c.p)).toBe(c.out);
    for (const c of fx.withs) expect(J(S.withPad(S.emptyPads(), c.i as number, c.p as never))).toEqual(c.out);
  });
  it("bins", () => {
    expect([...B.PALETTE]).toEqual(fx.palette);
    expect(J(B.STARTER_BINS)).toEqual(fx.starterBins);
    for (const c of fx.initialsOf) expect(B.initials(c.n)).toBe(c.out);
    for (const c of fx.colors) expect(B.defaultColor(c.n)).toBe(c.out);
    for (const c of fx.starter) expect(J(B.missingStarterBins({ collections: c.have.map((name) => ({ name })) } as never))).toEqual(c.out);
  });
});
