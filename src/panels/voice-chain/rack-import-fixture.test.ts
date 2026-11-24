// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import fixture from "../../../shared-fixtures/rack-import.json";
import { importAuditionRacks, readRacks } from "./audition-presets";

// The Rust reader checks the same file, so both sides stay the same.
describe("effect chain import, shared cases", () => {
  for (const c of fixture.cases) {
    it(c.name, () => {
      expect(JSON.parse(JSON.stringify(readRacks(c.xml)))).toEqual(c.racks);
      expect(JSON.parse(JSON.stringify(importAuditionRacks(c.xml)))).toEqual(c.imported);
    });
  }
});
