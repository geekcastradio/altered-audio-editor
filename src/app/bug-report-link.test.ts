// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { osFrom, reportUrl } from "./bug-report-link";

const WIN = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0 Safari/537.36 Edg/130.0";
const MAC = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko)";
const LIN = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/605.1.15 (KHTML, like Gecko)";

describe("bug report link", () => {
  it("names the operating system from the user agent", () => {
    expect(osFrom(WIN)).toBe("Windows");
    expect(osFrom(MAC)).toBe("macOS");
    expect(osFrom(LIN)).toBe("Linux");
    expect(osFrom("something else")).toBeNull();
  });

  it("fills in the version and the system, and nothing else", () => {
    const u = new URL(reportUrl("0.18.0", WIN));
    expect(u.origin + u.pathname).toBe("https://github.com/geekcastradio/altered-audio-editor/issues/new");
    expect([...u.searchParams.keys()].sort()).toEqual(["app_version", "os", "template"]);
    expect(u.searchParams.get("template")).toBe("bug_report.yml");
    expect(u.searchParams.get("app_version")).toBe("0.18.0");
    expect(u.searchParams.get("os")).toBe("Windows");
  });

  it("leaves a field out when it is not known or does not look like a version", () => {
    expect(new URL(reportUrl(null, "something else")).searchParams.has("app_version")).toBe(false);
    expect(new URL(reportUrl(null, "something else")).searchParams.has("os")).toBe(false);
    expect(new URL(reportUrl("C:\\Users\\steve", MAC)).searchParams.has("app_version")).toBe(false);
  });
});
