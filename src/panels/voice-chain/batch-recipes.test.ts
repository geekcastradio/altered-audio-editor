// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { BATCH_RECIPES, RECIPE_GROUPS, RECIPE_ONLY_GROUPS } from "./batch-recipes";
import { BUILTIN_PRESETS } from "./chain";
import { BUILTIN_EFFECT_PRESETS } from "./effect-presets";

describe("Batch recipes", () => {
  it("never share a name with each other or with a built-in chain", () => {
    const names = [...BUILTIN_PRESETS, ...BATCH_RECIPES].map((p) => p.name);
    expect(new Set(names).size).toBe(names.length);
  });

  it("make one recipe for every preset of the offered effects", () => {
    const want = Object.keys(RECIPE_GROUPS).reduce((n, e) => n + (BUILTIN_EFFECT_PRESETS[e]?.length ?? 0), 0);
    expect(BATCH_RECIPES.length).toBe(want);
    for (const r of BATCH_RECIPES) {
      expect(r.trackOnly).toBeUndefined();
      expect(r.chain.slots).toHaveLength(1);
      expect(RECIPE_GROUPS[r.chain.slots[0]!.effect]).toBe(r.group);
    }
  });

  it("carry the same settings as the single-effect preset", () => {
    const first = BUILTIN_EFFECT_PRESETS["altaudio.limiter"]![0]!;
    const r = BATCH_RECIPES.find((x) => x.name === first.name || x.name.endsWith(`: ${first.name}`))!;
    expect(r.chain.slots[0]!.params).toEqual(first.params);
    expect(r.chain.slots[0]!.params).not.toBe(first.params);
  });

  it("list their own groups once", () => {
    expect(new Set(RECIPE_ONLY_GROUPS).size).toBe(RECIPE_ONLY_GROUPS.length);
    for (const g of RECIPE_ONLY_GROUPS) expect(BATCH_RECIPES.some((r) => r.group === g)).toBe(true);
  });
});
