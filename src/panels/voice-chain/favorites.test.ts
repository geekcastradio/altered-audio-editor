// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { MAX_FAVORITES, sanitizeFavorites, toggleFavorite } from "./favorites";

describe("effect favorites", () => {
  it("drops anything that is not a short string, and repeats", () => {
    expect(sanitizeFavorites(["a", "a", 3, "", null, "b"])).toEqual(["a", "b"]);
    expect(sanitizeFavorites("nope")).toEqual([]);
  });
  it("toggles an effect on and off", () => {
    expect(toggleFavorite(["a"], "b")).toEqual(["a", "b"]);
    expect(toggleFavorite(["a", "b"], "a")).toEqual(["b"]);
  });
  it("keeps the newest when the list is full", () => {
    const full = Array.from({ length: MAX_FAVORITES }, (_, i) => `e${i}`);
    const next = toggleFavorite(full, "new");
    expect(next).toHaveLength(MAX_FAVORITES);
    expect(next.at(-1)).toBe("new");
    expect(next).not.toContain("e0");
  });
});
