// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import guideSource from "./guide.md?raw";
import { parseGuide, searchGuide, spans, welcomeDue } from "./guide";

describe("guide parsing", () => {
  it("splits bold and code runs", () => {
    expect(spans("a **b** `c` d")).toEqual([{ text: "a " }, { text: "b", bold: true }, { text: " " }, { text: "c", code: true }, { text: " d" }]);
  });

  it("makes sections, paragraphs and lists", () => {
    const s = parseGuide("## One\n\nHello\nthere.\n\n- a\n- b\n  more\n\n1. x\n2. y\n\n## Two Part\n\nBye");
    expect(s.map((x) => x.id)).toEqual(["one", "two-part"]);
    expect(s[0]!.blocks.map((b) => b.kind)).toEqual(["p", "ul", "ol"]);
    const p = s[0]!.blocks[0]!;
    expect(p.kind === "p" && p.spans[0]!.text).toBe("Hello there.");
    const ul = s[0]!.blocks[1]!;
    expect(ul.kind === "ul" && ul.items).toHaveLength(2);
  });

  it("searches by every word", () => {
    const s = parseGuide("## Mixer\n\nFaders and buses\n\n## Export\n\nMixdown");
    expect(searchGuide(s, "").length).toBe(2);
    expect(searchGuide(s, "buses fader").map((x) => x.title)).toEqual(["Mixer"]);
    expect(searchGuide(s, "nothing")).toEqual([]);
  });

  it("the shipped guide has sections, no empty ones and no em dashes", () => {
    const s = parseGuide(guideSource);
    expect(s.length).toBeGreaterThan(8);
    for (const x of s) expect(x.blocks.length, x.title).toBeGreaterThan(0);
    expect(guideSource).not.toContain("—");
    expect(new Set(s.map((x) => x.id)).size).toBe(s.length);
  });
});

describe("welcomeDue", () => {
  it("shows for a fresh install, once", () => {
    expect(welcomeDue(undefined)).toBe(true);
    expect(welcomeDue(null)).toBe(true);
    expect(welcomeDue({ welcomeSeen: false })).toBe(true);
    expect(welcomeDue({ welcomeSeen: true })).toBe(false);
    expect(welcomeDue({ musicMode: true })).toBe(false);
  });
});
