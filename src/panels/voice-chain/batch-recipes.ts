// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * Batch recipes: every built-in preset of the effects that make sense on a whole file, offered
 * in Batch Convert as a one-effect chain. They are built from the single-effect presets, so a
 * recipe and the preset of the same name always carry the same settings.
 */

import { BUILTIN_PRESETS, type ChainPreset } from "./chain";
import { BUILTIN_EFFECT_PRESETS } from "./effect-presets";

/** The effects offered as recipes, and the Batch Convert group each one is listed under. */
export const RECIPE_GROUPS: Readonly<Record<string, string>> = {
  "altaudio.dynamics": "Compression",
  "altaudio.dynamics_curve": "Compression",
  "altaudio.speech_leveler": "Leveling",
  "altaudio.limiter": "Limiting",
  "altaudio.breath_gate": "Breath gates",
  "altaudio.expander": "Breath gates",
  "altaudio.noise_reducer": "Noise reduction",
  "altaudio.click_repair": "Repair",
  "altaudio.precision_filter": "Filters",
  "altaudio.parametric_eq": "EQ",
};

/** Group names the recipes add to the ones the chains already use, in the order Batch Convert lists them. */
export const RECIPE_ONLY_GROUPS: readonly string[] = ["Compression", "Leveling", "Limiting", "Noise reduction", "Filters"];

/** The label shown before a name when two presets would otherwise share it. */
const LABELS: Readonly<Record<string, string>> = {
  "altaudio.dynamics": "Dynamics",
  "altaudio.dynamics_curve": "Curve",
  "altaudio.speech_leveler": "Leveler",
  "altaudio.limiter": "Limiter",
  "altaudio.breath_gate": "Gate",
  "altaudio.expander": "Expander",
  "altaudio.noise_reducer": "Noise",
  "altaudio.click_repair": "Clicks",
  "altaudio.precision_filter": "Filter",
  "altaudio.parametric_eq": "EQ",
};

/** The recipes for the given chains: a name that is already taken gets its effect's label in front. */
export function buildRecipes(taken: readonly ChainPreset[]): ChainPreset[] {
  const used = new Set(taken.map((p) => p.name));
  const out: ChainPreset[] = [];
  for (const [effect, group] of Object.entries(RECIPE_GROUPS)) {
    for (const preset of BUILTIN_EFFECT_PRESETS[effect] ?? []) {
      let name = preset.name;
      if (used.has(name)) name = `${LABELS[effect] ?? effect}: ${preset.name}`;
      if (used.has(name)) continue;
      used.add(name);
      out.push({
        name,
        group,
        chain: { enabled: true, input_gain_db: 0, output_gain_db: 0, mix: 1, slots: [{ effect, params: { ...preset.params } }] },
      });
    }
  }
  return out;
}

/** Every recipe Batch Convert offers. */
export const BATCH_RECIPES: readonly ChainPreset[] = buildRecipes(BUILTIN_PRESETS);
