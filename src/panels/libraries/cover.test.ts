// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { isPicturePath, picturesOnly, popoverPosition } from "./cover";

describe("cover pictures", () => {
  it("takes JPEG and PNG files in any letter case and nothing else", () => {
    expect(isPicturePath("C:\\Art\\show.PNG")).toBe(true);
    expect(isPicturePath("/a/b/c.jpeg")).toBe(true);
    expect(isPicturePath("cover.jpg")).toBe(true);
    expect(isPicturePath("cover.gif")).toBe(false);
    expect(isPicturePath("song.wav")).toBe(false);
    expect(picturesOnly(["a.wav", "b.png", "c.JPG"])).toEqual(["b.png", "c.JPG"]);
  });
});

describe("popoverPosition", () => {
  const view = { w: 1000, h: 700 };
  const size = { w: 280, h: 360 };

  it("hangs under the anchor", () => {
    expect(popoverPosition({ left: 40, top: 60, bottom: 100 }, size, view)).toEqual({ left: 40, top: 106 });
  });

  it("stays inside the window on the right", () => {
    expect(popoverPosition({ left: 900, top: 60, bottom: 100 }, size, view).left).toBe(712);
  });

  it("opens above when there is no room below", () => {
    expect(popoverPosition({ left: 40, top: 500, bottom: 540 }, size, view).top).toBe(134);
  });

  it("never leaves the top edge", () => {
    expect(popoverPosition({ left: 40, top: 100, bottom: 640 }, size, view).top).toBe(8);
  });
});
