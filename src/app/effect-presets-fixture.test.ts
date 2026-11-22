// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Checks the TypeScript effect preset rules against the same fixture the Rust test reads
// (shared-fixtures/effect-presets.json).
import { describe, expect, it } from "vitest";
import fx from "../../shared-fixtures/effect-presets.json";
import * as E from "../panels/voice-chain/effect-presets";
import type { EffectInfo } from "./core";
import type { EffectChainDoc, EffectSlotDoc } from "../stores/session";

const info = { id: "x", name: "X", category: "c", description: "", params: fx.info } as unknown as EffectInfo;
const slotOf = (s: unknown) => s as EffectSlotDoc;
const chain = (slots: EffectSlotDoc[]): EffectChainDoc => ({ enabled: true, input_gain_db: 0, output_gain_db: 0, mix: 1, slots });

describe("effect presets fixture", () => {
  it("presets from slots", () => {
    for (const c of fx.from) {
      const s = slotOf({ id: "s1", effect: "x", enabled: true, params: c.params, preset: null, ...(c.data === null ? {} : { data: c.data }) });
      expect(JSON.parse(JSON.stringify(E.effectPresetFrom(c.name as string, s, info)))).toEqual(c.out);
    }
  });
  it("the bank", () => {
    let bank: E.EffectPresetBank = {};
    const steps = fx.bank as { op: string; effect: string; preset?: E.EffectPreset; name?: string; out: unknown }[];
    steps.slice(0, 11).forEach((s) => {
      bank = s.op === "with" ? E.bankWith(bank, s.effect, s.preset!) : E.bankWithout(bank, s.effect, s.name!);
      expect(JSON.parse(JSON.stringify(bank))).toEqual(s.out);
    });
    let long = bank;
    for (let i = 0; i < 64; i++) long = E.bankWith(long, "m", { name: `Preset ${(i * 7) % 64}`, params: { a: i } });
    expect(JSON.parse(JSON.stringify(long))).toEqual(steps[steps.length - 1]!.out);
  });
  it("applying a preset", () => {
    for (const c of fx.apply) {
      const s = slotOf(c.slot);
      const other = slotOf({ id: "s2", effect: "x", enabled: true, params: {}, preset: null });
      const next = E.applyEffectPreset(chain([s, other]), "s1", c.preset as E.EffectPreset, info).slots[0]!;
      expect({ params: next.params, preset: next.preset, data: next.data ?? null }).toEqual(c.out);
    }
  });
  it("reading a bank back", () => {
    for (const c of fx.sanitize) expect(JSON.parse(JSON.stringify(E.sanitizeBank(c.raw)))).toEqual(c.out);
  });
});
