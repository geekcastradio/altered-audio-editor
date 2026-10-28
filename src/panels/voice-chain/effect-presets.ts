// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * Presets for one effect: its settings saved under a name, kept per effect type, so a Vox Strip
 * preset is offered only in the Vox Strip window. The Voice Chain presets (whole chains) are
 * separate and live in chain.ts. These are saved in the app's preferences and never leave the
 * computer.
 */

import type { EffectInfo } from "../../app/core";
import type { EffectChainDoc, EffectSlotDoc } from "../../stores/session";
import { clampParam, paramValue, setSlot } from "./chain";

/** One saved preset: every setting of the effect, and its own data when it is small. */
export interface EffectPreset {
  name: string;
  params: Record<string, number>;
  /** An effect's own data (the Convolution Reverb's impulse response file), kept when small. */
  data?: string;
}

/** Saved presets by effect id. */
export type EffectPresetBank = Record<string, EffectPreset[]>;

/** Larger own data (an Amp Model capture) stays out of a preset: preferences are not the place for it. */
export const MAX_PRESET_DATA = 16 * 1024;
/** Presets kept per effect. */
export const MAX_PRESETS_PER_EFFECT = 60;

const byName = (a: EffectPreset, b: EffectPreset) => a.name.localeCompare(b.name, undefined, { numeric: true, sensitivity: "base" });

/** A preset from a slot: all of the effect's settings as they are now. `dataLeftOut` is true when its data was too big to keep. */
export function effectPresetFrom(name: string, slot: EffectSlotDoc, info: EffectInfo): { preset: EffectPreset; dataLeftOut: boolean } {
  const params: Record<string, number> = {};
  for (const p of info.params) params[p.key] = paramValue(slot, p);
  const preset: EffectPreset = { name: name.trim().slice(0, 60), params };
  let dataLeftOut = false;
  if (slot.data) {
    if (slot.data.length <= MAX_PRESET_DATA) preset.data = slot.data;
    else dataLeftOut = true;
  }
  return { preset, dataLeftOut };
}

/** The bank with `preset` saved for `effect`, replacing one of the same name. */
export function bankWith(bank: EffectPresetBank, effect: string, preset: EffectPreset): EffectPresetBank {
  const list = [...(bank[effect] ?? []).filter((p) => p.name.toLowerCase() !== preset.name.toLowerCase()), preset].sort(byName).slice(0, MAX_PRESETS_PER_EFFECT);
  return { ...bank, [effect]: list };
}

/** The bank without the named preset. */
export function bankWithout(bank: EffectPresetBank, effect: string, name: string): EffectPresetBank {
  const list = (bank[effect] ?? []).filter((p) => p.name !== name);
  const { [effect]: _gone, ...rest } = bank;
  return list.length ? { ...rest, [effect]: list } : rest;
}

/**
 * The chain with the preset applied to one slot: each setting the effect has, snapped to its
 * range, and the preset's data when it kept some. Other slots, the slot's on/off state and its
 * sidechain are left as they are. Settings the preset lacks keep their value.
 */
export function applyEffectPreset(chain: EffectChainDoc, slotId: string, preset: EffectPreset, info: EffectInfo): EffectChainDoc {
  const slot = chain.slots.find((s) => s.id === slotId);
  if (!slot) return chain;
  const params = { ...slot.params };
  for (const p of info.params) {
    const v = preset.params[p.key];
    if (typeof v === "number" && Number.isFinite(v)) params[p.key] = clampParam(p, v);
  }
  return setSlot(chain, slotId, { params, preset: preset.name, ...(preset.data !== undefined ? { data: preset.data } : {}) });
}

/** Saved presets read back from preferences, keeping only what is well formed. */
export function sanitizeBank(raw: unknown): EffectPresetBank {
  const out: EffectPresetBank = {};
  if (!raw || typeof raw !== "object" || Array.isArray(raw)) return out;
  for (const [effect, list] of Object.entries(raw as Record<string, unknown>)) {
    if (!Array.isArray(list)) continue;
    const kept: EffectPreset[] = [];
    for (const r of list) {
      if (!r || typeof r !== "object") continue;
      const v = r as Record<string, unknown>;
      if (typeof v.name !== "string" || !v.name.trim() || !v.params || typeof v.params !== "object") continue;
      const params: Record<string, number> = {};
      for (const [k, n] of Object.entries(v.params as Record<string, unknown>)) if (typeof n === "number" && Number.isFinite(n)) params[k] = n;
      const preset: EffectPreset = { name: v.name.trim().slice(0, 60), params };
      if (typeof v.data === "string" && v.data.length <= MAX_PRESET_DATA) preset.data = v.data;
      if (!kept.some((p) => p.name.toLowerCase() === preset.name.toLowerCase())) kept.push(preset);
    }
    if (kept.length) out[effect] = kept.sort(byName).slice(0, MAX_PRESETS_PER_EFFECT);
  }
  return out;
}
