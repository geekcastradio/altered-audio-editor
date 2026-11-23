// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Checks clip editing in TypeScript against the fixture the Rust test reads too
// (shared-fixtures/clip-edit.json).
import { describe, expect, it } from "vitest";
import fx from "../../../shared-fixtures/clip-edit.json";
import { settings } from "../../stores/settings";
import * as E from "./clip-edit";

const J = (v: unknown) => JSON.parse(JSON.stringify(v ?? null));
const counter = () => {
  let n = 0;
  return () => `n${++n}0`;
};
const doc = fx.doc as never;
const clipOf = (id: string) => fx.doc.tracks.flatMap((t) => t.clips).find((c) => c.id === id) as never;

describe("clip edit fixture", () => {
  it("ends and ids", () => {
    const media = new Map(fx.doc.media.map((m) => [m.id, m])) as never;
    for (const c of fx.ends) expect(E.clipEnd(clipOf(c.id), media, 48000)).toBe(c.out);
    for (const c of fx.envIds) expect(E.envelopeId(c.id, c.kind as never)).toBe(c.out);
  });
  it("splitting and pieces", () => {
    for (const c of [...fx.split, ...fx.splitAll]) expect(J(E.splitClips(doc, c.ids, c.at, counter()))).toEqual(c.out);
    for (const c of fx.pieces) {
      const k = clipOf(c.id) as { media_id: string };
      const m = fx.doc.media.find((x) => x.id === k.media_id) as never;
      expect(J(E.pieceOf(k as never, m, 48000, c.from, c.to, c.start, "p1"))).toEqual(c.out);
    }
    for (const c of fx.isolates) {
      let out: unknown;
      try {
        out = J(E.isolate(doc, c.tracks as string[], c.a as number, c.b as number, counter()));
      } catch (e) {
        out = { locked: e instanceof E.LockedError };
      }
      expect(out).toEqual(c.out);
    }
  });
  it("trimming and fades", () => {
    for (const c of fx.trims) expect(J(E.trimClip(doc, clipOf(c.id), c.edge as never, c.delta, c.targets as never, c.tol as number))).toEqual(c.out);
    expect(settings.current.fadeShape).toBe(fx.fadeShape);
    for (const c of fx.fades) expect(J(E.fadeClip(doc, clipOf(c.id), c.which as never, c.length))).toEqual(c.out);
    for (const c of fx.loops) expect(J(E.loopChanges(doc, c.ids))).toEqual(c.out);
    for (const c of fx.ttp) expect(J(E.trimToPlayhead(doc, [c.id], c.at, c.edge as never))).toEqual(c.out);
    for (const c of fx.ttr) expect(J(E.trimToRange(doc, c.ids, c.a as number, c.b as number))).toEqual(c.out);
  });
  it("clips under a point", () => {
    for (const c of fx.clipsAt) expect(E.clipsAt(doc, c.at as number, new Set(c.pref as string[]))).toEqual(c.out);
  });
});
