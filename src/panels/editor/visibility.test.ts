// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.
import { describe, expect, it } from "vitest";
import { hasClipIn, hiddenNames, keepByPreset, MAX_VIS_PRESETS, sanitizePresets, showOnly, shownTracks, withPreset } from "./visibility";

const t = (id: string, clips: number, hidden = false) => ({
  id,
  hidden,
  clips: Array.from({ length: clips }, (_, i) => ({ id: `${id}${i}` })),
});

describe("visibility presets", () => {
  const tr = (name: string, hidden = false) => ({ name, hidden });
  it("saves the hidden names and loads them by name, ignoring case", () => {
    const names = hiddenNames([tr("Host"), tr("Music", true), tr("Guest", true)]);
    expect(names).toEqual(["Music", "Guest"]);
    const keep = keepByPreset(["music", " GUEST "]);
    expect(["Host", "Music", "Guest", "Bus"].filter((n) => keep({ name: n }))).toEqual(["Host", "Bus"]);
  });
  it("replaces a preset of the same name and keeps the newest few", () => {
    let list = withPreset([], { name: "A", hidden: ["x"] });
    list = withPreset(list, { name: "a", hidden: ["y"] });
    expect(list).toEqual([{ name: "a", hidden: ["y"] }]);
    for (let i = 0; i < MAX_VIS_PRESETS + 3; i++) list = withPreset(list, { name: `P${i}`, hidden: [] });
    expect(list).toHaveLength(MAX_VIS_PRESETS);
    expect(list.at(-1)?.name).toBe(`P${MAX_VIS_PRESETS + 2}`);
  });
  it("drops bad saved data", () => {
    expect(sanitizePresets(null)).toEqual([]);
    expect(sanitizePresets([{ name: "ok", hidden: ["a", 3] }, { name: "", hidden: [] }, 5, { name: "x" }])).toEqual([{ name: "ok", hidden: ["a"] }]);
  });
});

describe("track visibility", () => {
  it("lists only shown tracks", () => {
    expect(shownTracks([t("a", 1), t("b", 0, true)]).map((x) => x.id)).toEqual([
      "a",
    ]);
  });
  it("reports only the tracks whose flag changes", () => {
    const tracks = [t("a", 1), t("b", 0), t("c", 2, true)];
    expect(showOnly(tracks, (x) => x.clips.length > 0)).toEqual([
      { id: "b", hidden: true },
      { id: "c", hidden: false },
    ]);
    expect(showOnly([t("a", 1)], () => true)).toEqual([]);
  });
  it("finds clips in a range or at a point", () => {
    const spans = [{ start: 100, length: 50 }];
    expect(hasClipIn(spans, 120, 130)).toBe(true);
    expect(hasClipIn(spans, 150, 200)).toBe(false);
    expect(hasClipIn(spans, 149, 149)).toBe(true);
    expect(hasClipIn(spans, 150, 150)).toBe(false);
  });
});
