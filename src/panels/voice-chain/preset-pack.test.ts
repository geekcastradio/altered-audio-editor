// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import type { ChainPreset } from "./chain";
import { MAX_PRESETS_PER_EFFECT, type EffectPresetBank } from "./effect-presets";
import { mergePack, packPresets, readPack, uniqueName } from "./preset-pack";

const chain = (name: string, gain = 0): ChainPreset => ({
  name,
  group: "Voice chains",
  chain: { enabled: true, input_gain_db: gain, output_gain_db: 0, mix: 1, slots: [{ effect: "altaudio.limiter", params: { ceiling: -3 } }] },
});
const bank: EffectPresetBank = { "altaudio.limiter": [{ name: "Loud", params: { ceiling: -1 } }] };

describe("preset packs", () => {
  it("round trip", () => {
    const back = readPack(packPresets([chain("A")], bank));
    expect("error" in back).toBe(false);
    if ("error" in back) return;
    expect(back.chains.map((c) => c.name)).toEqual(["A"]);
    expect(back.bank).toEqual(bank);
  });

  it("refuse other files and newer packs", () => {
    expect(readPack("nope")).toHaveProperty("error");
    expect(readPack(JSON.stringify({ format: "other", version: 1 }))).toHaveProperty("error");
    expect(readPack(JSON.stringify({ format: "altaudio.presets", version: 99 }))).toHaveProperty("error");
  });

  it("drop malformed entries instead of failing", () => {
    const r = readPack(JSON.stringify({ format: "altaudio.presets", version: 1, chains: [5, { name: "x" }], effects: { a: "bad" } }));
    expect(r).toEqual({ chains: [], bank: {} });
  });

  it("number a taken name", () => {
    expect(uniqueName("A", (n) => n === "A")).toBe("A (2)");
    expect(uniqueName("A", (n) => n === "A" || n === "A (2)")).toBe("A (3)");
    const long = "x".repeat(60);
    const got = uniqueName(long, (n) => n === long);
    expect(got).toHaveLength(60);
    expect(got.endsWith(" (2)")).toBe(true);
  });

  it("never overwrite: same name with other settings comes in renamed", () => {
    const m = mergePack([chain("A", 0)], {}, { chains: [chain("A", 3)], bank: {} }, []);
    expect(m.chains.map((c) => c.name)).toEqual(["A", "A (2)"]);
    expect(m.chains[0]!.chain.input_gain_db).toBe(0);
    expect(m.renamed).toBe(1);
  });

  it("skip an identical preset and keep clear of built-in names", () => {
    const m = mergePack([chain("A")], bank, { chains: [chain("A"), chain("Podcast Voice")], bank }, ["Podcast Voice"]);
    expect(m.chains.map((c) => c.name)).toEqual(["A", "Podcast Voice (2)"]);
    expect(m.addedChains).toBe(1);
    expect(m.addedEffects).toBe(0);
  });

  it("add single-effect presets and rename clashes", () => {
    const m = mergePack([], bank, { chains: [], bank: { "altaudio.limiter": [{ name: "loud", params: { ceiling: -9 } }, { name: "Soft", params: { ceiling: -6 } }] } }, []);
    expect(m.bank["altaudio.limiter"]!.map((p) => p.name)).toEqual(["Loud", "loud (2)", "Soft"]);
    expect(m.addedEffects).toBe(2);
  });

  it("stop at the per-effect limit", () => {
    const many = Array.from({ length: MAX_PRESETS_PER_EFFECT }, (_, i) => ({ name: `p${i}`, params: { a: i } }));
    const m = mergePack([], { e: many }, { chains: [], bank: { e: [{ name: "extra", params: {} }] } }, []);
    expect(m.bank.e).toHaveLength(MAX_PRESETS_PER_EFFECT);
    expect(m.full).toBe(1);
  });
});
