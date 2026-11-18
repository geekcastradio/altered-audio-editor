// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { ownsArrowKeys } from "./owns-arrows";

const el = (tagName: string, extra: Record<string, unknown> = {}, role: string | null = null) => ({
  tagName,
  getAttribute: (n: string) => (n === "role" ? role : null),
  ...extra,
});

describe("ownsArrowKeys", () => {
  it("lets sliders, drop-downs and lists keep their arrow keys", () => {
    expect(ownsArrowKeys(el("INPUT", { type: "range" }))).toBe(true);
    expect(ownsArrowKeys(el("SELECT"))).toBe(true);
    expect(ownsArrowKeys(el("DIV", {}, "listbox"))).toBe(true);
    expect(ownsArrowKeys(el("DIV", {}, "tab"))).toBe(true);
  });

  it("does not claim plain controls, the page or nothing", () => {
    expect(ownsArrowKeys(el("BUTTON"))).toBe(false);
    expect(ownsArrowKeys(el("DIV"))).toBe(false);
    expect(ownsArrowKeys(el("INPUT", { type: "checkbox" }))).toBe(false);
    expect(ownsArrowKeys(null)).toBe(false);
  });
});
