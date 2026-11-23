// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Reads the same fixture as src-tauri/tests/shortcut_import_fixture.rs.
import { describe, expect, it } from "vitest";
import raw from "../../shared-fixtures/shortcut-import.json";
import { applyAudition, parseAudition } from "./audition-import";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const fx = raw as any;

describe("shortcut import fixture", () => {
  it("reads every export the same as Rust", () => {
    expect(fx.parse.length).toBeGreaterThan(150);
    for (const c of fx.parse) {
      let out: unknown;
      try {
        const r = parseAudition(c.xml);
        out = { bindings: r.bindings, unmapped: r.unmapped, unreadable: r.unreadable };
      } catch {
        out = { error: true };
      }
      expect(out).toEqual(c.out);
    }
  });
  it("applies every import the same as Rust", () => {
    for (const c of fx.apply) {
      const r = applyAudition(c.imported, c.current, fx.defaults);
      const key = (t: { action: string; chord: string }) => `${t.action}\u0000${t.chord}`;
      expect(r.overrides).toEqual(c.out.overrides);
      expect([...r.takenFrom].sort((a, b) => key(a).localeCompare(key(b)))).toEqual([...c.out.takenFrom].sort((a: never, b: never) => key(a).localeCompare(key(b))));
    }
  });
});
