// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Checks the controller bindings and feedback frames in TypeScript against
// the fixture the Rust test reads too (shared-fixtures/controller-map.json).
import { describe, expect, it } from "vitest";
import fx from "../../../shared-fixtures/controller-map.json";
import * as F from "./feedback";
import * as M from "./map";

const J = (v: unknown) => JSON.parse(JSON.stringify(v ?? null));

describe("controller map fixture", () => {
  it("names and choices", () => {
    for (const c of fx.sources) {
      expect(M.sourceKey(c.s as never)).toBe(c.key);
      expect(M.sourceName(c.s as never)).toBe(c.name);
    }
    for (const c of fx.targets) expect(M.targetName(c.t as never)).toBe(c.name);
    expect(M.ACTIONS).toEqual(fx.actions);
    for (const c of fx.choices) expect(J(M.targetChoices(c.n))).toEqual(c.out);
    expect(J(M.mackiePreset())).toEqual(fx.mackie);
  });
  it("messages", () => {
    for (const c of fx.pan) expect(M.panAt(c.p)).toBe(c.out);
    for (const c of fx.turn) expect(M.turn(c.v)).toBe(c.out);
    for (const c of fx.position) expect(M.position({ kind: c.kind, value: c.value } as never)).toBeCloseTo(c.out, 12);
    for (const c of fx.matches) expect(M.matches(c.b as never, c.m as never)).toBe(c.out);
  });
  it("feedback", () => {
    for (const c of fx.fader14) expect(F.fader14(c.p)).toBe(c.out);
    for (const c of fx.ring) expect(F.ringByte(c.p)).toBe(c.out);
    const strips = fx.strips.map((s) => s ?? undefined) as never;
    for (const c of fx.frames) expect(J(F.frames(c.b as never, strips, c.m))).toEqual(c.out);
  });
});
