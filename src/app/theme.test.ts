// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
import { describe, expect, it } from "vitest";
import { resolveTheme, THEMES } from "./theme";

describe("resolveTheme", () => {
  it("honors an explicit choice", () => {
    expect(resolveTheme("light", true)).toBe("light");
    expect(resolveTheme("dark", false)).toBe("dark");
  });
  it("follows the system for sync", () => {
    expect(resolveTheme("system", true)).toBe("dark");
    expect(resolveTheme("system", false)).toBe("light");
  });
  it("falls back to dark", () => {
    expect(resolveTheme("nope", false)).toBe("dark");
  });
  it("describes all three choices", () => {
    expect(THEMES.map((t) => t.id)).toEqual(["light", "system", "dark"]);
    for (const t of THEMES) expect(t.hint.length).toBeGreaterThan(20);
  });
});
