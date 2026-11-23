// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Checks the QuickPass edits in TypeScript against the fixture the Rust test
// reads too (shared-fixtures/quickpass-edit.json).
import { describe, expect, it } from "vitest";
import fx from "../../../shared-fixtures/quickpass-edit.json";
import { LockedError } from "../editor/clip-edit";
import * as Q from "./quickpass";

const J = (v: unknown) => JSON.parse(JSON.stringify(v ?? null));
const counter = () => {
  let n = 0;
  return () => `n${++n}0`;
};
const doc = fx.doc as never;
const lists = fx.lists as never[][];

describe("quickpass edit fixture", () => {
  it("removing, splitting and ducking", () => {
    for (const c of fx.removes) {
      let out: unknown;
      try {
        out = J(Q.removeOp(doc, lists[c.li]!, { ripple: c.ripple, keepMs: c.keepMs } as never, counter()));
      } catch (e) {
        out = { locked: e instanceof LockedError };
      }
      expect(out).toEqual(c.out);
    }
    for (const c of fx.splits) expect(J(Q.splitOp(doc, lists[c.li]!, counter()))).toEqual(c.out);
    for (const c of fx.clipsOn) {
      const op = Q.splitOp(doc, lists[c.li]!, counter());
      expect(Q.clipsOn(doc, lists[c.li]!)).toEqual(c.before);
      expect(Q.clipsOn(op ? Q.applyLocal(doc, op) : doc, lists[c.li]!)).toEqual(c.after);
    }
    for (const c of fx.ducks) expect(J(Q.duckOp(doc, lists[c.li]!, c.duckDb))).toEqual(c.out);
  });
  it("markers, shifts, cuts and local edits", () => {
    for (const c of fx.markers) expect(J(Q.markersFor(doc, lists[c.li]!, counter(), c.base))).toEqual(c.out);
    for (const c of fx.shifts) {
      const all = lists[c.li]! as { id: string }[];
      const gone = all.find((x) => x.id === c.goneId)!;
      expect(J(Q.shiftFindings(all.filter((x) => x !== gone) as never, gone as never, c.ripple as never, c.keepMs, 48000))).toEqual(c.out);
    }
    for (const c of fx.cuts) expect(J(Q.cutRange({ start: c.start, end: c.end }, c.keepMs, 48000))).toEqual(c.out);
    for (const c of fx.ops) expect(J(Q.applyLocal(doc, c.op as never))).toEqual(c.out);
  });
});
