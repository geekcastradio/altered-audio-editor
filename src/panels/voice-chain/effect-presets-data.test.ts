// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// The core reads the built-in single-effect presets from src-tauri/data/effect-presets.json. This keeps
// that file equal to BUILTIN_EFFECT_PRESETS in effect-presets.ts. To change a preset, change the
// TypeScript, then write { builtin: BUILTIN_EFFECT_PRESETS } to the file as JSON.
import { describe, expect, it } from "vitest";
import data from "../../../src-tauri/data/effect-presets.json?raw";
import { BUILTIN_EFFECT_PRESETS } from "./effect-presets";

describe("built-in effect presets data", () => {
  it("is the same as the list in effect-presets.ts", () => {
    expect(JSON.parse(data).builtin).toEqual(JSON.parse(JSON.stringify(BUILTIN_EFFECT_PRESETS)));
  });
});
