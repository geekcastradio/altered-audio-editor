// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Checks the TypeScript marker and chapter export rules against the same fixture the Rust
// tests read (shared-fixtures/markers.json).
import { describe, expect, it } from "vitest";
import fx from "../../shared-fixtures/markers.json";
import * as M from "../panels/markers/markers";
import * as C from "../panels/markers/chapters";
import type { MarkerDoc, MarkerKind } from "../stores/session";

const all = fx.markers[0] as unknown as MarkerDoc[];
const pick = (...ids: string[]) => ids.map((i) => all.find((m) => m.id === i)!);
const sets: Record<string, MarkerDoc[]> = {
  all,
  none: [],
  one: pick("a"),
  few: pick("a", "b", "c"),
  twoAtOnce: [
    ...pick("d", "e"),
    { ...all[4]!, id: "k", name: "Marker 3", position: 1440000 },
  ],
};

describe("markers fixture", () => {
  it("kinds and new markers", () => {
    for (const c of fx.kinds) {
      const k = c.k as MarkerKind;
      expect([
        M.KIND_LABELS[k],
        M.KIND_COLORS[k],
        M.markerCss({ kind: k, color: null }),
        M.markerCss({ kind: k, color: "#123456" }),
        M.markerCss({ kind: k, color: "" }),
      ]).toEqual([c.label, c.color, c.css, c.cssOwn, c.cssEmpty]);
    }
    for (const c of fx.news) {
      const a = c.args as [string, string, number, number, MarkerKind];
      expect(M.newMarker(...a)).toEqual(c.m);
    }
  });
  it("picking and searching", () => {
    for (const c of fx.names)
      expect(M.nextMarkerName(sets[c.set]!, c.base)).toBe(c.out);
    for (const c of fx.picks)
      expect(
        M.pickedMarkers(sets[c.set]!, new Set(c.ids)).map((m) => m.id),
      ).toEqual(c.out);
    for (const c of fx.between)
      expect(M.markersBetween(sets[c.set]!, c.a, c.b)).toEqual(c.out);
    for (const c of fx.labels) expect(M.deleteLabel(c.n)).toBe(c.out);
    for (const c of fx.filters)
      expect(M.filterMarkers(sets[c.set]!, c.q).map((m) => m.id)).toEqual(
        c.out,
      );
    for (const c of fx.hits) {
      const ms = sets[c.set]!;
      const h = M.markerAt(ms, c.x, (v) => v / c.spp, c.tol);
      expect(h ? ms.indexOf(h) : null).toBe(c.out);
    }
  });
  it("ranges, kind changes and playlists", () => {
    for (const c of fx.ranges)
      expect(M.rangeLengthFor({ position: c.pos }, c.sel, c.rate)).toBe(c.out);
    for (const c of fx.kindChanges)
      expect(
        M.kindChange(
          {
            kind: "cue",
            duration: c.duration,
            contracted_duration: c.contracted,
          },
          c.kind as MarkerKind,
          c.rate,
        ),
      ).toEqual(c.out);
    for (const c of fx.playlists)
      expect(
        M.togglePlaylist(c.pl ?? undefined, { id: c.id, duration: c.d }),
      ).toEqual(c.out);
  });
  it("chapter lists and files", () => {
    for (const c of fx.chapters)
      expect(
        C.chaptersOf(sets[c.set]!, 48000, c.which as "chapters" | "all"),
      ).toEqual(c.out);
    for (const c of fx.renders) {
      const list = C.chaptersOf(
        sets[c.set]!,
        48000,
        c.which as "chapters" | "all",
      );
      expect(C.render(list, c.fmt as C.ChapterFormat)).toBe(c.text);
      expect(C.warnings(list, c.fmt as C.ChapterFormat)).toEqual(c.warnings);
    }
    for (const c of fx.embeds) {
      const range = c.range as [number, number] | null;
      expect(
        C.embedList(sets[c.set]!, 48000, range).map(
          ({ title, startMs, endMs, url }) => ({
            title,
            startMs,
            endMs,
            url: url ?? null,
          }),
        ),
      ).toEqual(c.chapters);
      expect(C.markerList(sets[c.set]!, 48000, range)).toEqual(c.all);
    }
  });
  it("urls, tips and clock", () => {
    for (const c of fx.urls) {
      if ("t" in c) expect(C.isChapterUrl(c.t as string)).toBe(c.ok);
      else
        expect(C.chapterUrl(all.find((m) => m.id === c.marker)!)).toBe(c.url);
    }
    for (const c of fx.tips) {
      if ("t" in c) expect(C.titleTips(c.t as string)).toEqual(c.out);
      else expect(C.clock(c.clock as number)).toBe(c.out);
    }
  });
});
