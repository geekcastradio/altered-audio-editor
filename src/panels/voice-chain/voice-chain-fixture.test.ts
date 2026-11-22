// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Checks the TypeScript Voice Chain rules against the same fixture the Rust test reads
// (shared-fixtures/voice-chain.json).
import { describe, expect, it } from "vitest";
import fx from "../../../shared-fixtures/voice-chain.json";
import * as C from "./chain";

const J = (v: unknown) => JSON.parse(JSON.stringify(v ?? null));
const counter = () => {
  let n = 0;
  return () => `n${++n}`;
};
const slot = { id: "new", effect: "altaudio.gain", enabled: true, params: {}, preset: null } as never;
type Chain = never;

describe("voice chain fixture", () => {
  it("chain and slot edits", () => {
    for (const c of fx.chainOf) expect(J(C.chainOf(c.c as never))).toEqual(c.out);
    for (const c of fx.newSlots) expect(J(C.newSlot(c.info as never, "id1"))).toEqual(c.out);
    for (const o of fx.ops) {
      const ch = o.c as Chain;
      expect(J(C.layoutSlots(ch))).toEqual(o.layout);
      for (const a of o.add) expect(J(C.addSlot(ch, slot, a.index ?? undefined))).toEqual(a.out);
      for (const a of o.addAt) expect(J(C.addSlotAt(ch, slot, a.at))).toEqual(a.out);
      for (const a of o.toIdx) expect(J(C.moveSlotToIndex(ch, a.id, a.at))).toEqual(a.out);
      for (const a of o.remove) expect(J(C.removeSlot(ch, a.id))).toEqual(a.out);
      for (const a of o.move) expect(J(C.moveSlot(ch, a.id, a.by))).toEqual(a.out);
      for (const a of o.moveTo) expect(J(C.moveSlotTo(ch, a.id, a.to))).toEqual(a.out);
      for (const a of o.replace) expect(J(C.replaceSlot(ch, a.id, slot))).toEqual(a.out);
      for (const a of o.set) expect(J(C.setSlot(ch, a.id, a.change as never))).toEqual(a.out);
    }
  });
  it("params and sliders", () => {
    const chain = fx.ops[3]!.c as Chain;
    for (const c of fx.params) {
      const p = c.info as never;
      for (const a of c.clamp) expect(C.clampParam(p, a.v)).toBe(a.out);
      for (const a of c.slider) expect(J(C.toSlider(p, a.v))).toEqual(a.out);
      for (const a of c.from) expect(C.fromSlider(p, a.t)).toBe(a.out);
      for (const a of c.fmt) expect(C.formatParam(p, a.v)).toBe(a.out);
      for (const a of c.setp) expect(J(C.setParam(chain, "s0", p, a.v))).toEqual(a.out);
      for (const a of c.pv) expect(C.paramValue(a.slot as never, p)).toBe(a.out);
    }
  });
  it("presets", () => {
    expect(C.BUILTIN_PRESETS.map((p) => p.name)).toEqual(fx.flags.names);
    expect(C.BUILTIN_PRESETS.map((p) => C.addsOne(p))).toEqual(fx.flags.addsOne);
    expect(C.batchPresets(C.BUILTIN_PRESETS).map((p) => p.name)).toEqual(fx.flags.batch);
    const effects = ["altaudio.limiter", "altaudio.dynamics", "altaudio.parametric_eq", "altaudio.noise_reducer", "vst3:abc"];
    const all = new Set(effects.slice(0, 4).concat(C.BUILTIN_PRESETS.flatMap((p) => p.chain.slots.map((s) => s.effect))));
    const some = new Set(["altaudio.limiter", "altaudio.dynamics"]);
    for (const c of fx.apply) {
      const p = C.BUILTIN_PRESETS.find((x) => x.name === c.name)!;
      const known = c.kn === "all" ? all : some;
      const cur = (c.ci === 0 ? fx.ops[0]!.c : fx.ops[3]!.c) as Chain;
      expect(J(C.presetChain(p, known, counter()))).toEqual(c.chain);
      expect(J(C.applyPreset(cur, p, known, counter()))).toEqual(c.apply);
    }
    for (const c of fx.from) expect(J(C.presetFrom(c.name, c.chain as never))).toEqual(c.out);
    for (const c of fx.sanitized) expect(J(C.sanitizePresets(c.raw))).toEqual(c.out);
  });
});
