// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Reads the same fixture as src-tauri/tests/chain_run_notes_fixture.rs.
import { describe, expect, it } from "vitest";
import raw from "../../shared-fixtures/chain-run-notes.json";
import { chainSummary, preflight } from "./chains";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const fx = raw as any;

describe("chain run notices fixture", () => {
  it("explains why a chain cannot run the same as Rust", () => {
    for (const c of fx.pre) {
      const handlers = Object.fromEntries(c.handlers.map((h: string) => [h, { title: h, run: () => ({ op: null, message: "" }) }]));
      const loaded = { chain: { version: 1, name: "x", steps: c.steps.map((a: string) => ({ action: a })) }, problems: c.problems, resolved: [] };
      expect(preflight(loaded as never, handlers as never)).toEqual(c.out);
    }
  });
  it("sums up a run the same as Rust", () => {
    for (const c of fx.sum) expect(chainSummary(c.name, c.reports)).toBe(c.out);
  });
});
