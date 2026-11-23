// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Checks the automation lane helpers in TypeScript against the fixture the
// Rust test reads too (shared-fixtures/auto-lanes.json).
import { describe, expect, it, vi } from "vitest";
import fx from "../../../shared-fixtures/auto-lanes.json";
import * as A from "./automation";

const J = (v: unknown) => JSON.parse(JSON.stringify(v ?? null));
const catalog = fx.catalog as never[];
const owners = fx.owners as never[];
const targets = fx.targets as never[];

describe("auto lanes fixture", () => {
  it("lists choices and describes targets", () => {
    for (const c of fx.choices) expect(J(A.autoChoices(owners[c.oi]!, catalog, c.in3d))).toEqual(c.out);
    for (const c of fx.per) {
      const [o, t] = [owners[c.oi]!, targets[c.ti]!];
      expect(A.autoName(o, catalog, t)).toBe(c.name);
      expect(A.isOrphan(o, catalog, t)).toBe(c.orphan);
      expect(J(A.autoRange(o, catalog, t))).toEqual(c.range);
      expect(A.autoRest(o, catalog, t)).toBe(c.rest);
    }
    expect(targets.map((t) => A.targetKey(t))).toEqual(fx.keys);
    for (const c of fx.next) expect(J(A.nextTarget(owners[c.oi]!, catalog, c.in3d))).toEqual(c.out);
  });
  it("edits lanes and modes", () => {
    owners.forEach((o, i) => expect(J(A.shownLanes(o))).toEqual(fx.shown[i]));
    for (const c of fx.drop) expect(J(A.dropLane(owners[c.oi]!, c.id))).toEqual(c.out);
    owners.forEach((o, i) => expect(A.autoMode(o)).toBe(fx.mode[i]));
    for (const c of fx.modeChange) expect(J(A.modeChange(owners[c.oi]!, c.mode as never))).toEqual(c.out);
    for (const c of fx.setMode) expect(J(A.setMode(owners[c.oi]!, c.mode as never))).toEqual(c.out);
    let n = 0;
    vi.spyOn(crypto, "randomUUID").mockImplementation(() => `id-${++n}` as never);
    expect(targets.slice(0, 3).map((t) => J(A.newLane(t)))).toEqual(fx.newLane);
    for (const c of fx.put) expect(J(A.putLane(owners[c.oi]!, fx.lanes[c.li] as never))).toEqual(c.out);
    expect(n).toBe(fx.idsUsed);
  });
});
