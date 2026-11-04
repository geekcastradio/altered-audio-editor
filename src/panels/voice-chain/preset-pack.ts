// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * A preset pack: your saved Voice Chains and single-effect presets in one small JSON file, so
 * they can be backed up, moved to another computer or handed to a co-host. Reading a pack never
 * overwrites anything you already have; a name that is taken gets a number after it.
 */

import { sanitizePresets, type ChainPreset } from "./chain";
import { MAX_PRESETS_PER_EFFECT, sanitizeBank, type EffectPreset, type EffectPresetBank } from "./effect-presets";

export const PACK_FORMAT = "altaudio.presets";
export const PACK_VERSION = 1;

/** The pack as text: pretty printed so it diffs and reads well. */
export function packPresets(chains: readonly ChainPreset[], bank: EffectPresetBank): string {
  return JSON.stringify({ format: PACK_FORMAT, version: PACK_VERSION, chains, effects: bank }, null, 2) + "\n";
}

export interface ReadPack {
  chains: ChainPreset[];
  bank: EffectPresetBank;
}

/** A pack read from text, or the reason it cannot be used. */
export function readPack(text: string): ReadPack | { error: string } {
  let raw: unknown;
  try {
    raw = JSON.parse(text);
  } catch {
    return { error: "That file is not a preset pack." };
  }
  if (!raw || typeof raw !== "object" || (raw as { format?: unknown }).format !== PACK_FORMAT) return { error: "That file is not an Altered Audio preset pack." };
  const v = (raw as { version?: unknown }).version;
  if (typeof v !== "number" || v > PACK_VERSION) return { error: "That pack was made by a newer version of Altered Audio. Update the app to read it." };
  const r = raw as { chains?: unknown; effects?: unknown };
  return { chains: sanitizePresets(r.chains), bank: sanitizeBank(r.effects) };
}

/** `name` made unique among `taken` by adding " (2)", " (3)" and so on, within 60 characters. */
export function uniqueName(name: string, taken: (n: string) => boolean): string {
  if (!taken(name)) return name;
  for (let i = 2; ; i++) {
    const suffix = ` (${i})`;
    const next = name.slice(0, 60 - suffix.length) + suffix;
    if (!taken(next)) return next;
  }
}

export interface Merged {
  chains: ChainPreset[];
  bank: EffectPresetBank;
  addedChains: number;
  addedEffects: number;
  /** Presets that kept their settings but got a new name because theirs was taken. */
  renamed: number;
  /** Single-effect presets left out because that effect already holds the most it can. */
  full: number;
}

const same = (a: EffectPreset, b: EffectPreset) => JSON.stringify(a.params) === JSON.stringify(b.params) && a.data === b.data;

/**
 * Your presets plus the pack's. Nothing you have is replaced. A preset that matches one you have
 * in name and settings is skipped; one that only shares the name comes in under a new name.
 * `reserved` are names that cannot be used (the built-in chains).
 */
export function mergePack(chains: readonly ChainPreset[], bank: EffectPresetBank, pack: ReadPack, reserved: readonly string[]): Merged {
  const lower = (s: string) => s.toLowerCase();
  const outChains = [...chains];
  const used = new Set([...reserved, ...chains.map((c) => c.name)].map(lower));
  let addedChains = 0;
  let addedEffects = 0;
  let renamed = 0;
  let full = 0;
  for (const c of pack.chains) {
    const mine = outChains.find((x) => lower(x.name) === lower(c.name));
    if (mine && JSON.stringify(mine.chain) === JSON.stringify(c.chain)) continue;
    const name = uniqueName(c.name, (n) => used.has(lower(n)));
    if (name !== c.name) renamed++;
    used.add(lower(name));
    outChains.push({ ...c, name });
    addedChains++;
  }
  outChains.sort((a, b) => a.name.localeCompare(b.name));
  const outBank: EffectPresetBank = {};
  for (const [effect, list] of Object.entries(bank)) outBank[effect] = [...list];
  for (const [effect, list] of Object.entries(pack.bank)) {
    const have = outBank[effect] ?? [];
    for (const p of list) {
      const mine = have.find((x) => lower(x.name) === lower(p.name));
      if (mine && same(mine, p)) continue;
      if (have.length >= MAX_PRESETS_PER_EFFECT) {
        full++;
        continue;
      }
      const name = uniqueName(p.name, (n) => have.some((x) => lower(x.name) === lower(n)));
      if (name !== p.name) renamed++;
      have.push({ ...p, name });
      addedEffects++;
    }
    if (have.length) outBank[effect] = have.sort((a, b) => a.name.localeCompare(b.name, undefined, { numeric: true, sensitivity: "base" }));
  }
  return { chains: outChains, bank: outBank, addedChains, addedEffects, renamed, full };
}
