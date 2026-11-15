// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import type { SessionRow } from "../files/session-row";
import { neighborOf, showTabs, tabLabel } from "./session-tabs";

const row = (id: string, over: Partial<SessionRow> = {}): SessionRow => ({ id, name: id, dirty: false, seconds: 0, sampleRate: 48000, active: false, front: false, ...over });

describe("session tabs", () => {
  it("show only with two or more sessions open", () => {
    expect(showTabs([])).toBe(false);
    expect(showTabs([row("a")])).toBe(false);
    expect(showTabs([row("a"), row("b")])).toBe(true);
  });
  it("close to the right neighbor, else the left", () => {
    const rows = [row("a"), row("b"), row("c")];
    expect(neighborOf(rows, "b")).toBe("c");
    expect(neighborOf(rows, "c")).toBe("b");
    expect(neighborOf([row("a")], "a")).toBeNull();
    expect(neighborOf(rows, "zzz")).toBeNull();
  });
  it("mark unsaved sessions with a star", () => {
    expect(tabLabel(row("Ep 12"))).toBe("Ep 12");
    expect(tabLabel(row("Ep 12", { dirty: true }))).toBe("Ep 12 *");
  });
});
