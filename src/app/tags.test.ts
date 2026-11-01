// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { TAGGABLE, TAG_FIELDS, cleanTags, saveTags } from "./tags";

describe("file tags", () => {
  it("writes only the filled-in fields, trimmed", () => {
    expect(cleanTags({ title: "  Episode 12 ", artist: "", album: "   ", comment: "Notes\nline two" })).toEqual({ title: "Episode 12", comment: "Notes\nline two" });
    expect(cleanTags({})).toEqual({});
  });
  it("lists every field once and knows which files take tags", () => {
    expect(new Set(TAG_FIELDS.map((f) => f.key)).size).toBe(TAG_FIELDS.length);
    expect(TAGGABLE.test("a/b/Show.MP3")).toBe(true);
    expect(TAGGABLE.test("a/b/Show.ogg")).toBe(false);
  });
});

describe("saveTags", () => {
  it("sends blank fields as empty strings so they are removed", () => {
    const t = saveTags({ title: " A ", artist: "" });
    expect(t.title).toBe("A");
    expect(t.artist).toBe("");
    expect(t.album).toBe("");
  });
});
