// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { deleteLabel, filterMarkers, kindChange, markerAt, markerEnd, markersBetween, newMarker, nextMarkerName, pickedMarkers, rangeLengthFor, togglePlaylist } from "./markers";

describe("markers", () => {
  const ms = [newMarker("a", "Marker 1", 100), newMarker("b", "Marker 3", 140, 50, "chapter")];

  it("names new markers without clashing", () => {
    expect(nextMarkerName(ms)).toBe("Marker 4");
    expect(nextMarkerName([newMarker("x", "Marker 2", 0)])).toBe("Marker 3");
  });

  it("finds the flag under the pointer", () => {
    const xOf = (s: number) => s / 10;
    expect(markerAt(ms, 11, xOf)?.id).toBe("a");
    expect(markerAt(ms, 14, xOf)?.id).toBe("b");
    expect(markerAt(ms, 30, xOf)).toBeNull();
  });

  it("picks markers by id in time order and ignores ids that no longer exist", () => {
    const list = [newMarker("c", "Marker 3", 300), newMarker("a", "Marker 1", 100), newMarker("b", "Marker 2", 200)];
    expect(pickedMarkers(list, new Set(["c", "a", "gone"])).map((m) => m.id)).toEqual(["a", "c"]);
    expect(pickedMarkers(list, new Set())).toEqual([]);
  });

  it("selects a run of markers between two clicks, whichever way round", () => {
    const list = [newMarker("c", "Marker 3", 300), newMarker("a", "Marker 1", 100), newMarker("b", "Marker 2", 200), newMarker("d", "Marker 4", 400)];
    expect(markersBetween(list, "a", "c")).toEqual(["a", "b", "c"]);
    expect(markersBetween(list, "d", "b")).toEqual(["b", "c", "d"]);
    expect(markersBetween(list, "a", "a")).toEqual(["a"]);
    expect(markersBetween(list, "a", "gone")).toEqual([]);
  });

  it("finds where a marker ends", () => {
    expect(markerEnd(newMarker("a", "A", 100))).toBe(100);
    expect(markerEnd(newMarker("b", "B", 100, 50))).toBe(150);
  });

  it("filters by name, description or type and keeps order", () => {
    const list = [newMarker("a", "Intro", 0), { ...newMarker("b", "Break", 10, 0, "ad_slot"), description: "Mid-roll read" }, newMarker("c", "Outro", 20, 0, "chapter")];
    expect(filterMarkers(list, "").map((m) => m.id)).toEqual(["a", "b", "c"]);
    expect(filterMarkers(list, "  OUTRO ").map((m) => m.id)).toEqual(["c"]);
    expect(filterMarkers(list, "mid-roll").map((m) => m.id)).toEqual(["b"]);
    expect(filterMarkers(list, "ad slot").map((m) => m.id)).toEqual(["b"]);
    expect(filterMarkers(list, "zzz")).toEqual([]);
  });

  it("converts a point to a range up to the selection end, else five seconds", () => {
    expect(rangeLengthFor({ position: 1000 }, { start: 500, end: 4000 }, 48000)).toBe(3000);
    expect(rangeLengthFor({ position: 5000 }, { start: 500, end: 4000 }, 48000)).toBe(240000);
    expect(rangeLengthFor({ position: 1000 }, null, 48000)).toBe(240000);
  });

  it("carries the contracted length with ad slots only", () => {
    const m = newMarker("a", "A", 0, 96000);
    expect(kindChange(m, "ad_slot", 48000)).toEqual({ kind: "ad_slot", contracted_duration: 96000 });
    expect(kindChange(newMarker("b", "B", 0), "ad_slot", 48000)).toEqual({ kind: "ad_slot", contracted_duration: 1440000 });
    expect(kindChange({ ...m, kind: "ad_slot", contracted_duration: 5 }, "cue", 48000)).toEqual({ kind: "cue", clear_contracted: true });
    expect(kindChange(m, "chapter", 48000)).toEqual({ kind: "chapter" });
  });

  it("adds a range to the playlist, takes it out again, and never adds a point", () => {
    const range = newMarker("r", "R", 0, 100);
    expect(togglePlaylist(undefined, range)).toEqual(["r"]);
    expect(togglePlaylist(["x"], range)).toEqual(["x", "r"]);
    expect(togglePlaylist(["x", "r"], range)).toEqual(["x"]);
    expect(togglePlaylist(["x"], newMarker("p", "P", 0))).toEqual(["x"]);
  });

  it("labels the undo step for one or several markers", () => {
    expect(deleteLabel(1)).toBe("Delete Marker");
    expect(deleteLabel(3)).toBe("Delete 3 Markers");
  });
});
