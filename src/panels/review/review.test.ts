// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { newMarker } from "../markers/markers";
import { reviewCounts, reviewKind, reviewMarks, stepMark } from "./review";

const list = [
  newMarker("a", "Great bit 1", 300),
  newMarker("b", "Intro", 50),
  newMarker("c", "cut this 2", 100),
  newMarker("d", "Retake 1", 200),
  newMarker("e", "Retake 2", 400),
];

describe("review marks", () => {
  it("knows the three kinds by name, any case, and ignores other markers", () => {
    expect(reviewKind({ name: "Cut this 7" })).toBe("cut");
    expect(reviewKind({ name: "  retake" })).toBe("retake");
    expect(reviewKind({ name: "GREAT BIT 2" })).toBe("great");
    expect(reviewKind({ name: "Intro" })).toBeNull();
  });

  it("lists them in time order, all or one kind", () => {
    expect(reviewMarks(list).map((m) => m.id)).toEqual(["c", "d", "a", "e"]);
    expect(reviewMarks(list, "retake").map((m) => m.id)).toEqual(["d", "e"]);
    expect(reviewMarks(list, "cut").map((m) => m.id)).toEqual(["c"]);
    expect(reviewMarks([], "all")).toEqual([]);
  });

  it("counts each kind", () => {
    expect(reviewCounts(list)).toEqual({ cut: 1, retake: 2, great: 1 });
  });

  it("steps to the next or previous mark, skipping one at the spot", () => {
    const marks = reviewMarks(list);
    expect(stepMark(marks, 0, 1)?.id).toBe("c");
    expect(stepMark(marks, 100, 1)?.id).toBe("d");
    expect(stepMark(marks, 400, 1)).toBeNull();
    expect(stepMark(marks, 400, -1)?.id).toBe("a");
    expect(stepMark(marks, 100, -1)).toBeNull();
    expect(stepMark([], 10, 1)).toBeNull();
  });
});
