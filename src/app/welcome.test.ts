// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { WELCOME_PAGES, WELCOME_REV, welcomePlan, type WelcomePage } from "./welcome";

const pages: WelcomePage[] = [
  { rev: 0, title: "intro", body: [] },
  { rev: 1, title: "a", body: [] },
  { rev: 2, title: "b", body: [] },
];

describe("welcomePlan", () => {
  it("shows everything on a fresh install", () => {
    expect(welcomePlan(undefined, pages)).toEqual({ kind: "install", pages });
    expect(welcomePlan(null, pages)?.kind).toBe("install");
    expect(welcomePlan({ welcomeSeen: false }, pages)?.kind).toBe("install");
    expect(welcomePlan({ welcomeRev: 0 }, pages)?.pages).toHaveLength(3);
  });

  it("shows only newer pages after an upgrade, once", () => {
    expect(welcomePlan({ welcomeRev: 1 }, pages)).toEqual({ kind: "upgrade", pages: [pages[2]] });
    expect(welcomePlan({ welcomeRev: 2 }, pages)).toBeNull();
  });

  it("treats settings from before the welcome, and the old flag, as revision 1", () => {
    expect(welcomePlan({ musicMode: true }, pages)?.pages).toEqual([pages[2]]);
    expect(welcomePlan({ welcomeSeen: true }, pages)?.pages).toEqual([pages[2]]);
  });

  it("never shows the install-only page on an upgrade", () => {
    for (const rev of [1, 2]) expect(welcomePlan({ welcomeRev: rev }, pages)?.pages.some((p) => p.rev === 0) ?? false).toBe(false);
  });

  it("keeps the revision in step with the newest page", () => {
    expect(WELCOME_REV).toBe(Math.max(...WELCOME_PAGES.map((p) => p.rev)));
  });
});
