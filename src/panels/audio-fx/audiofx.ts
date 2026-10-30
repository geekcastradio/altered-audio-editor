// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * Audio FX (dialogue): four guided looks, each Off, Light, Medium or
 * Strong. The panel is a shortcut, never a second engine: choosing looks
 * writes ordinary Voice Chain effects at the front of the chain, tagged
 * with a preset name so the panel can read them back and replace them.
 * Everything it makes can be opened and edited in the Voice Chain, and
 * building the same effects by hand sounds the same.
 */

import type { EffectChainDoc, EffectSlotDoc } from "../../stores/session";
import { BLOCKS } from "../voice-chain/chain";

export type LookId = "restoration" | "level" | "clarity" | "polish";
export type Strength = 0 | 1 | 2 | 3;
export type Looks = Record<LookId, Strength>;

export const NO_LOOKS: Looks = { restoration: 0, level: 0, clarity: 0, polish: 0 };

export interface LookInfo {
  id: LookId;
  label: string;
  /** What it does, in plain words. */
  about: string;
  /** What each strength means, Light first. */
  strengths: readonly [string, string, string];
}

/** In chain order: clean up, even out, shape, then catch the peaks. */
export const LOOKS: readonly LookInfo[] = [
  { id: "restoration", label: "Restoration", about: "Turns down steady background noise such as hiss, fans and room tone.", strengths: ["6 dB", "10 dB", "14 dB"] },
  { id: "level", label: "Level", about: "Evens out quiet and loud moments, so listeners stop reaching for the volume.", strengths: ["Gentle", "Medium", "Firm"] },
  { id: "clarity", label: "Clarity", about: "Rolls off rumble, clears the mud and adds presence, with a de-esser to keep S sounds soft.", strengths: ["Subtle", "Medium", "Bright"] },
  { id: "polish", label: "Polish", about: "Stops peaks from going over a ceiling so the voice never clips.", strengths: ["-6 dB ceiling", "-3 dB ceiling", "-1.5 dB ceiling"] },
];

/** The preset name on every slot the panel makes, ending in the look and its strength. */
export const TAG = "Audio FX: ";
const tagOf = (id: LookId, s: Strength) => `${TAG}${id}/${s}`;

export const isPanelSlot = (s: EffectSlotDoc): boolean => typeof s.preset === "string" && s.preset.startsWith(TAG);

const pick = <T,>(s: Strength, light: T, medium: T, strong: T): T => (s === 1 ? light : s === 2 ? medium : strong);

/** The plain effects one look makes at one strength (not Off). */
export function lookEffects(id: LookId, s: Exclude<Strength, 0>): { effect: string; params: Record<string, number> }[] {
  const { fx, voiceEq, leveler, comp, limit, cleanNoise } = BLOCKS;
  switch (id) {
    case "restoration":
      return [cleanNoise(pick(s, 6, 10, 14))];
    case "level":
      return [leveler, pick(s, comp(-18, 2, 2), comp(-22, 3, 4), comp(-26, 4, 6))];
    case "clarity":
      return [
        { ...voiceEq, params: { ...voiceEq.params, b3_gain: pick(s, 1, 2, 3.5) } },
        fx("deesser", { freq: 6500, threshold: pick(s, -28, -32, -34), reduction: pick(s, 4, 6, 8) }),
      ];
    case "polish":
      return [limit(pick(s, -6, -3, -1.5))];
  }
}

/** Which looks the chain already carries, read back from its tagged slots. */
export function readLooks(chain: EffectChainDoc | null | undefined): Looks {
  const out: Looks = { ...NO_LOOKS };
  for (const s of chain?.slots ?? []) {
    const m = isPanelSlot(s) ? /^Audio FX: (\w+)\/([1-3])$/.exec(s.preset!) : null;
    if (m && m[1]! in out) out[m[1] as LookId] = Number(m[2]) as Strength;
  }
  return out;
}

/**
 * The chain with the panel's slots replaced by the chosen looks, placed
 * first; every other effect stays where it was, after them. Null when the
 * result would hold more than `max` effects.
 */
export function withLooks(chain: EffectChainDoc, looks: Looks, newId: () => string, max = 16): EffectChainDoc | null {
  const others = chain.slots.filter((s) => !isPanelSlot(s) && !isSoundTypeSlot(s));
  const made: EffectSlotDoc[] = [];
  for (const info of LOOKS) {
    const s = looks[info.id];
    if (s === 0) continue;
    for (const e of lookEffects(info.id, s)) {
      made.push({ id: newId(), effect: e.effect, enabled: true, params: { ...e.params }, preset: tagOf(info.id, s) });
    }
  }
  if (others.length + made.length > max) return null;
  return { ...chain, slots: [...made, ...others] };
}

/** The chain without the panel's slots. */
export const withoutLooks = (chain: EffectChainDoc): EffectChainDoc => ({ ...chain, slots: chain.slots.filter((s) => !isPanelSlot(s)) });

/** True for slots made by the sibling Sound Types tab, which replaces the looks rather than stacking on them. */
const isSoundTypeSlot = (s: EffectSlotDoc): boolean => typeof s.preset === "string" && s.preset.startsWith("Sound Types: ");

/** The name of the History entry: "Dialogue: Restoration, Clarity". */
export function describeLooks(looks: Looks): string {
  const on = LOOKS.filter((l) => looks[l.id] !== 0).map((l) => l.label);
  return on.length ? `Dialogue: ${on.join(", ")}` : "Dialogue: Off";
}

export const sameLooks = (a: Looks, b: Looks): boolean => LOOKS.every((l) => a[l.id] === b[l.id]);
