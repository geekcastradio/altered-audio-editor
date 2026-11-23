// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Checks take lanes and editor rows in TypeScript against the fixture the Rust
// test reads too (shared-fixtures/takes-rows.json).
import { describe, expect, it } from "vitest";
import fx from "../../../shared-fixtures/takes-rows.json";
import { LockedError } from "./clip-edit";
import { laneRows, moveRowOps } from "./rows";
import * as T from "./takes";

const J = (v: unknown) => JSON.parse(JSON.stringify(v ?? null));
const counter = () => {
  let n = 0;
  return () => `n${++n}0`;
};
const doc = fx.doc as never;

describe("takes and rows fixture", () => {
  it("takes", () => {
    expect(J(T.takeSets(doc))).toEqual(fx.sets);
    for (const c of fx.nums) expect(T.nextTakeNumber(doc, c.set)).toBe(c.out);
    const clips = fx.doc.tracks.flatMap((t) => t.clips);
    for (const c of fx.offs) expect(T.isOffTake(clips.find((k) => k.id === c.id) as never)).toBe(c.out);
    for (const c of fx.makes) expect(J(T.makeTakesOps(doc, c.ids, () => "newset"))).toEqual(c.out);
    for (const c of fx.picks) {
      let out: unknown;
      try {
        out = J(T.pickRangeOps(doc, c.set, c.number, c.a as number, c.b as number, counter()));
      } catch (e) {
        out = { locked: e instanceof LockedError };
      }
      expect(out).toEqual(c.out);
    }
    for (const c of fx.flat) expect(J(T.flattenOps(doc, c.set))).toEqual(c.out);
    for (const c of fx.dels) expect(J(T.deleteTakeOps(doc, c.set as string, c.number as number))).toEqual(c.out);
    for (const c of fx.gaps) expect(J(T.silentGaps(doc, c.set))).toEqual(c.out);
  });
  it("rows", () => {
    const docs = fx.rdocs as never[];
    for (const c of fx.rows) {
      const rows = laneRows(docs[c.di]!, new Set(c.listed)).map((r) => (r.kind === "track" ? { kind: "track", id: r.id, i: r.i } : { kind: "bus", id: r.id }));
      expect(J(rows)).toEqual(c.out);
    }
    for (const c of fx.moves) expect(J(moveRowOps(docs[c.di]!, new Set(c.listed), c.id, c.before))).toEqual(c.out);
  });
});
