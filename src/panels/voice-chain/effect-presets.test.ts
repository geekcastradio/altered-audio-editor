// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import type { EffectInfo, EffectParamInfo } from "../../app/core";
import type { EffectChainDoc, EffectSlotDoc } from "../../stores/session";
import { applyEffectPreset, bankWith, bankWithout, BUILTIN_EFFECT_PRESETS, effectPresetFrom, MAX_PRESET_DATA, sanitizeBank } from "./effect-presets";

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

  describe("built-in presets", () => {
    // The real ranges from the Rust effect catalog (src-tauri/src/dsp/effects). Keep in step with it.
    const band = (n: number): Record<string, [number, number]> => ({ [`b${n}_freq`]: [20, 20000], [`b${n}_gain`]: [-24, 24], [`b${n}_q`]: [0.1, 12] });
    const curvePoints = (): Record<string, [number, number]> => {
      const r: Record<string, [number, number]> = {};
      for (let i = 1; i <= 8; i++) {
        r[`p${i}_in`] = [-120, 0];
        r[`p${i}_out`] = [-120, 12];
      }
      return r;
    };
    const ranges: Record<string, Record<string, [number, number]>> = {
      "altaudio.dynamics_curve": {
        points: [2, 8], knee: [0, 24], makeup: [-24, 24], detect_attack: [0.1, 200], detect_release: [5, 2000], rms: [0, 1],
        gain_attack: [0.1, 200], gain_release: [5, 2000], lookahead: [0, 20], ...curvePoints(),
      },
      "altaudio.plosive": { freq: [60, 250], trigger: [0, 24], reduction: [0, 30], release: [20, 400] },
      "altaudio.dynamics": { threshold: [-60, 0], ratio: [1, 20], attack: [0.1, 200], release: [5, 2000], knee: [0, 24], makeup: [0, 24] },
      "altaudio.limiter": { ceiling: [-24, 0], input_boost: [0, 24], lookahead: [0, 20], release: [10, 1000], linked: [0, 1] },
      "altaudio.breath_gate": { threshold: [-80, 0], attack: [0.1, 100], hold: [0, 1000], release: [5, 2000] },
      "altaudio.noise_reducer": { reduction: [0, 40] },
      "altaudio.precision_filter": { design: [0, 3], mode: [0, 3], order: [0, 3], freq: [10, 20000], freq2: [20, 20000] },
      "altaudio.expander": { threshold: [-80, 0], ratio: [1, 20] },
      "altaudio.click_repair": { sensitivity: [0, 100], width: [0.1, 3] },
      "altaudio.speech_leveler": { target: [-40, -6], amount: [0, 100], range: [0, 60], boost_low: [0, 1], comp_on: [0, 1], comp_thr: [-9, -1], gate_on: [0, 1], gate_off: [0, 25] },
      "altaudio.parametric_eq": {
        hp_on: [0, 1], hp_freq: [10, 2000], hp_slope: [0, 2], ls_freq: [20, 2000], ls_gain: [-24, 24],
        ...band(1), ...band(2), ...band(3),
        hs_freq: [1000, 20000], hs_gain: [-24, 24], lp_on: [0, 1], lp_freq: [1000, 22000], lp_slope: [0, 2],
      },
    };

    it("keeps every value inside the effect's real range, with no stray keys", () => {
      for (const [effect, presets] of Object.entries(BUILTIN_EFFECT_PRESETS)) {
        const r = ranges[effect];
        expect(r, `no range table for ${effect}`).toBeDefined();
        for (const pr of presets) {
          for (const [k, v] of Object.entries(pr.params)) {
            const range = r![k];
            expect(range, `${effect} / ${pr.name}: unknown key ${k}`).toBeDefined();
            expect(v, `${effect} / ${pr.name} / ${k}`).toBeGreaterThanOrEqual(range![0]);
            expect(v, `${effect} / ${pr.name} / ${k}`).toBeLessThanOrEqual(range![1]);
          }
        }
      }
    });

    it("names each preset once per effect", () => {
      for (const [effect, presets] of Object.entries(BUILTIN_EFFECT_PRESETS)) {
        const names = presets.map((p) => p.name.toLowerCase());
        expect(new Set(names).size, effect).toBe(names.length);
        expect(names.every((n) => n.trim().length > 0 && n.length <= 60), effect).toBe(true);
      }
    });

    it("lists every setting of a Parametric EQ preset, so one fully replaces another", () => {
      const keys = Object.keys(ranges["altaudio.parametric_eq"]!).sort();
      for (const pr of BUILTIN_EFFECT_PRESETS["altaudio.parametric_eq"]!) expect(Object.keys(pr.params).sort(), pr.name).toEqual(keys);
    });
  });
});
