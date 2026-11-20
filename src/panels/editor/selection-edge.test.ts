// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { anchorFor, edgeAt, EDGE_GRAB_PX } from "./selection-edge";

const view = { scroll: 1000, spp: 10 };
// Selection 2000..5000 sits at x = 100..400.
const sel = { start: 2000, end: 5000 };

describe("edgeAt", () => {
  it("finds the start and end borders within reach", () => {
    expect(edgeAt(sel, 100, view)).toBe("start");
    expect(edgeAt(sel, 96, view)).toBe("start");
    expect(edgeAt(sel, 104, view)).toBe("start");
    expect(edgeAt(sel, 400, view)).toBe("end");
    expect(edgeAt(sel, 405, view)).toBe("end");
  });

  it("is null away from the borders, including the middle of the selection", () => {
    expect(edgeAt(sel, 250, view)).toBeNull();
    expect(edgeAt(sel, 100 - EDGE_GRAB_PX - 1, view)).toBeNull();
    expect(edgeAt(sel, 400 + EDGE_GRAB_PX + 1, view)).toBeNull();
  });

  it("is null with no selection or an empty one", () => {
    expect(edgeAt(null, 100, view)).toBeNull();
    expect(edgeAt(undefined, 100, view)).toBeNull();
    expect(edgeAt({ start: 3000, end: 3000 }, 200, view)).toBeNull();
  });

  it("reach is in pixels, so it follows zoom", () => {
    const zoomed = { scroll: 1000, spp: 1 };
    // Same selection is now at x = 1000..4000; 100 is nowhere near it.
    expect(edgeAt(sel, 100, zoomed)).toBeNull();
    expect(edgeAt(sel, 1003, zoomed)).toBe("start");
  });

  it("picks the closer border on a very narrow selection", () => {
    const narrow = { start: 2000, end: 2040 }; // x = 100..104
    expect(edgeAt(narrow, 100, view)).toBe("start");
    expect(edgeAt(narrow, 104, view)).toBe("end");
    // Equidistant: the side the pointer is outside of is not ambiguous here, it is inside, so end.
    expect(edgeAt(narrow, 102, view)).toBe("end");
    expect(edgeAt(narrow, 97, view)).toBe("start");
    expect(edgeAt(narrow, 107, view)).toBe("end");
  });

  it("ignores a view with no zoom", () => {
    expect(edgeAt(sel, 100, { scroll: 0, spp: 0 })).toBeNull();
  });
});

describe("anchorFor", () => {
  it("anchors on the border that stays put", () => {
    expect(anchorFor(sel, "start")).toBe(5000);
    expect(anchorFor(sel, "end")).toBe(2000);
  });
});
