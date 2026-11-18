// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { followEffect } from "./follow";

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
