// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// The core reads the built-in Voice Chain presets from src-tauri/data/voice-presets.json. This keeps
// that file equal to the list in chain.ts. To change a preset, change chain.ts, then write
// { builtin: BUILTIN_PRESETS, info: PRESET_INFO, colors: CATEGORY_COLORS } to the file as JSON.
import { describe, expect, it } from "vitest";
import presets from "../../../src-tauri/data/voice-presets.json?raw";
import { BUILTIN_PRESETS, CATEGORY_COLORS, PRESET_INFO } from "./chain";

describe("built-in Voice Chain presets data", () => {
  it("is the same as the list in chain.ts", () => {
    const file = JSON.parse(presets);
    expect(file.builtin).toEqual(JSON.parse(JSON.stringify(BUILTIN_PRESETS)));
    expect(file.info).toEqual(PRESET_INFO);
    expect(file.colors).toEqual(CATEGORY_COLORS);
  });
});
