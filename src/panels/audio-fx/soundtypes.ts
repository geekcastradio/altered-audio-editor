// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * Sound Types: pick what a clip or track is (dialogue, music, SFX, ambience), tick the
 * jobs it needs, choose a starting preset and one Amount. Like Audio FX it is a shortcut,
 * never a second engine: it writes ordinary Voice Chain effects, tagged so the panel can
 * read them back. Sync runs one way. If the effects are changed in the Voice Chain the
 * panel shows the preset as Custom and stops claiming they are its own settings.
 */

import type { EffectChainDoc, EffectSlotDoc } from "../../stores/session";
import { BLOCKS } from "../voice-chain/chain";
import { isPanelSlot as isLookSlot } from "./audiofx";

export type SoundType = "dialogue" | "music" | "sfx" | "ambience";

export interface SectionInfo {
  id: string;
  label: string;
  about: string;
}

export interface PresetInfo {
  id: string;
  label: string;
  /** Sections ticked, and the Amount (0 to 100), when the preset is picked. */
  sections: readonly string[];
  amount: number;
}

export interface TypeInfo {
  id: SoundType;
  label: string;
  about: string;
  sections: readonly SectionInfo[];
  presets: readonly PresetInfo[];
}

export const TYPES: readonly TypeInfo[] = [
  {
    id: "dialogue",
    label: "Dialogue",
    about: "Voices: hosts, guests and reads.",
    sections: [
      { id: "repair", label: "Repair", about: "Turns down steady background noise such as hiss, fans and room tone." },
      { id: "level", label: "Level", about: "Evens out quiet and loud moments." },
      { id: "clarity", label: "Clarity", about: "Rolls off rumble, clears mud, adds presence, and keeps S sounds soft." },
      { id: "limit", label: "Limit", about: "Stops peaks from going over a ceiling." },
    ],
    presets: [
      { id: "clean", label: "Clean Voice", sections: ["repair", "level", "clarity", "limit"], amount: 50 },
      { id: "noisy", label: "Noisy Room", sections: ["repair", "level", "clarity", "limit"], amount: 85 },
      { id: "remote", label: "Remote Guest", sections: ["repair", "level", "clarity"], amount: 65 },
      { id: "light", label: "Light Touch", sections: ["level", "limit"], amount: 30 },
    ],
  },
  {
    id: "music",
    label: "Music",
    about: "Beds, themes and stingers.",
    sections: [
      { id: "tone", label: "Tone", about: "Trims low rumble and a little midrange mud." },
      { id: "room", label: "Make Room", about: "Dips the middle of the music so a voice sits on top." },
      { id: "glue", label: "Glue", about: "Light compression to hold the mix together." },
      { id: "limit", label: "Limit", about: "Stops peaks from going over a ceiling." },
    ],
    presets: [
      { id: "bed", label: "Under Voice", sections: ["tone", "room", "glue", "limit"], amount: 60 },
      { id: "theme", label: "Theme Up Front", sections: ["tone", "glue", "limit"], amount: 40 },
      { id: "safe", label: "Just Safe", sections: ["limit"], amount: 40 },
    ],
  },
  {
    id: "sfx",
    label: "SFX",
    about: "Hits, whooshes and one-shot sounds.",
    sections: [
      { id: "tidy", label: "Tidy", about: "Cuts low rumble that adds nothing to a short sound." },
      { id: "punch", label: "Punch", about: "Firm compression so hits stay present." },
      { id: "limit", label: "Limit", about: "Stops peaks from going over a ceiling." },
    ],
    presets: [
      { id: "hit", label: "Punchy Hit", sections: ["tidy", "punch", "limit"], amount: 60 },
      { id: "soft", label: "Soft Cue", sections: ["tidy", "limit"], amount: 30 },
    ],
  },
  {
    id: "ambience",
    label: "Ambience",
    about: "Room tone, crowds and atmosphere.",
    sections: [
      { id: "hiss", label: "Hiss", about: "Turns down steady hiss." },
      { id: "soften", label: "Soften", about: "Rolls off low rumble and the harshest highs so it sits back." },
      { id: "level", label: "Level", about: "Gentle compression to keep it steady." },
    ],
    presets: [
      { id: "room", label: "Room Tone", sections: ["hiss", "soften", "level"], amount: 50 },
      { id: "crowd", label: "Crowd Bed", sections: ["soften", "level"], amount: 40 },
    ],
  },
];

export const typeInfo = (id: SoundType): TypeInfo => TYPES.find((t) => t.id === id)!;

/** What a slot's preset name starts with when this panel made it. */
export const TAG = "Sound Types: ";
export const isSoundSlot = (s: EffectSlotDoc): boolean => typeof s.preset === "string" && s.preset.startsWith(TAG);

export interface Setup {
  type: SoundType;
  sections: readonly string[];
  preset: string;
  amount: number;
}

const lerp = (a: number, b: number, t: number) => a + (b - a) * t;
const round = (n: number, d = 1) => Math.round(n * 10 ** d) / 10 ** d;

type Made = { effect: string; params: Record<string, number> };

/** The plain effects one section makes at an Amount of 0 to 100. */
export function sectionEffects(type: SoundType, section: string, amount: number): Made[] {
  const { fx, voiceEq, leveler, comp, limit, cleanNoise } = BLOCKS;
  const a = Math.min(100, Math.max(0, amount)) / 100;
  const key = `${type}/${section}`;
  switch (key) {
    case "dialogue/repair":
      return [cleanNoise(round(lerp(4, 14, a), 0))];
    case "dialogue/level":
      return [leveler, comp(round(lerp(-16, -26, a), 0), round(lerp(2, 4, a)), round(lerp(2, 6, a), 0))];
    case "dialogue/clarity":
      return [
        { ...voiceEq, params: { ...voiceEq.params, b3_gain: round(lerp(1, 3.5, a)) } },
        fx("deesser", { freq: 6500, threshold: round(lerp(-28, -34, a), 0), reduction: round(lerp(4, 8, a), 0) }),
      ];
    case "music/tone":
      return [fx("parametric_eq", { hp_on: 1, hp_freq: 35, hp_slope: 1, b1_freq: 300, b1_gain: -round(lerp(0.5, 2.5, a)), b1_q: 1 })];
    case "music/room":
      return [fx("parametric_eq", { b2_freq: 1800, b2_gain: -round(lerp(1, 5, a)), b2_q: 0.7 })];
    case "music/glue":
      return [comp(round(lerp(-12, -22, a), 0), round(lerp(1.5, 2.5, a)), round(lerp(0, 3, a), 0))];
    case "sfx/tidy":
      return [fx("parametric_eq", { hp_on: 1, hp_freq: round(lerp(40, 90, a), 0), hp_slope: 1 })];
    case "sfx/punch":
      return [comp(round(lerp(-14, -24, a), 0), round(lerp(2, 4, a)), round(lerp(1, 4, a), 0))];
    case "ambience/hiss":
      return [cleanNoise(round(lerp(3, 10, a), 0))];
    case "ambience/soften":
      return [fx("parametric_eq", { hp_on: 1, hp_freq: round(lerp(60, 140, a), 0), hp_slope: 1, lp_on: 1, lp_freq: round(lerp(14000, 8000, a), 0), lp_slope: 1 })];
    case "ambience/level":
      return [comp(round(lerp(-18, -28, a), 0), round(lerp(1.5, 3, a)), 0)];
    default:
      // Every type ends in Limit, with a ceiling that gets closer to 0 as Amount rises.
      return section === "limit" ? [limit(round(lerp(-6, -1.5, a)))] : [];
  }
}

const tagOf = (type: SoundType, section: string, preset: string, amount: number) => `${TAG}${type}/${section}/${preset}/${Math.round(amount)}`;

/** The slots a setup would make, in chain order. */
export function makeSlots(setup: Setup, newId: () => string): EffectSlotDoc[] {
  const info = typeInfo(setup.type);
  const out: EffectSlotDoc[] = [];
  for (const sec of info.sections) {
    if (!setup.sections.includes(sec.id)) continue;
    for (const e of sectionEffects(setup.type, sec.id, setup.amount)) {
      out.push({ id: newId(), effect: e.effect, enabled: true, params: { ...e.params }, preset: tagOf(setup.type, sec.id, setup.preset, setup.amount) });
    }
  }
  return out;
}

/** The setup recorded on the chain's tagged slots, or null when there are none. */
export function readSetup(chain: EffectChainDoc | null | undefined): Setup | null {
  const tagged = (chain?.slots ?? []).filter(isSoundSlot);
  let type: SoundType | null = null;
  let preset = "";
  let amount = 50;
  const sections: string[] = [];
  for (const s of tagged) {
    const m = /^Sound Types: (\w+)\/(\w+)\/([\w-]*)\/(\d+)$/.exec(s.preset!);
    if (!m || !TYPES.some((t) => t.id === m[1])) continue;
    type = m[1] as SoundType;
    if (!sections.includes(m[2]!)) sections.push(m[2]!);
    preset = m[3]!;
    amount = Number(m[4]);
  }
  return type ? { type, sections, preset, amount } : null;
}

const sameParams = (a: Record<string, number>, b: Record<string, number>) => {
  const keys = new Set([...Object.keys(a), ...Object.keys(b)]);
  for (const k of keys) if (Math.abs((a[k] ?? 0) - (b[k] ?? 0)) > 1e-6) return false;
  return true;
};

/** True when the tagged effects no longer match what the setup would make (changed in the Voice Chain). */
export function isCustom(chain: EffectChainDoc | null | undefined, setup: Setup | null): boolean {
  if (!setup) return false;
  const have = (chain?.slots ?? []).filter(isSoundSlot);
  const want = makeSlots(setup, () => "");
  if (have.length !== want.length) return true;
  return have.some((h, i) => h.effect !== want[i]!.effect || !h.enabled || !sameParams(h.params ?? {}, want[i]!.params ?? {}));
}

/**
 * The chain with this panel's slots (and the Audio FX looks, which would double up) replaced
 * by the setup, placed first. Null when the result would exceed `max` effects.
 */
export function withSetup(chain: EffectChainDoc, setup: Setup, newId: () => string, max = 16): EffectChainDoc | null {
  const others = chain.slots.filter((s) => !isSoundSlot(s) && !isLookSlot(s));
  const made = makeSlots(setup, newId);
  if (others.length + made.length > max) return null;
  return { ...chain, slots: [...made, ...others] };
}

export const withoutSetup = (chain: EffectChainDoc): EffectChainDoc => ({ ...chain, slots: chain.slots.filter((s) => !isSoundSlot(s)) });

export const describeSetup = (s: Setup): string => `${typeInfo(s.type).label}: ${typeInfo(s.type).presets.find((p) => p.id === s.preset)?.label ?? "Custom"}, ${Math.round(s.amount)}%`;

export function presetSetup(type: SoundType, presetId: string): Setup {
  const info = typeInfo(type);
  const p = info.presets.find((x) => x.id === presetId) ?? info.presets[0]!;
  return { type, sections: p.sections, preset: p.id, amount: p.amount };
}
