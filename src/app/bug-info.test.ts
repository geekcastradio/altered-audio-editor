// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { bugInfoText, recordError, recordedErrors, scrub } from "./bug-info";

describe("bug info", () => {
  it("hides the user's name in paths", () => {
    expect(scrub("at C:\\Users\\steve\\GCRN\\x.ts:1")).toBe("at <user>\\GCRN\\x.ts:1");
    expect(scrub("/home/steve/a.ts")).toBe("<user>/a.ts");
  });

  it("keeps the newest twenty errors, with stacks", () => {
    for (let i = 0; i < 25; i++) recordError(new Error(`boom ${i}`));
    const list = recordedErrors();
    expect(list.length).toBe(20);
    expect(list[19]!.message).toBe("boom 24");
    expect(list[0]!.stack).toContain("boom 5");
  });

  it("writes a report", () => {
    const text = bugInfoText({ appVersion: "0.9.7", system: "Windows", state: ["Editor: Waveform"], plugins: ["vst3:abc"] }, [{ at: "t", message: "m", stack: "s" }]);
    expect(text).toContain("Altered Audio Editor 0.9.7");
    expect(text).toContain("Plug-ins in use: vst3:abc");
    expect(text).toContain("[t] m");
    expect(bugInfoText({ appVersion: "1", system: "x", state: [], plugins: [] }, [])).toContain("No errors recorded");
  });
});
