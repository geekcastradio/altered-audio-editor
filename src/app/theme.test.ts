// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { effectiveTheme, THEME_CHOICES } from "./theme";

describe("theme", () => {
  it("draws the chosen theme, or the computer's when synced", () => {
    expect(effectiveTheme("light", false)).toBe("light");
    expect(effectiveTheme("dark", true)).toBe("dark");
    expect(effectiveTheme("system", true)).toBe("light");
    expect(effectiveTheme("system", false)).toBe("dark");
  });
  it("describes every choice", () => {
    expect(THEME_CHOICES.map((c) => c.id)).toEqual(["light", "system", "dark"]);
    for (const c of THEME_CHOICES) expect(c.help.length).toBeGreaterThan(10);
  });
});
