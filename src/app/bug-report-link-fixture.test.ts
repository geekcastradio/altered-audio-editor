// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// The TypeScript bug report link must agree with shared-fixtures/bug-report-link.json,
// the same file the Rust tests read (src-tauri/tests/bug_report_link_fixture.rs).

import { describe, expect, it } from "vitest";
import fixture from "../../shared-fixtures/bug-report-link.json";
import { osFrom, reportUrl } from "./bug-report-link";

describe("shared bug report link fixture", () => {
  it("osFrom", () => {
    for (const c of fixture.osFrom)
      expect(osFrom(c.ua), JSON.stringify(c)).toBe(c.want);
  });
  it("reportUrl", () => {
    for (const c of fixture.reportUrl)
      expect(reportUrl(c.version, c.ua), JSON.stringify(c)).toBe(c.want);
  });
});
