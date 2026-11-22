// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Checks preset packs and Batch Convert recipes in TypeScript against the fixture the Rust test
// reads too (shared-fixtures/pack-recipes.json).
import { describe, expect, it } from "vitest";
import fx from "../../../shared-fixtures/pack-recipes.json";
import { BUILTIN_PRESETS } from "./chain";
import * as R from "./batch-recipes";
import * as P from "./preset-pack";

const J = (v: unknown) => JSON.parse(JSON.stringify(v ?? null));

describe("preset pack and recipes fixture", () => {
  it("reading and merging", () => {
    for (const c of fx.reads) expect(J(P.readPack(c.text))).toEqual(c.out);
    const mine = fx.mine as never as { chainsMine: never[]; bankMine: never };
    for (const c of fx.merges) {
      const pack = P.readPack(c.text) as P.ReadPack;
      const [chains, bank] = c.k === 0 ? [mine.chainsMine, mine.bankMine] : [[], {}];
      expect(J(P.mergePack(chains, bank, pack, ["Podcast Voice"]))).toEqual(c.out);
    }
  });
  it("packing and names", () => {
    const mine = fx.mine as never as { chainsMine: never[]; bankMine: never };
    for (const c of fx.packed) expect(P.packPresets(c.k === 0 ? mine.chainsMine : [], c.k === 0 ? mine.bankMine : {})).toBe(c.text);
    for (const c of fx.names) {
      const cut = (i: number) => `${c.name.slice(0, 60 - ` (${i})`.length)} (${i})`;
      const taken = new Set([c.name, ...Array.from({ length: c.n }, (_, i) => cut(i + 2))]);
      expect(P.uniqueName(c.name, (n) => taken.has(n))).toBe(c.out);
    }
  });
  it("recipes", () => {
    expect(J(R.RECIPE_GROUPS)).toEqual(fx.recipes.groups);
    expect(J(R.RECIPE_ONLY_GROUPS)).toEqual(fx.recipes.only);
    expect(J(R.BATCH_RECIPES)).toEqual(fx.recipes.all);
    expect(J(R.buildRecipes([]))).toEqual(fx.recipes.none);
    const taken = [...BUILTIN_PRESETS.slice(0, 3), { name: "Soft", group: "x", chain: {} } as never, ...R.BATCH_RECIPES.slice(0, 5)];
    expect(J(R.buildRecipes(taken))).toEqual(fx.recipes.taken);
  });
});
