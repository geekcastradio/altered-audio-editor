// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import type { EffectInfo, EffectParamInfo } from "../../app/core";
import type { EffectChainDoc, EffectSlotDoc } from "../../stores/session";
import { applyEffectPreset, bankWith, bankWithout, effectPresetFrom, MAX_PRESET_DATA, sanitizeBank } from "./effect-presets";

const param = (key: string, over: Partial<EffectParamInfo> = {}): EffectParamInfo => ({ key, name: key, unit: "", min: 0, max: 100, default: 50, step: 1, log: false, toggle: false, ...over });
const info: EffectInfo = { id: "altaudio.test", name: "Test", category: "Dynamics", description: "", params: [param("a"), param("b", { min: -12, max: 12, default: 0, step: 0.5 })] };
const slot = (over: Partial<EffectSlotDoc> = {}): EffectSlotDoc => ({ id: "s1", effect: "altaudio.test", enabled: true, params: {}, preset: null, ...over });
const chain = (slots: EffectSlotDoc[]): EffectChainDoc => ({ enabled: true, input_gain_db: 0, output_gain_db: 0, mix: 1, slots });

describe("Effect presets", () => {
  it("saves every setting, using the default for ones never moved", () => {
    const { preset } = effectPresetFrom("  Warm  ", slot({ params: { b: 3 } }), info);
    expect(preset).toEqual({ name: "Warm", params: { a: 50, b: 3 } });
  });

  it("keeps small own data and leaves out large data", () => {
    const small = effectPresetFrom("x", slot({ data: '{"path":"a.wav"}' }), info);
    expect(small.preset.data).toBe('{"path":"a.wav"}');
    expect(small.dataLeftOut).toBe(false);
    const big = effectPresetFrom("x", slot({ data: "x".repeat(MAX_PRESET_DATA + 1) }), info);
    expect(big.preset.data).toBeUndefined();
    expect(big.dataLeftOut).toBe(true);
  });

  it("replaces a preset of the same name and keeps the list sorted", () => {
    let bank = bankWith({}, "e", { name: "b", params: { a: 1 } });
    bank = bankWith(bank, "e", { name: "A", params: { a: 2 } });
    bank = bankWith(bank, "e", { name: "B", params: { a: 3 } });
    expect(bank["e"]!.map((p) => [p.name, p.params["a"]])).toEqual([["A", 2], ["B", 3]]);
  });

  it("deletes one preset and drops an empty list", () => {
    const bank = bankWith(bankWith({}, "e", { name: "a", params: {} }), "f", { name: "z", params: {} });
    const less = bankWithout(bank, "e", "a");
    expect(less).toEqual({ f: [{ name: "z", params: {} }] });
    expect(bankWithout(bank, "e", "nope")).toEqual(bank);
  });

  it("applies settings inside their ranges to the one slot only", () => {
    const c = chain([slot(), slot({ id: "s2", params: { a: 9 } })]);
    const next = applyEffectPreset(c, "s1", { name: "Hot", params: { a: 500, b: 3.3, unknown: 5 } }, info);
    expect(next.slots[0]!.params).toEqual({ a: 100, b: 3.5 });
    expect(next.slots[0]!.preset).toBe("Hot");
    expect(next.slots[1]).toBe(c.slots[1]);
  });

  it("leaves on/off, sidechain and data alone when the preset has no data", () => {
    const c = chain([slot({ enabled: false, sidechain: "t2", data: "keep" })]);
    const next = applyEffectPreset(c, "s1", { name: "P", params: { a: 10 } }, info).slots[0]!;
    expect(next.enabled).toBe(false);
    expect(next.sidechain).toBe("t2");
    expect(next.data).toBe("keep");
    expect(applyEffectPreset(c, "s1", { name: "P", params: {}, data: "new" }, info).slots[0]!.data).toBe("new");
  });

  it("settings the preset lacks keep their value", () => {
    const next = applyEffectPreset(chain([slot({ params: { b: 4 } })]), "s1", { name: "P", params: { a: 10 } }, info).slots[0]!;
    expect(next.params).toEqual({ a: 10, b: 4 });
  });

  it("reads back only what is well formed", () => {
    const bank = sanitizeBank({
      e: [{ name: "Ok", params: { a: 1, bad: "x" } }, { name: "", params: {} }, { params: {} }, null, { name: "ok", params: {} }],
      f: "nope",
      g: [],
    });
    expect(bank).toEqual({ e: [{ name: "Ok", params: { a: 1 } }] });
    expect(sanitizeBank(null)).toEqual({});
    expect(sanitizeBank([1])).toEqual({});
  });
});
