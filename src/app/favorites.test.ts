// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { MAX_FAVORITES, isPinnable, menuLabels, sanitizeFavorites, shortLabel, toggleFavorite } from "./favorites";
import { MENUS } from "./menus/model";

describe("favorites", () => {
  it("pins and unpins in order", () => {
    let l: string[] = [];
    l = toggleFavorite(l, "fx.normalize");
    l = toggleFavorite(l, "edit.crop");
    expect(l).toEqual(["fx.normalize", "edit.crop"]);
    expect(toggleFavorite(l, "fx.normalize")).toEqual(["edit.crop"]);
  });

  it("will not pin the Favorites rows, Open Recent rows or nothing", () => {
    expect(isPinnable("favorites.slot.1")).toBe(false);
    expect(isPinnable("file.recent.3")).toBe(false);
    expect(isPinnable("")).toBe(false);
    expect(isPinnable("fx.normalize")).toBe(true);
    expect(toggleFavorite([], "favorites.clear")).toEqual([]);
  });

  it("stops at the limit and still unpins", () => {
    const full = Array.from({ length: MAX_FAVORITES }, (_, i) => `x.${i}`);
    expect(toggleFavorite(full, "x.new")).toEqual(full);
    expect(toggleFavorite(full, "x.0")).toHaveLength(MAX_FAVORITES - 1);
  });

  it("cleans a saved list", () => {
    expect(sanitizeFavorites(null)).toEqual([]);
    expect(sanitizeFavorites(["a.b", 3, "a.b", "favorites.slot.1", "file.recent.1", "c.d"])).toEqual(["a.b", "c.d"]);
    expect(sanitizeFavorites(Array.from({ length: 40 }, (_, i) => `x.${i}`))).toHaveLength(MAX_FAVORITES);
  });

  it("drops the trailing dots from a label", () => {
    expect(shortLabel("Normalize...")).toBe("Normalize");
    expect(shortLabel("Crop")).toBe("Crop");
    expect(shortLabel("Amplify…")).toBe("Amplify");
  });

  it("finds a label for menu commands, including inside submenus", () => {
    const labels = menuLabels(MENUS);
    expect(labels.get("file.save")).toBe("Save");
    expect(labels.size).toBeGreaterThan(100);
  });
});
