// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { WELCOME_PAGES } from "./welcome";

// The same lookup the Welcome dialog uses, so a missing picture fails here and not on screen.
const pictures = import.meta.glob("../assets/welcome/*.png", { eager: true, query: "?url", import: "default" }) as Record<
  string,
  string
>;

describe("welcome pictures", () => {
  it("has a picture file for every page that names one", () => {
    for (const page of WELCOME_PAGES) {
      if (!page.image) continue;
      expect(pictures[`../assets/welcome/${page.image}.png`], `${page.image}.png`).toBeTruthy();
    }
  });

  it("has no two pages with the same revision", () => {
    const revs = WELCOME_PAGES.map((p) => p.rev);
    expect(new Set(revs).size).toBe(revs.length);
  });
});
