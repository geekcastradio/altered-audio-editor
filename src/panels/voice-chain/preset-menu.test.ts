// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { BUILTIN_PRESETS, PRESET_INFO } from "./chain";
import { presetMenu } from "./preset-menu";

describe("presetMenu", () => {
  it("groups voice chains and describes every built-in voice chain", () => {
    for (const p of BUILTIN_PRESETS.filter((x) => x.group === "Voice chains")) expect(PRESET_INFO[p.name], p.name).toBeDefined();
    const picked: string[] = [];
    const menu = presetMenu([], BUILTIN_PRESETS, (n) => picked.push(n));
    const voice = menu[0]!;
    expect(voice.kind === "sub" && voice.items.map((i) => (i.kind === "sub" ? i.label : ""))).toEqual(["Podcast Voices", "Character Types", "Cleanup", "Voice Over"]);
    const podcast = voice.kind === "sub" ? voice.items[0] : null;
    const first = podcast?.kind === "sub" ? podcast.items[0] : null;
    expect(first?.kind === "item" && first.shortcut).toMatch(/limit/);
    if (first?.kind === "item") first.run();
    expect(picked).toEqual(["Podcast Voice"]);
  });
  it("lists the user's own presets first", () => {
    const menu = presetMenu([BUILTIN_PRESETS[0]!], BUILTIN_PRESETS, () => {});
    expect(menu[0]).toMatchObject({ kind: "sub", label: "Yours" });
  });
  it("lists starred presets first, in star order, and skips names that are gone", () => {
    const [a, b] = [BUILTIN_PRESETS[0]!, BUILTIN_PRESETS[1]!];
    const menu = presetMenu([], BUILTIN_PRESETS, () => {}, undefined, undefined, [b.name, "No Such Preset", a.name]);
    const first = menu[0]!;
    expect(first.kind).toBe("sub");
    if (first.kind === "sub") {
      expect(first.label).toBe("Favorites");
      expect(first.items.map((i) => (i.kind === "item" ? i.label : ""))).toEqual([b.name, a.name]);
    }
  });
  it("has no Favorites entry when nothing is starred", () => {
    const menu = presetMenu([], BUILTIN_PRESETS, () => {});
    expect(menu.some((m) => m.kind === "sub" && m.label === "Favorites")).toBe(false);
  });
});
