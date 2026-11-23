// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Checks list selection, folder and solo helpers in TypeScript against the
// fixture the Rust test reads too (shared-fixtures/panel-nav.json).
import { describe, expect, it } from "vitest";
import fx from "../../../shared-fixtures/panel-nav.json";
import { parentFolder } from "../drive-manager/paths";
import { silentTracks, trackOp } from "../mixer/solo";
import { formatSeconds, planDuration } from "../properties/remix";
import { clickSelection, stepSelection } from "./selection";

const J = (v: unknown) => JSON.parse(JSON.stringify(v ?? null));

describe("panel nav fixture", () => {
  it("selects and steps", () => {
    for (const c of fx.click) expect(J(clickSelection(new Set(c.current), c.anchor, c.clicked, fx.visible, { ctrl: c.ctrl, shift: c.shift }))).toEqual(c.out);
    for (const c of fx.step) expect(J(stepSelection(c.v, c.from, c.delta))).toEqual(c.out);
    for (const c of fx.paths) expect(J(parentFolder(c.p))).toEqual(c.out);
  });
  it("applies solo rules", () => {
    fx.docs.forEach((d, i) => expect([...silentTracks(d as never)].sort()).toEqual(fx.silent[i]));
    for (const c of fx.op) expect(J(trackOp(fx.docs[c.di] as never, c.id, c.changes, c.exclusive))).toEqual(c.out);
    expect(J(trackOp(null, "t1", { solo: true }, true))).toEqual(fx.opNull);
  });
  it("plans and formats remix times", () => {
    for (const c of fx.plan) expect(planDuration(c.p as never)).toBe(c.out);
    for (const c of fx.fmt) expect(formatSeconds(typeof c.s === "string" ? Number(c.s) : c.s)).toBe(c.out);
  });
});
