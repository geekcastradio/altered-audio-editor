// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// The core reads the Sound Types list from src-tauri/data/sound-types.json. This keeps that file
// equal to TYPES in soundtypes.ts. To change a type, change soundtypes.ts, then write { types: TYPES }
// to the file as JSON.
import { describe, expect, it } from "vitest";
import data from "../../../src-tauri/data/sound-types.json?raw";
import { TYPES } from "./soundtypes";

describe("Sound Types data", () => {
  it("is the same as the list in soundtypes.ts", () => {
    expect(JSON.parse(data).types).toEqual(JSON.parse(JSON.stringify(TYPES)));
  });
});
