// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// The TypeScript playlist and review rules must agree with
// shared-fixtures/playlist-review.json, the same file the Rust tests read
// (src-tauri/tests/playlist_review_fixture.rs).

import { describe, expect, it } from "vitest";
import fixture from "../../shared-fixtures/playlist-review.json";
import { available, exportNames, nextStart, playlistItems, withAdded, withMoved, withRemoved } from "../panels/playlist/playlist";
import { reviewCounts, reviewKind, reviewMarks, stepMark, type ReviewFilter } from "../panels/review/review";
import type { MarkerDoc, SessionDoc } from "../stores/session";

const markers = fixture.markers as unknown as MarkerDoc[];
const ids = (a: { id: string }[]) => a.map((x) => x.id);

describe("shared playlist fixture", () => {
  it.each(fixture.playlist)("playlist: $playlist", (c) => {
    const d = { markers, playlist: c.playlist } as unknown as SessionDoc;
    const items = playlistItems(d);
    expect(ids(items)).toEqual(c.items);
    expect(ids(available(d))).toEqual(c.available);
    expect(withAdded(d, ["a", "e", "e", "zzz"])).toEqual(c.added[0]);
    expect(withAdded(d, [])).toEqual(c.added[1]);
    ["a", "b", "nope"].forEach((id, i) => expect(withRemoved(d, id)).toEqual(c.removed[i]));
    for (const m of c.moved) expect(withMoved(d, m.id as string, m.by as 1 | -1)).toEqual(m.want);
    items.forEach((_, i) => expect(nextStart(items, i)).toEqual(c.next[i]));
    expect(exportNames(items)).toEqual(c.names);
  });
  it.each(fixture.exportNames)("exportNames: $names", (c) => {
    const ms = c.names.map((name, i) => ({ id: `x${i}`, name, position: i, duration: 1 })) as unknown as MarkerDoc[];
    expect(exportNames(ms)).toEqual(c.want);
  });
});

describe("shared review fixture", () => {
  it.each(fixture.reviewKinds)("reviewKind: $name", (c) => {
    expect(reviewKind({ name: c.name })).toBe(c.want);
  });
  it.each(fixture.reviewMarks)("reviewMarks: $filter", (c) => {
    expect(ids(reviewMarks(markers, c.filter as ReviewFilter))).toEqual(c.want);
  });
  it("counts and steps through marks", () => {
    expect(reviewCounts(markers)).toEqual(fixture.reviewCounts);
    const sorted = reviewMarks(markers);
    for (const c of fixture.stepMark) {
      const r = stepMark(sorted, c.at, c.dir as 1 | -1);
      expect(r ? sorted.indexOf(r) : null).toBe(c.want);
    }
  });
});
