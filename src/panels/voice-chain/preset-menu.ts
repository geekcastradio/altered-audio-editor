// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import type { ContextItem } from "../../app/ContextMenu.svelte";
import { PRESET_INFO, type ChainPreset } from "./chain";

const CATEGORIES = ["Podcast Voices", "Character Types", "Cleanup", "Voice Over"] as const;
const ADD_GROUPS = ["Breath gates", "Processing", "Delay", "Repair", "EQ"] as const;

/**
 * The Presets pop-out: Voice Chains (Podcast Voices, Character Types,
 * Cleanup, Voice Over) replace the chain; the single-effect groups add
 * one effect. Each row carries a short description on the right.
 */
export function presetMenu(user: readonly ChainPreset[], builtin: readonly ChainPreset[], pick: (name: string) => void): ContextItem[] {
  const row = (p: ChainPreset, hint?: string): ContextItem => ({ kind: "item", label: p.name, run: () => pick(p.name), shortcut: hint });
  const out: ContextItem[] = [];
  if (user.length) out.push({ kind: "sub", label: "Yours", items: user.map((p) => row(p)) });
  const voice = builtin.filter((p) => p.group === "Voice chains");
  out.push({
    kind: "sub",
    label: "Voice Chains (replace the chain)",
    items: CATEGORIES.map((c): ContextItem => ({
      kind: "sub",
      label: c,
      items: voice.filter((p) => (PRESET_INFO[p.name]?.category ?? "Cleanup") === c).map((p) => row(p, PRESET_INFO[p.name]?.blurb)),
    })).filter((c) => c.kind === "sub" && c.items.length > 0),
  });
  for (const g of ADD_GROUPS) {
    const list = builtin.filter((p) => p.group === g);
    if (list.length) out.push({ kind: "sub", label: `${g} (add one effect)`, items: list.map((p) => row(p, PRESET_INFO[p.name]?.blurb)) });
  }
  return out;
}
