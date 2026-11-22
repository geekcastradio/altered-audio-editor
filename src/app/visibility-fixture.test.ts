// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// The TypeScript track visibility rules must agree with
// shared-fixtures/visibility.json, the same file the Rust tests read
// (src-tauri/tests/visibility_fixture.rs).

import { describe, expect, it } from "vitest";
import fixture from "../../shared-fixtures/visibility.json";
import { hasClipIn, hiddenNames, keepByPreset, sanitizePresets, showOnly, withPreset } from "../panels/editor/visibility";

describe("shared visibility fixture", () => {
  const tracks = fixture.showOnly.tracks.map(([id, hidden]) => ({ id: id as string, hidden: hidden as boolean, clips: [] }));
  it.each(fixture.showOnly.cases)("showOnly: $keep", (c) => {
    const got = showOnly(tracks, (t) => c.keep[tracks.indexOf(t)] as boolean).map((x) => [x.id, x.hidden]);
    expect(got).toEqual(c.want);
  });
  it("hiddenNames", () => {
    const t = fixture.hiddenNames.tracks.map(([name, hidden]) => ({ name: name as string, hidden: hidden as boolean }));
    expect(hiddenNames(t)).toEqual(fixture.hiddenNames.want);
  });
  it.each(fixture.keptByPreset)("keepByPreset: $hidden", (c) => {
    const keep = keepByPreset(c.hidden);
    for (const [name, want] of c.results) expect(keep({ name: name as string })).toBe(want);
  });
  it.each(fixture.withPreset)("withPreset: $preset.name", (c) => {
    expect(withPreset(c.list, c.preset)).toEqual(c.want);
  });
  it.each(fixture.sanitize)("sanitizePresets: $raw", (c) => {
    expect(sanitizePresets(c.raw)).toEqual(c.want);
  });
  it.each(fixture.hasClipIn)("hasClipIn: $start $end", (c) => {
    const spans = c.spans.map(([start, length]) => ({ start: start as number, length: length as number }));
    expect(hasClipIn(spans, c.start, c.end)).toBe(c.want);
  });
});
