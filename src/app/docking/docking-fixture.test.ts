// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Reads the same fixture as src-tauri/tests/docking_fixture.rs, and checks that the data
// file Rust reads (src-tauri/data/docking.json) still matches the lists in registry.ts.
// To write the file again, save JSON.stringify of the object built in the first test below.
import { describe, expect, it } from "vitest";
import data from "../../../src-tauri/data/docking.json";
import raw from "../../../shared-fixtures/docking.json";
import * as R from "./registry";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const fx = raw as any;
const J = (v: unknown) => JSON.parse(JSON.stringify(v));

describe("docking fixture", () => {
  it("keeps the data file Rust reads in step with the lists here", () => {
    expect(J({ panels: R.PANELS, sizes: R.DEFAULT_SIZES, layout: R.DEFAULT_LAYOUT, layoutVersion: R.DEFAULT_LAYOUT_VERSION, presets: R.WORKSPACE_PRESETS, minSize: R.MIN_SIZE })).toEqual(data);
  });
  it("places panels the same as Rust", () => {
    for (const c of fx.homes) expect(J(R.resolveHome(c.id, new Set(c.open)))).toEqual(c.out);
  });
  it("names workspaces the same as Rust", () => {
    for (const c of fx.wids) expect(R.workspaceId(c.name, new Set(c.taken))).toBe(c.out);
  });
  it("sizes groups and finds panels and presets the same as Rust", () => {
    for (const c of fx.groups) expect(R.groupMin(c.ids)).toEqual(c.out);
    for (const c of fx.presets) expect(R.presetById(c.id) !== undefined).toBe(c.found);
    for (const c of fx.defs) expect(R.panelDef(c.id)?.title ?? null).toBe(c.title);
  });
});
