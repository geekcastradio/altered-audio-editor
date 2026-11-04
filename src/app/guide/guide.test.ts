// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import guideSource from "./guide.md?raw";
import { parseGuide, searchGuide, spans } from "./guide";

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

  it("reads a picture on its own line and searches its caption", () => {
    const s = parseGuide("## Mixer\n\nText\n![Faders on a bus](bus)\nMore");
    expect(s[0]!.blocks.map((b) => b.kind)).toEqual(["p", "img", "p"]);
    const img = s[0]!.blocks[1]!;
    expect(img.kind === "img" && [img.name, img.alt]).toEqual(["bus", "Faders on a bus"]);
    expect(searchGuide(s, "faders").length).toBe(1);
  });

  it("every picture the guide names exists and is a plain SVG", () => {
    const files = import.meta.glob("./images/*.svg", { eager: true, query: "?raw", import: "default" }) as Record<string, string>;
    const named = [...guideSource.matchAll(/^!\[[^\]]*\]\(([^)]+)\)$/gm)].map((m) => m[1]!);
    expect(named.length).toBeGreaterThan(0);
    for (const n of named) {
      const svg = files[`./images/${n}.svg`];
      expect(svg, n).toBeTruthy();
      expect(svg, n).toContain("<svg");
      expect(svg, n).not.toMatch(/<script|onload=|onclick=/i);
    }
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
