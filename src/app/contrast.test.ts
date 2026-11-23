// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import css from "../styles/tokens.css?raw";

/** The `--name: #hex` tokens in one rule, found by its selector. */
function tokens(selector: string): Record<string, string> {
  const start = css.indexOf(selector);
  if (start < 0) throw new Error(`no rule ${selector}`);
  const body = css.slice(css.indexOf("{", start) + 1, css.indexOf("}", start));
  const out: Record<string, string> = {};
  for (const m of body.matchAll(/--([\w-]+):\s*(#[0-9a-fA-F]{6})\s*;/g)) out[m[1]!] = m[2]!;
  return out;
}

function lum(hex: string): number {
  const c = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255).map((v) => (v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4));
  return 0.2126 * c[0]! + 0.7152 * c[1]! + 0.0722 * c[2]!;
}

export function ratio(a: string, b: string): number {
  const [hi, lo] = [lum(a), lum(b)].sort((x, y) => y - x) as [number, number];
  return (hi + 0.05) / (lo + 0.05);
}

const dark = tokens(":root {");
const light = { ...dark, ...tokens(':root[data-theme="light"]') };

// [foreground, background, minimum ratio]. 4.5 for text, 3 for large marks and control edges.
const PAIRS: [string, string, number][] = [
  ["text", "surface-0", 4.5],
  ["text", "surface-1", 4.5],
  ["text", "surface-2", 4.5],
  ["text", "surface-3", 4.5],
  ["text", "header-a", 4.5],
  ["text", "header-b", 4.5],
  ["text", "lane-a", 4.5],
  ["text", "lane-selected", 4.5],
  ["text-dim", "surface-1", 4.5],
  ["text-dim", "surface-2", 4.5],
  ["text-dim", "header-a", 4.5],
  ["text-dim", "surface-3", 4.5],
  ["value", "surface-2", 4.5],
  ["value", "surface-3", 4.5],
  ["value", "header-a", 4.5],
  ["value", "header-b", 4.5],
  ["accent", "surface-1", 3],
  ["accent", "header-a", 3],
  ["title-bar-text", "title-bar", 4.5],
  ["border-strong", "surface-1", 1.5],
];

describe("theme contrast", () => {
  for (const [name, theme] of [["dark", dark], ["light", light]] as const) {
    it(`${name} theme text and controls are readable`, () => {
      const bad: string[] = [];
      for (const [fg, bg, min] of PAIRS) {
        const r = ratio(theme[fg]!, theme[bg]!);
        if (r < min) bad.push(`${fg} on ${bg}: ${r.toFixed(2)} (needs ${min})`);
      }
      expect(bad).toEqual([]);
    });
  }

  it("light lanes are darker than the page so clips stand out", () => {
    expect(lum(light["lane-a"]!)).toBeLessThan(lum(light["surface-0"]!));
  });
});
