// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it, vi } from "vitest";
import type { ContextItem } from "../../app/ContextMenu.svelte";
import { markerMenuItems, type MarkerMenuDeps } from "./marker-menu";
import { newMarker } from "./markers";

function deps(over: Partial<MarkerMenuDeps> = {}): MarkerMenuDeps {
  return { rate: 1000, selection: null, playlist: [], goTo: vi.fn(), rename: vi.fn(), change: vi.fn(), setPlaylist: vi.fn(), remove: vi.fn(), ...over };
}
const labels = (items: ContextItem[]) => items.flatMap((i) => (i.kind === "item" || i.kind === "sub" ? [i.label] : []));
const find = (items: ContextItem[], label: string) => items.find((i) => (i.kind === "item" || i.kind === "sub") && i.label === label);

describe("marker menu", () => {
  it("offers Show in Markers Panel only when a panel handler is given", () => {
    const m = newMarker("a", "A", 0);
    expect(labels(markerMenuItems(m, deps()))).not.toContain("Show in Markers Panel");
    expect(labels(markerMenuItems(m, deps({ showInPanel: () => {} })))).toContain("Show in Markers Panel");
  });

  it("converts a point to a range sized by the selection, and a range back to a point", () => {
    const d = deps({ selection: { start: 0, end: 700 } });
    const point = find(markerMenuItems(newMarker("a", "A", 200), d), "Convert to Range");
    if (point?.kind !== "item") throw new Error("missing");
    point.run();
    expect(d.change).toHaveBeenCalledWith("Convert to Range Marker", expect.anything(), { duration: 500 });
    const range = find(markerMenuItems(newMarker("b", "B", 200, 300), d), "Convert to Point");
    if (range?.kind !== "item") throw new Error("missing");
    range.run();
    expect(d.change).toHaveBeenLastCalledWith("Convert to Point Marker", expect.anything(), { duration: 0 });
  });

  it("keeps points out of the playlist and lets a listed range leave it", () => {
    const point = find(markerMenuItems(newMarker("a", "A", 0), deps()), "Insert into Playlist");
    expect(point).toMatchObject({ disabled: true });
    const d = deps({ playlist: ["r"] });
    const listed = find(markerMenuItems(newMarker("r", "R", 0, 10), d), "Remove from Playlist");
    if (listed?.kind !== "item") throw new Error("missing");
    listed.run();
    expect(d.setPlaylist).toHaveBeenCalledWith([]);
  });

  it("lists every type with the current one checked and changes only to a different one", () => {
    const d = deps();
    const m = newMarker("a", "A", 0);
    const sub = find(markerMenuItems(m, d), "Change Marker Type");
    if (sub?.kind !== "sub") throw new Error("missing");
    expect(sub.items.length).toBe(7);
    const cue = sub.items.find((i) => i.kind === "item" && i.label === "Cue");
    expect(cue).toMatchObject({ checked: true });
    if (cue?.kind === "item") cue.run();
    expect(d.change).not.toHaveBeenCalled();
    const chapter = sub.items.find((i) => i.kind === "item" && i.label === "Chapter");
    if (chapter?.kind === "item") chapter.run();
    expect(d.change).toHaveBeenCalledWith("Marker Type", m, { kind: "chapter" });
  });
});
