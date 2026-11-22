// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Checks the TypeScript User Guide reader and session document rules against the same fixture the
// Rust test reads (shared-fixtures/guide-media.json).
import { describe, expect, it } from "vitest";
import fx from "../../shared-fixtures/guide-media.json";
import * as G from "./guide/guide";
import * as SM from "./session-media";
import * as SP from "./session-properties";
import * as B from "./bleed";
import * as H from "./host-profiles";

const J = (v: unknown) => JSON.parse(JSON.stringify(v));

describe("guide and session rules fixture", () => {
  it("guide reading", () => {
    for (const c of fx.parsed) expect(J(G.parseGuide(c.md))).toEqual(c.out);
    for (const c of fx.spanCases) expect(J(G.spans(c.l))).toEqual(c.out);
    const sections = G.parseGuide(fx.parsed[fx.parsed.length - 1]!.md);
    for (const c of fx.searches) expect(G.searchGuide(sections, c.q).map((s) => s.id)).toEqual(c.out);
  });
  it("media copy rules", () => {
    const paths = ["C:\\Audio\\a.wav", "c:/audio/A.wav", "C:\\Session\\x\\b.wav", "C:\\Session\\x\\b.wav", "/home/u/lib/c.wav", "/home/u/lib2/d.wav", "D:\\Other\\e.wav", "C:\\Session\\xx\\f.wav", "/home/u/Lib/g.wav"];
    for (const c of fx.copies) {
      expect(SM.folderOf(c.f)).toBe(c.home);
      expect(SM.toCopy(paths, c.f, c.leave, c.ic)).toEqual(c.out);
    }
    for (const c of fx.inside) expect([true, false].map((ic) => SM.insideAny(c.p as string, c.d as string[], ic))).toEqual(c.ic);
    for (const c of fx.repoints) expect(J(SM.repointPlan(c.media as never, c.pool, new Map(c.moved as [string, string][]), c.ic))).toEqual(c.out);
  });
  it("session properties and lists", () => {
    for (const c of fx.props) expect(J(SP.propertiesOp(c.doc as never, c.next as never))).toEqual(c.out);
    for (const c of fx.current) expect(J(SP.currentProperties(c.doc as never))).toEqual(c.out);
    for (const c of fx.bleeds) expect(J(B.replaceOps("t1", c.files as never, new Set(c.known)))).toEqual(c.out);
    expect(J([SP.SAMPLE_RATES, SP.BIT_DEPTHS, SP.PAN_LAWS, SP.LOUDNESS_CHOICES])).toEqual([fx.consts.rates, fx.consts.depths, fx.consts.pans, fx.consts.loud]);
    expect(J(H.HOST_PROFILES)).toEqual(fx.profiles.list);
    for (const c of fx.profiles.ids) expect(H.profileById(c.id)?.name ?? null).toBe(c.out);
  });
});
