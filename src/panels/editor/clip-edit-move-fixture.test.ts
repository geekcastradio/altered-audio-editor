// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Checks moving, ripple delete and gaps in TypeScript against the fixture the
// Rust test reads too (shared-fixtures/clip-edit-2.json).
import { describe, expect, it } from "vitest";
import fx from "../../../shared-fixtures/clip-edit-2.json";
import * as E from "./clip-edit";

const J = (v: unknown) => JSON.parse(JSON.stringify(v ?? null));
const counter = () => {
  let n = 0;
  return () => `n${++n}0`;
};
const doc = fx.doc as never;

describe("clip edit move fixture", () => {
  it("moving", () => {
    for (const c of fx.moves) expect(J(E.moveClips(doc, c.ids as string[], c.delta as number, c.rows as number, c.targets as number[], c.tol as number))).toEqual(c.out);
  });
  it("ripple deleting", () => {
    for (const c of fx.rdr) {
      let out: unknown;
      try {
        out = J(E.rippleDeleteRange(doc, c.tracks, c.a, c.b, counter(), c.only ? new Set(c.only as string[]) : undefined));
      } catch (e) {
        out = { locked: e instanceof E.LockedError };
      }
      expect(out).toEqual(c.out);
    }
    for (const c of fx.rdc) {
      let out: unknown;
      try {
        out = J(E.rippleDeleteClips(doc, c.ids));
      } catch (e) {
        out = { locked: e instanceof E.LockedError };
      }
      expect(out).toEqual(c.out);
    }
  });
  it("gaps", () => {
    for (const c of fx.gaps) {
      expect(J(E.gapAt(doc, c.track, c.at))).toEqual(c.gap);
      expect(J(E.gapToClose(doc, c.track, c.at))).toEqual(c.close);
      expect(E.rippleTracks(doc, c.track)).toEqual(c.rt);
    }
  });
});
