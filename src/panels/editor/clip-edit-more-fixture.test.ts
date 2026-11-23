// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Checks copy and paste, previews, layers, crossfades, groups, insert time,
// clear range and copy-moved in TypeScript against the fixture the Rust test
// reads too (shared-fixtures/clip-edit-3.json).
import { describe, expect, it } from "vitest";
import fx from "../../../shared-fixtures/clip-edit-3.json";
import { settings } from "../../stores/settings";
import * as E from "./clip-edit";

const J = (v: unknown) => JSON.parse(JSON.stringify(v ?? null));
const counter = () => {
  let n = 0;
  return () => `n${++n}0`;
};
const doc = fx.doc as never;

describe("clip edit more fixture", () => {
  it("copy and paste", () => {
    for (const c of fx.copies) expect(J(E.copyClips(doc, c.ids))).toEqual(c.out);
    for (const c of fx.pastes) expect(J(E.pasteClips(doc, c.board as never, c.at, c.row, counter()))).toEqual(c.out);
    expect(J(E.pasteClips(doc, { items: [], media: [] }, 0, 0, counter()))).toEqual(fx.pasteEmpty[0]!.out);
    expect(J(E.pasteClips({ ...fx.doc, tracks: [] } as never, fx.pasteEmpty[1]!.board as never, 0, 0, counter()))).toEqual(fx.pasteEmpty[1]!.out);
  });
  it("previews, layers and crossfades", () => {
    expect(settings.current.fadeShape).toBe(fx.shape);
    for (const c of fx.previews) expect(J(E.previewTracks(doc, c.changes as never))).toEqual(c.out);
    for (const c of fx.layers) expect(J(E.withLayers(doc, c.changes as never))).toEqual(c.out);
    for (const c of fx.crossfades) expect(J(E.withCrossfades(doc, c.changes as never))).toEqual(c.out);
    for (const c of fx.crossfades2) expect(J(E.withCrossfades(fx.xdoc as never, c.changes as never))).toEqual(c.out);
  });
  it("shapes and groups", () => {
    const clips = fx.doc.tracks.flatMap((t) => t.clips) as never[];
    for (const c of fx.shapes) expect(J(E.fadeShapeChanges(clips, c.shape))).toEqual(c.out);
    for (const c of fx.groups) {
      E.setGroupsSuspended(c.sus as boolean);
      const out = E.withGroups(doc, c.ids as string[]);
      E.setGroupsSuspended(false);
      expect(out).toEqual(c.out);
    }
  });
  it("inserting, clearing and copying", () => {
    const run = (fn: () => unknown) => {
      try {
        return J(fn());
      } catch (e) {
        return { locked: e instanceof E.LockedError };
      }
    };
    for (const c of fx.ins) expect(run(() => E.insertTime(doc, c.tracks as string[], c.at as number, c.len as number, counter()))).toEqual(c.out);
    for (const c of fx.clr) expect(run(() => E.clearRange(doc, c.tracks as string[], c.a as number, c.b as number, counter()))).toEqual(c.out);
    for (const c of fx.cm) expect(J(E.copyMoved(doc, c.moving as string[], c.changes as never, counter()))).toEqual(c.out);
  });
});
