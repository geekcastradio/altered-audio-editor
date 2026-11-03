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
import { DYNAMICS_CURVE_PRESETS } from "./curve-presets";

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

/**
 * Presets that ship with an effect, listed apart from the user's own and never edited or
 * deleted. Named for the job, not for any other product's presets.
 */
/** A compressor preset: threshold and ratio carry the idea, attack and release follow speech-friendly defaults. */
const comp = (name: string, threshold: number, ratio: number, makeup = 0, knee = 6, attack = 5, release = 150): EffectPreset => ({
  name,
  params: { threshold, ratio, attack, release, knee, makeup },
});

/** A limiter preset. Input boost tops out at 24 dB here, so the extreme ones sit at the top of the range. */
const lim = (name: string, ceiling: number, input_boost: number, lookahead: number, release: number): EffectPreset => ({
  name,
  params: { ceiling, input_boost, lookahead, release, linked: 1 },
});

/** A gate preset (Breath Gate): threshold, attack, hold and release; the closed level keeps its value. */
const gate = (name: string, threshold: number, attack: number, hold: number, release: number): EffectPreset => ({
  name,
  params: { threshold, attack, hold, release },
});

/** A Speech Leveler preset: the five settings that matter, the rest keep their values. */
const lev = (name: string, target: number, amount: number, range: number, comp: number | null, gate: number | null): EffectPreset => ({
  name,
  params: {
    target,
    amount,
    range,
    boost_low: 0,
    comp_on: comp === null ? 0 : 1,
    comp_thr: comp ?? -6,
    gate_on: gate === null ? 0 : 1,
    gate_off: gate ?? 15,
  },
});

/** A Parametric EQ preset. Every setting is listed, so applying one fully replaces what was there; bands left out are flat and the filters off. */
const eq = (name: string, p: Record<string, number>): EffectPreset => ({
  name,
  params: {
    hp_on: 0, hp_freq: 80, hp_slope: 1,
    ls_freq: 120, ls_gain: 0,
    b1_freq: 250, b1_gain: 0, b1_q: 1,
    b2_freq: 1000, b2_gain: 0, b2_q: 1,
    b3_freq: 4000, b3_gain: 0, b3_q: 1,
    hs_freq: 8000, hs_gain: 0,
    lp_on: 0, lp_freq: 18000, lp_slope: 1,
    ...p,
  },
});

export const BUILTIN_EFFECT_PRESETS: Readonly<Record<string, readonly EffectPreset[]>> = {
  "altaudio.dynamics": [
    { name: "Voice, Gentle", params: { threshold: -18, ratio: 2, attack: 12, release: 180, knee: 8, makeup: 2 } },
    { name: "Voice, Firm", params: { threshold: -24, ratio: 4, attack: 8, release: 160, knee: 6, makeup: 5 } },
    { name: "Broadcast Squash", params: { threshold: -28, ratio: 6, attack: 4, release: 120, knee: 4, makeup: 8 } },
    { name: "Peak Catcher", params: { threshold: -10, ratio: 12, attack: 1, release: 80, knee: 2, makeup: 0 } },
    { name: "Music Glue", params: { threshold: -16, ratio: 2, attack: 30, release: 250, knee: 10, makeup: 1.5 } },
    { name: "Drum Punch", params: { threshold: -20, ratio: 4, attack: 25, release: 100, knee: 3, makeup: 3 } },
    comp("Sibilance, Light", -24, 1.5, 0, 6, 1, 120),
    comp("Sibilance, Medium", -30, 2, 0, 6, 1, 120),
    comp("Sibilance, Hard", -35, 4, 0, 4, 1, 120),
    comp("Vocal, Even", -20, 2),
    comp("Vocal, Driven", -24, 3.5, 2),
    comp("Vocal, Peak Guard", -8, 6, 0, 2, 1),
    comp("Narration, Even", -15, 2.5),
    comp("Ceiling, -12 dB", -12, 20, 0, 0, 0.5, 100),
    comp("Ceiling, -24 dB", -24, 20, 0, 0, 0.5, 100),
    comp("Bass, Dense", -30, 6, 12, 6, 10, 200),
    comp("Mix Bus, Gentle", -10, 2, 2, 8, 20, 250),
    comp("Leveling, Strong", -48.7, 3, 19.1, 6, 0.7, 50),
    comp("Leveling, Medium", -48.2, 2.2, 11.7, 6, 1, 50),
    comp("Leveling, Soft", -12.3, 1.9, 3.3, 6, 1, 50),
  ],
  "altaudio.dynamics_curve": DYNAMICS_CURVE_PRESETS,
  "altaudio.breath_gate": [
    gate("Gate, Quiet Room", -20, 2, 50, 200),
    gate("Gate, Spoken Word", -14, 2, 50, 200),
    gate("Gate, Room Noise Only", -23.1, 4.3, 1, 87),
    gate("Gate, Hard Close", -11.2, 1, 1, 87),
  ],
  "altaudio.expander": [
    { name: "Quiet Parts Down, Gentle", params: { threshold: -20, ratio: 2.5 } },
    { name: "Quiet Parts Down, Firm", params: { threshold: -20, ratio: 5 } },
  ],
  "altaudio.noise_reducer": [
    // Reduction follows the strengths in the readouts (20, 40 and 80 percent) at 0.3 dB per percent. Check by ear.
    { name: "Noise, Light", params: { reduction: 6 } },
    { name: "Noise, Medium", params: { reduction: 12 } },
    { name: "Noise, Heavy", params: { reduction: 24 } },
  ],
  "altaudio.precision_filter": [
    // design 0 is Butterworth; mode 0 low-pass, 1 high-pass, 2 band-pass, 3 band-stop; order 0 to 3 is 12 to 48 dB per octave.
    { name: "Notch Out 60 Hz Hum", params: { design: 0, mode: 3, order: 1, freq: 58, freq2: 62 } },
    { name: "Low Cut, 100 Hz Steep", params: { design: 0, mode: 1, order: 3, freq: 100 } },
    { name: "Low Cut, 250 Hz", params: { design: 0, mode: 1, order: 1, freq: 250 } },
    { name: "Low Cut, Subsonic", params: { design: 0, mode: 1, order: 1, freq: 25 } },
    { name: "High Cut, 10 kHz", params: { design: 0, mode: 0, order: 1, freq: 10000 } },
    { name: "Bass Only, 10 to 400 Hz", params: { design: 0, mode: 2, order: 1, freq: 10, freq2: 400 } },
    { name: "Narrow Band, Around 1 kHz", params: { design: 0, mode: 2, order: 1, freq: 900, freq2: 1100 } },
  ],
  "altaudio.click_repair": [
    // Estimated from another editor's three strengths, so check them by ear.
    { name: "Clicks, Light", params: { sensitivity: 25, width: 0.6 } },
    { name: "Clicks, Medium", params: { sensitivity: 50, width: 1.2 } },
    { name: "Clicks, Heavy", params: { sensitivity: 85, width: 2.5 } },
  ],
  "altaudio.limiter": [
    lim("Stop at -0.1 dB", -0.1, 0, 7, 100),
    lim("Lift, Stop at -0.1 dB", -0.1, 6, 7, 100),
    lim("Lift, Stop at -3 dB", -3, 6, 7, 100),
    lim("Lift, Stop at -6 dB", -6, 3, 7, 100),
    lim("Squeeze, Light", -3, 3, 7, 100),
    lim("Squeeze, Medium", -2, 6, 7, 100),
    lim("Squeeze, Heavy", -1, 15, 7, 100),
    lim("Squeeze, Crushed", -1, 24, 5, 40),
    lim("Wall of Loud", -1, 24, 10, 40),
    lim("Streaming Ceiling, -1 dB", -1, 0, 12, 200),
    lim("Limit, Soft Touch", -11.6, 0, 5, 74),
    lim("Limit, Tight Ceiling", -24, 0, 5, 173),
  ],
  "altaudio.speech_leveler": [
    lev("Even Out, Full", -12, 100, 45, null, null),
    lev("Even Out, Light Touch", -18, 50, 60, -1, null),
    lev("Even Out, Balanced", -18, 80, 45, -3, 15),
    lev("Even Out, Soft", -18, 50, 45, -1, 15),
    lev("Even Out, Tight", -18, 100, 45, -9, 15),
  ],
  "altaudio.parametric_eq": [
    eq("Flat", {}),
    eq("Voice, Clean and Present", { hp_on: 1, hp_freq: 80, ls_freq: 110, ls_gain: 3, b1_freq: 290, b1_gain: -3, b1_q: 1.5, b2_freq: 3200, b2_gain: 2, b2_q: 2, hs_freq: 12000, hs_gain: 3 }),
    eq("Voice, Forward", { hp_on: 1, hp_freq: 80, ls_freq: 110, ls_gain: 4, b1_freq: 290, b1_gain: -3, b1_q: 1.5, b2_freq: 1440, b2_gain: 5, b2_q: 2, hs_freq: 12000, hs_gain: 4 }),
    eq("Voice, Mud Cut", { hp_on: 1, hp_freq: 70, hp_slope: 0, b1_freq: 280, b1_gain: -4, b1_q: 1.2 }),
    eq("Cut Rumble", { hp_on: 1, hp_freq: 80, hp_slope: 1 }),
    eq("Cut Hiss", { lp_on: 1, lp_freq: 9000, lp_slope: 1 }),
    eq("Telephone Band", { hp_on: 1, hp_freq: 300, lp_on: 1, lp_freq: 3400 }),
    eq("Vintage Broadcast", { hp_on: 1, hp_freq: 300, hp_slope: 2, b1_freq: 700, b1_gain: 9, b1_q: 2, b2_freq: 2000, b2_gain: -8, b2_q: 2, lp_on: 1, lp_freq: 4000 }),
    eq("Loudness Lift", { ls_freq: 120, ls_gain: 3, hs_freq: 6000, hs_gain: 6 }),
    eq("Acoustic Guitar, Open", { ls_freq: 60, ls_gain: -6, b1_freq: 200, b1_gain: -3, b2_freq: 2000, b2_gain: -2, b3_freq: 5000, b3_gain: 2, hs_freq: 15000, hs_gain: 2 }),
    eq("Guitar, Tight and Heavy", { ls_freq: 40, ls_gain: -12, b1_freq: 120, b1_gain: -3, b2_freq: 450, b2_gain: 4, b2_q: 4, b3_freq: 2900, b3_gain: -3, hs_freq: 18000, hs_gain: -12 }),
    eq("Kick, Thump and Click", { b1_freq: 80, b1_gain: 3, b1_q: 2, b2_freq: 300, b2_gain: -3, b2_q: 3, b3_freq: 2500, b3_gain: 4, b3_q: 2, hs_freq: 18000, hs_gain: -12 }),
    eq("Snare, Fat Body", { ls_freq: 30, ls_gain: -12, b2_freq: 270, b2_gain: 6, b2_q: 3, b3_freq: 5700, b3_gain: -6, hs_freq: 16000, hs_gain: -6 }),
  ],
};

export const builtinPresets = (effect: string): readonly EffectPreset[] => BUILTIN_EFFECT_PRESETS[effect] ?? [];
