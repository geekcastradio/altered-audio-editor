// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Checks the Audio FX looks in TypeScript against the fixture the Rust test
// reads too (shared-fixtures/audio-looks.json).
import { describe, expect, it } from "vitest";
import fx from "../../../shared-fixtures/audio-looks.json";
import * as A from "./audiofx";

const J = (v: unknown) => JSON.parse(JSON.stringify(v ?? null));

describe("audio looks fixture", () => {
  it("lists and reads", () => {
    expect(J(A.LOOKS.map((l) => ({ id: l.id, label: l.label })))).toEqual(fx.looks);
    for (const c of fx.effects) expect(J(A.lookEffects(c.id as never, c.s as never))).toEqual(c.out);
    for (const c of fx.read) expect(J(A.readLooks(c.c as never))).toEqual(c.out);
    for (const c of fx.panel) expect(A.isPanelSlot(c.s as never)).toBe(c.out);
  });
  it("writes and names", () => {
    for (const c of fx.with) {
      let n = 0;
      expect(J(A.withLooks(fx.chains[c.ci] as never, fx.sets[c.li] as never, () => `N${++n}`, c.max))).toEqual(c.out);
    }
    for (const c of fx.without) expect(J(A.withoutLooks(fx.chains[c.ci] as never))).toEqual(c.out);
    for (const c of fx.describe) expect(A.describeLooks(fx.sets[c.li] as never)).toBe(c.out);
    for (const c of fx.same) expect(A.sameLooks(fx.sets[c.ai] as never, fx.sets[c.bi] as never)).toBe(c.out);
  });
});
