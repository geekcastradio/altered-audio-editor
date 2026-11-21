// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { followEffect, maxScroll } from "./follow";

const now = { spp: 512, scroll: 10_000 };

describe("followEffect", () => {
  it("lets a move of the playhead bring the following back", () => {
    expect(followEffect(now, { playhead: 5 })).toBe("release");
  });

  it("does not count a zoom, even when the zoom moves the scroll with it", () => {
    expect(followEffect(now, { spp: 1024 })).toBe("none");
    expect(followEffect(now, { spp: 1024, scroll: 4_000 })).toBe("none");
  });

  it("holds the following when the view is scrolled by hand", () => {
    expect(followEffect(now, { scroll: 20_000 })).toBe("hold");
  });

  it("ignores a change that leaves the view as it was", () => {
    expect(followEffect(now, { spp: 512, scroll: 10_000 })).toBe("none");
    expect(followEffect(now, {})).toBe("none");
  });
});

describe("maxScroll", () => {
  const base = { width: 1000, spp: 100, following: false };

  it("stops where the end of the file fills the view", () => {
    expect(maxScroll({ ...base, total: 500_000, liveEnd: null })).toBe(400_000);
  });

  it("never goes below zero for a file shorter than the view", () => {
    expect(maxScroll({ ...base, total: 10_000, liveEnd: null })).toBe(0);
  });

  it("lets following reach the end of a recording that is past the file's end", () => {
    // Recording into an empty file: the old limit was 0, so the view never moved.
    expect(maxScroll({ ...base, total: 0, liveEnd: 300_000, following: true })).toBe(298_000);
    expect(maxScroll({ ...base, total: 100_000, liveEnd: 300_000, following: true })).toBe(298_000);
  });

  it("lets a scroll by hand reach the end of the recording but not past it", () => {
    expect(maxScroll({ ...base, total: 100_000, liveEnd: 300_000 })).toBe(200_000);
  });

  it("keeps the file's end when the recording is shorter than the file", () => {
    expect(maxScroll({ ...base, total: 500_000, liveEnd: 100_000 })).toBe(400_000);
  });

  it("is the old limit when nothing is recording, even when following playback", () => {
    expect(maxScroll({ ...base, total: 500_000, liveEnd: null, following: true })).toBe(400_000);
  });
});
