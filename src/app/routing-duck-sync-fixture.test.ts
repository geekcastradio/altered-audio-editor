// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Checks the TypeScript mix routing, Duck Music Under Voice and Sync Tracks rules against the same
// fixture the Rust test reads (shared-fixtures/routing-duck-sync.json).
import { describe, expect, it } from "vitest";
import fx from "../../shared-fixtures/routing-duck-sync.json";
import * as R from "../panels/mixer/routing";
import * as D from "./duck";
import * as S from "./sync-tracks";
import { LockedError } from "../panels/editor/clip-edit";

const J = (v: unknown) => JSON.parse(JSON.stringify(v));
const docs = fx.docs as never[];
const idsOf = (l: { id: string }[]) => l.map((x) => x.id);

describe("routing, duck and sync fixture", () => {
  it("routing", () => {
    for (const c of fx.routing) {
      const doc = docs[c.di]!;
      for (const p of c.pairs) expect(R.reaches(doc, p.a, p.b)).toBe(p.out);
      for (const b of c.bus) expect(idsOf(R.busChoices(doc, b.o))).toEqual(b.out);
      for (const k of c.keys) expect(J(R.keyChoices(doc, k.o))).toEqual(k.out);
      for (const o of c.outs) {
        expect(R.outputName(doc, (o.o ?? undefined) as never)).toBe(o.name);
        expect(R.outputValue((o.o ?? undefined) as never)).toBe(o.value);
      }
      for (const o of c.ops) expect(J(R.busOp(doc, o.id, o.changes, o.ex))).toEqual(o.out);
      expect(R.isRouted(doc)).toBe(c.routed);
      (doc as { tracks: { effects?: never }[] }).tracks.forEach((t, i) => expect(R.keySources(t.effects)).toEqual(c.sources[i]));
    }
    expect(R.isRouted(null)).toBe(fx.noDoc.routed);
    expect(J(R.busOp(null, "b", { solo: true }, true))).toEqual(fx.noDoc.op);
    for (const c of fx.picks) expect(J(R.outputOf(c.v))).toEqual(c.out);
    for (const c of fx.patches) expect(J(R.setSend([R.newSend("b1", "s1"), R.newSend("b2", "s2")], "s2", c.p))).toEqual(c.out);
    for (const c of fx.removes) expect(J(R.removeSend([R.newSend("b1", "s1"), R.newSend("b2", "s2")], c.id))).toEqual(c.out);
  });
  it("duck", () => {
    expect(D.DUCK_PRESET).toBe(fx.consts.preset);
    expect(D.DUCK_THRESHOLD_DB).toBe(fx.consts.thr);
    expect(J(D.DUCK_DEPTHS)).toEqual(fx.consts.depths);
    for (const c of fx.params) {
      expect(J(D.duckParams(c.db))).toEqual(c.out);
      expect(J(D.duckSlot("id", "v1", c.db))).toEqual(c.slot);
    }
    for (const c of fx.slotTests) expect(D.isDuckSlot(c.sl as never)).toBe(c.out);
    for (const c of fx.duck) {
      const doc = docs[c.di] as unknown as { tracks: { id: string; effects?: never }[] };
      const find = (id: string) => doc.tracks.find((t) => t.id === id)! as never;
      doc.tracks.forEach((t, i) => {
        expect(D.hasDuck(t as never)).toBe(c.has[i]);
      });
      for (const p of c.plans) {
        const got = D.duckPlan(doc as never, p.sel === null ? null : find(p.sel));
        expect(got === null ? null : { targets: idsOf(got.targets), keys: idsOf(got.keys), byRole: got.byRole }).toEqual(p.out);
      }
      for (const k of c.keys) expect(idsOf(D.duckKeys(doc as never, find(k.id)))).toEqual(k.out);
      for (const w of c.withs) {
        const t = find(w.t) as { effects?: never };
        const chain = (t.effects ?? { enabled: true, input_gain_db: 0, output_gain_db: 0, slots: [] }) as never;
        let n = 0;
        expect(J(D.withDuck(chain, w.v, 10, () => `n${++n}`, w.max))).toEqual(w.out);
        expect(J(D.withoutDuck(chain))).toEqual(w.without);
      }
    }
  });
  it("sync", () => {
    expect(S.DRIFT_QUIET_PPM).toBe(fx.consts.quiet);
    for (const c of fx.moves) {
      let out: unknown;
      try {
        out = S.moveTrack(docs[c.di]!, c.id, c.off);
      } catch (e) {
        out = e instanceof LockedError ? { throws: "locked" } : "other";
      }
      expect(J(out)).toEqual(c.out);
    }
    for (const c of fx.describes) expect(S.describeSync(c.r, c.rate)).toEqual(c.out);
    for (const c of fx.trusts) expect(S.trust({ offset: 0, driftPpm: 0, confidence: c.c, inverted: false, windows: 1 })).toBe(c.out);
  });
});
