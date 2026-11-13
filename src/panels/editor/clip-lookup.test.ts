// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import type { ClipDoc, SessionDoc, TrackDoc } from "../../stores/session";
import { clipById } from "./clip-lookup";

const clip = (id: string) => ({ id, name: id }) as unknown as ClipDoc;
const track = (id: string, clips: ClipDoc[]) => ({ id, name: id, clips }) as unknown as TrackDoc;
const doc = { tracks: [track("A", [clip("a1")]), track("B", [clip("b1"), clip("b2")])] } as unknown as SessionDoc;

describe("clipById", () => {
  it("finds a clip and its track", () => {
    expect(clipById(doc, "b2")).toEqual({ clip: clip("b2"), trackId: "B" });
    expect(clipById(doc, "a1")?.trackId).toBe("A");
  });

  it("returns null for an empty id, a missing clip or no session", () => {
    expect(clipById(doc, null)).toBeNull();
    expect(clipById(doc, "")).toBeNull();
    expect(clipById(doc, "zzz")).toBeNull();
    expect(clipById(null, "a1")).toBeNull();
  });
});
