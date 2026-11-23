// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Checks the settings sanitizer in TypeScript against the fixture the Rust
// test reads too (shared-fixtures/settings-clean.json). Each case stores
// only the settings that differ from the defaults.
import { describe, expect, it } from "vitest";
import fx from "../../shared-fixtures/settings-clean.json";
import { cleanBindings } from "../app/controller/map";
import { DEFAULT_SETTINGS, sanitizeSettings } from "./settings";

const J = (v: unknown) => JSON.parse(JSON.stringify(v ?? null));
const def = J(DEFAULT_SETTINGS) as Record<string, unknown>;
const diff = (full: Record<string, unknown>) => Object.fromEntries(Object.entries(full).filter(([k, v]) => JSON.stringify(v) !== JSON.stringify(def[k])));

describe("settings fixture", () => {
  it("has the same defaults", () => {
    expect(J(DEFAULT_SETTINGS)).toEqual(fx.defaults);
  });
  it("sanitizes", () => {
    for (const c of fx.cases) expect(diff(J(sanitizeSettings(c.raw)))).toEqual(c.out);
  });
  it("cleans bindings", () => {
    for (const c of fx.bindings) expect(J(cleanBindings(c.raw))).toEqual(c.out);
  });
});
