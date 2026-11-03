// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import type { LibraryItem } from "../../app/core";
import { arrange, DEFAULT_VIEW, flatIds, foldedIds, sanitizeView, toggleSection } from "./arrange";

const item = (id: string, name: string, kind: "audio" | "image" = "audio", added = "2026-01-01T00:00:00Z"): LibraryItem => ({
  id, name, kind, file: `${name}.wav`, copied: true, original_path: null, sample_rate: 48000, channels: 2, length: 1, tags: [], notes: "", color: null, added,
});
const items = [item("a", "Intro 10"), item("b", "Intro 2"), item("c", "Cover", "image", "2026-03-01T00:00:00Z"), item("d", "Bed")];
const lib = {
  collections: [
    { id: "k1", name: "Intro", items: ["b", "a"], color: "--clip-2" },
    { id: "k2", name: "Art", items: ["c"], color: null },
  ],
};

describe("Libraries arrangement", () => {
  it("groups by bin in the bin's own order, with the leftovers last", () => {
    const s = arrange(lib, items, DEFAULT_VIEW);
    expect(s.map((x) => [x.name, x.items.map((i) => i.id)])).toEqual([
      ["Intro", ["b", "a"]],
      ["Art", ["c"]],
      ["Unsorted", ["d"]],
    ]);
    expect(s[0]?.bin).toBe("k1");
    expect(s[2]?.bin).toBeNull();
  });

  it("an item in two bins appears in both but is selectable once", () => {
    const two = { collections: [{ id: "x", name: "X", items: ["a"], color: null }, { id: "y", name: "Y", items: ["a", "d"], color: null }] };
    const s = arrange(two, items, DEFAULT_VIEW);
    expect(flatIds(s, [])).toEqual(["a", "d", "b", "c"]);
  });

  it("filters by kind, sorts naturally by name or newest first, and groups by type", () => {
    const pics = arrange(lib, items, { ...DEFAULT_VIEW, filter: "image", groupBy: "none" });
    expect(pics[0]?.items.map((i) => i.id)).toEqual(["c"]);
    const byName = arrange(lib, items, { ...DEFAULT_VIEW, groupBy: "none", sortBy: "name" });
    expect(byName[0]?.items.map((i) => i.name)).toEqual(["Bed", "Cover", "Intro 2", "Intro 10"]);
    const byDate = arrange(lib, items, { ...DEFAULT_VIEW, groupBy: "none", sortBy: "date" });
    expect(byDate[0]?.items[0]?.id).toBe("c");
    const types = arrange(lib, items, { ...DEFAULT_VIEW, groupBy: "type" });
    expect(types.map((s) => s.name)).toEqual(["Audio", "Pictures"]);
  });

  it("a library with no bins shows one section, and folded sections drop out of selection order", () => {
    const s = arrange({ collections: [] }, items, DEFAULT_VIEW);
    expect(s).toHaveLength(1);
    expect(s[0]?.name).toBe("All");
    const grouped = arrange(lib, items, DEFAULT_VIEW);
    expect(flatIds(grouped, ["k1"])).toEqual(["c", "d"]);
    expect(flatIds(grouped, foldedIds(grouped, DEFAULT_VIEW, false))).toEqual([]);
  });

  it("opens and folds sections, and repairs a damaged saved view", () => {
    const v = toggleSection(DEFAULT_VIEW, "k1", true);
    expect(v.open).toEqual(["k1"]);
    expect(toggleSection(v, "k1", false).open).toEqual([]);
    expect(toggleSection(v, "k1", true).open).toEqual(["k1"]);
    expect(sanitizeView({ groupBy: "nonsense", view: "grid", open: [1, "a"] })).toEqual({ ...DEFAULT_VIEW, view: "grid", open: ["a"] });
    expect(sanitizeView({ collapsed: ["old"] })).toEqual(DEFAULT_VIEW);
    expect(sanitizeView(null)).toEqual(DEFAULT_VIEW);
  });

  it("starts every bin folded, keeps the ones the user opened, and opens all while searching", () => {
    const s = arrange(lib, items, DEFAULT_VIEW);
    expect(foldedIds(s, DEFAULT_VIEW, false)).toEqual(["k1", "k2", "unsorted"]);
    expect(foldedIds(s, { ...DEFAULT_VIEW, open: ["k2"] }, false)).toEqual(["k1", "unsorted"]);
    expect(foldedIds(s, DEFAULT_VIEW, true)).toEqual([]);
  });

  it("never folds a library that has a single section", () => {
    const one = arrange({ collections: [] }, items, DEFAULT_VIEW);
    expect(foldedIds(one, DEFAULT_VIEW, false)).toEqual([]);
  });
});
