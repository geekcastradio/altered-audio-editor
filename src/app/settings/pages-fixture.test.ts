// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Reads the same fixture as src-tauri/tests/settings_draft_fixture.rs.
import { describe, expect, it } from "vitest";
import raw from "../../../shared-fixtures/settings-draft.json";
import { cleanDraft, sizeText } from "./pages";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const fx = raw as any;
const num = (v: unknown) => (v === "NaN" ? NaN : v === "Infinity" ? Infinity : v === "-Infinity" ? -Infinity : v === "__undefined" ? undefined : v);
const J = (v: unknown) => JSON.parse(JSON.stringify(v ?? null));

describe("settings draft fixture", () => {
  it("cleans every draft the same as Rust", () => {
    for (const c of fx.drafts) {
      const d = JSON.parse(JSON.stringify(c.in));
      d.record.offsetMs = num(d.record.offsetMs);
      d.record.prerollSec = num(d.record.prerollSec);
      expect(J(cleanDraft(d))).toEqual(c.out);
    }
  });
  it("writes sizes the same as Rust", () => {
    for (const c of fx.sz) expect(sizeText(c.b)).toBe(c.out);
  });
});
