// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Checks the TypeScript library view and Files list rules against the same fixture the Rust
// test reads (shared-fixtures/library-view.json).
import { describe, expect, it } from "vitest";
import fx from "../../shared-fixtures/library-view.json";
import * as L from "../panels/libraries/arrange";
import * as F from "../panels/files/format";

const items = fx.inputs[0]!.items as never[];
const bins = (bi: number) => fx.inputs[0]!.bins[bi] as never;
const V = (v: unknown) => v as L.ViewState;
const R = (r: unknown) => r as never[];

describe("library view fixture", () => {
  it("saved views", () => {
    for (const c of fx.view) expect(L.sanitizeView(c.raw)).toEqual(c.out);
  });
  it("sections and folding", () => {
    for (const c of fx.arrange) {
      const got = L.arrange({ collections: bins(c.bi) }, items, V(c.v));
      expect(
        got.map((s) => ({
          id: s.id,
          name: s.name,
          color: s.color,
          bin: s.bin,
          items: s.items.map((i) => i.id),
        })),
      ).toEqual(c.out);
    }
    for (const c of fx.fold) {
      const v = V(c.v);
      const sections = L.arrange({ collections: bins(c.bi) }, items, v);
      expect(L.flatIds(sections, c.collapsed)).toEqual(c.flat);
      expect([
        L.foldedIds(sections, v, false),
        L.foldedIds(sections, v, true),
      ]).toEqual(c.folded);
      expect(
        sections
          .slice(0, 2)
          .map((s) => [
            L.toggleSection(v, s.id, true),
            L.toggleSection(v, s.id, false),
          ]),
      ).toEqual(c.toggles);
    }
  });
  it("files list", () => {
    for (const c of fx.rows)
      expect(
        F.sortItems(R(c.rows), c.key as F.SortKey | null, c.desc).map(
          (r: { path: string }) => r.path,
        ),
      ).toEqual(c.out);
    for (const c of fx.match)
      expect(
        R(fx.rows[0]!.rows)
          .filter((r) => F.matches(r, c.q))
          .map((r: { path: string }) => r.path),
      ).toEqual(c.out);
  });
});
