// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Checks the clip edit helpers in TypeScript against the fixture the Rust
// test reads too (shared-fixtures/edit-ops.json).
import { describe, expect, it } from "vitest";
import fx from "../../../shared-fixtures/edit-ops.json";
import { silenceChanges } from "./silence-span";
import { stackChanges } from "./clip-order";
import { clipById } from "./clip-lookup";
import { alignOps, alignPair } from "./align";
import { findMidiClips, snapTargets } from "./clip-edit";

const J = (v: unknown) => JSON.parse(JSON.stringify(v ?? null));
const doc = fx.doc as never;

describe("edit ops fixture", () => {
  it("silences and stacks", () => {
    for (const c of fx.silence) expect(J(silenceChanges(doc, new Set(fx.sets[c.si]), fx.ranges[c.ri]![0]!, fx.ranges[c.ri]![1]!))).toEqual(c.out);
    for (const c of fx.stack) expect(J(stackChanges(doc, new Set(fx.sets[c.si]), c.move as "front" | "back"))).toEqual(c.out);
  });
  it("looks up clips", () => {
    for (const c of fx.lookup) expect(J(clipById(doc, c.id))).toEqual(c.out);
    for (const c of fx.midi) expect(J(findMidiClips(doc, c.ids))).toEqual(c.out);
  });
  it("snaps and aligns", () => {
    for (const c of fx.snap) expect(J(snapTargets(doc, new Set(c.ids), c.ph, c.use as never))).toEqual(c.out);
    for (const c of fx.pairs) expect(J(alignPair(doc, c.p))).toEqual(c.out);
    for (const c of fx.ops) expect(J(alignOps(doc, c.other, c.track, { lag: c.lag, score: 0.9, inverted: c.inverted }))).toEqual(c.out);
  });
});
