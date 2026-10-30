// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import type { EffectChainDoc } from "../../stores/session";
import { withLooks } from "./audiofx";
import { isCustom, makeSlots, presetSetup, readSetup, sectionEffects, TYPES, withoutSetup, withSetup } from "./soundtypes";

const empty: EffectChainDoc = { enabled: true, input_gain_db: 0, output_gain_db: 0, mix: 1, slots: [] };
let n = 0;
const id = () => `s${n++}`;

describe("Sound Types", () => {
  it("every section of every type makes at least one effect", () => {
    for (const t of TYPES) for (const s of t.sections) expect(sectionEffects(t.id, s.id, 50).length, `${t.id}/${s.id}`).toBeGreaterThan(0);
  });

  it("every preset only names sections its type has", () => {
    for (const t of TYPES) for (const p of t.presets) for (const s of p.sections) expect(t.sections.some((x) => x.id === s), `${t.id}/${p.id}/${s}`).toBe(true);
  });

  it("reads back what it wrote", () => {
    const setup = presetSetup("music", "bed");
    const chain = withSetup(empty, setup, id)!;
    expect(readSetup(chain)).toEqual({ type: "music", sections: setup.sections, preset: "bed", amount: 60 });
    expect(isCustom(chain, readSetup(chain))).toBe(false);
  });

  it("shows Custom once an effect is changed elsewhere", () => {
    const chain = withSetup(empty, presetSetup("dialogue", "clean"), id)!;
    const edited = { ...chain, slots: chain.slots.map((s, i) => (i === 0 ? { ...s, params: { ...s.params, reduction: 1 } } : s)) };
    expect(isCustom(edited, readSetup(edited))).toBe(true);
  });

  it("keeps other effects after its own and replaces itself on re-apply", () => {
    const other = { id: "x", effect: "altaudio.echo", enabled: true, params: {}, preset: null };
    const once = withSetup({ ...empty, slots: [other] }, presetSetup("sfx", "hit"), id)!;
    const twice = withSetup(once, presetSetup("sfx", "soft"), id)!;
    expect(twice.slots.at(-1)).toEqual(other);
    expect(readSetup(twice)?.preset).toBe("soft");
    expect(withoutSetup(twice).slots).toEqual([other]);
  });

  it("replaces the Audio FX looks instead of stacking on them, and the other way round", () => {
    const looks = withLooks(empty, { restoration: 2, level: 0, clarity: 0, polish: 0 }, id)!;
    const sound = withSetup(looks, presetSetup("dialogue", "clean"), id)!;
    expect(sound.slots.every((s) => s.preset?.startsWith("Sound Types: "))).toBe(true);
    const back = withLooks(sound, { restoration: 1, level: 0, clarity: 0, polish: 0 }, id)!;
    expect(back.slots.every((s) => s.preset?.startsWith("Audio FX: "))).toBe(true);
  });

  it("refuses to overflow the chain", () => {
    const full = { ...empty, slots: Array.from({ length: 16 }, (_, i) => ({ id: `f${i}`, effect: "altaudio.echo", enabled: true, params: {}, preset: null })) };
    expect(withSetup(full, presetSetup("dialogue", "clean"), id)).toBeNull();
  });
});
