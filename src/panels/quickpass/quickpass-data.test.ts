// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// The core reads the QuickPass scan types, defaults and built-in presets from
// src-tauri/data/quickpass.json. This keeps that file equal to the lists in quickpass.ts. To change
// one, change quickpass.ts, then write { scanTypes: SCAN_TYPES, defaults: DEFAULTS, presets: BUILTIN_PRESETS }
// to the file as JSON.
import { describe, expect, it } from "vitest";
import data from "../../../src-tauri/data/quickpass.json?raw";
import { BUILTIN_PRESETS, DEFAULTS, SCAN_TYPES } from "./quickpass";

describe("QuickPass data", () => {
  it("is the same as the lists in quickpass.ts", () => {
    const file = JSON.parse(data);
    expect(file.scanTypes).toEqual(JSON.parse(JSON.stringify(SCAN_TYPES)));
    expect(file.defaults).toEqual(DEFAULTS);
    expect(file.presets).toEqual(JSON.parse(JSON.stringify(BUILTIN_PRESETS)));
  });
});
