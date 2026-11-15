// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { baseName, draggedOffset, laneBar, MIN_BAR } from "./video-lane";

describe("laneBar", () => {
  it("places the bar by offset and length", () => {
    // 100 samples per pixel at 48 kHz: one second is 480 px.
    expect(laneBar(1, 2, 100, 0, 48000, 2000)).toEqual({ left: 480, width: 960 });
  });
  it("cuts the bar at the left and right edges", () => {
    expect(laneBar(0, 10, 100, 48000, 48000, 1000)).toEqual({ left: 0, width: 1000 });
  });
  it("is null when the video is out of view", () => {
    expect(laneBar(0, 1, 100, 96000, 48000, 1000)).toBeNull();
    expect(laneBar(30, 1, 100, 0, 48000, 1000)).toBeNull();
  });
  it("runs to the right edge when the length is unknown", () => {
    expect(laneBar(0.5, 0, 100, 0, 48000, 1000)).toEqual({ left: 240, width: 760 });
  });
  it("never draws a bar too small to grab", () => {
    expect(laneBar(0, 0.001, 1000, 0, 48000, 1000)!.width).toBe(MIN_BAR);
  });
});

describe("draggedOffset", () => {
  it("moves by the distance dragged and stops at zero", () => {
    expect(draggedOffset(1, 480, 100, 48000)).toBe(2);
    expect(draggedOffset(1, -2000, 100, 48000)).toBe(0);
  });
});

describe("baseName", () => {
  it("takes the last part of either kind of path", () => {
    expect(baseName("C:\\a\\b\\take.mp4")).toBe("take.mp4");
    expect(baseName("/a/b/take.mp4")).toBe("take.mp4");
  });
});
