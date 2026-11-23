// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Checks the Waveform editor edits in TypeScript against the fixture the Rust
// test reads too (shared-fixtures/wave-edit.json).
import { describe, expect, it } from "vitest";
import fx from "../../../shared-fixtures/wave-edit.json";
import { settings } from "../../stores/settings";
import { LockedError } from "./clip-edit";
import * as W from "./wave-edit";

const J = (v: unknown) => JSON.parse(JSON.stringify(v ?? null));
const counter = () => {
  let n = 0;
  return () => `n${++n}0`;
};
const run = (fn: () => unknown) => {
  try {
    return J(fn());
  } catch (e) {
    return { locked: e instanceof LockedError };
  }
};
const docs = fx.docs as never[];
const doc = (c: { i: number }) => docs[c.i]!;
const pair = (r: { op: unknown; length: number } | null) => (r ? { op: r.op, length: r.length } : null);

describe("wave edit fixture", () => {
  it("ranges and markers", () => {
    expect(settings.current.fadeShape).toBe(fx.shape);
    for (const c of fx.clamp) expect(J(W.clampRange(doc(c), c.a, c.b))).toEqual(c.out);
    for (const c of fx.marks) {
      expect(J(W.markersAfterCut(docs[0]!, c.a, c.b))).toEqual(c.cut);
      expect(J(W.markersAfterCrop(docs[0]!, c.a, c.b))).toEqual(c.crop);
      expect(J(W.markersAfterInsert(docs[0]!, c.a, c.b - c.a))).toEqual(c.ins);
    }
    for (const c of fx.fadeDb) expect(W.fadeDb(c.t, c.s as "power" | "linear")).toBeCloseTo(c.out, 9);
  });
  it("range edits", () => {
    for (const c of fx.del) expect(run(() => W.deleteRange(doc(c), c.a, c.b, counter()))).toEqual(c.out);
    for (const c of fx.crop) expect(run(() => W.cropTo(doc(c), c.a, c.b, counter()))).toEqual(c.out);
    for (const c of fx.sil) expect(run(() => W.silenceRange(doc(c), c.a, c.b, counter()))).toEqual(c.out);
    for (const c of fx.amp) expect(run(() => W.amplifyRange(doc(c), c.a, c.b, c.db, counter()))).toEqual(c.out);
    for (const c of fx.fade) expect(run(() => W.fadeRange(doc(c), c.a, c.b, c.d as "in" | "out", counter(), c.s as "power" | "linear"))).toEqual(c.out);
  });
  it("copy and paste", () => {
    const n = fx.copy.length;
    fx.copy.forEach((c, k) => expect(J(W.copyRange(docs[k < n - 3 ? 0 : 1]!, c.a, c.b))).toEqual(c.out));
    for (const c of fx.mix) expect(run(() => pair(W.mixPasteAt(doc(c), fx.boards[c.bi] as never, c.at, c.db, counter())))).toEqual(c.out);
    for (const c of fx.paste) expect(run(() => pair(W.pasteAt(doc(c), fx.boards[c.bi] as never, c.at, counter())))).toEqual(c.out);
  });
  it("joins, replace, stretch and insert", () => {
    for (const c of fx.seams) expect(run(() => W.crossfadeSeams(doc(c), c.s, c.len))).toEqual(c.out);
    for (const c of fx.repl) expect(run(() => W.replaceRange(doc(c), c.a, c.b, c.m as never, "Fixed", counter()))).toEqual(c.out);
    for (const c of fx.str) expect(run(() => W.stretchRange(doc(c), c.a, c.b, c.m as never, "Slow", counter()))).toEqual(c.out);
    const media = [fx.media.mediaA, fx.media.mediaB, { ...fx.media.mediaA, length: 0 }];
    for (const c of fx.ins) expect(run(() => pair(W.insertFile(doc(c), c.at, media[c.mi] as never, "Take", counter())))).toEqual(c.out);
  });
});
