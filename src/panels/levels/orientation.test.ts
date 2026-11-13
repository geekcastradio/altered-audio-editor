// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { levelsOrientation } from "./orientation";

describe("levelsOrientation", () => {
  it("lies down in a wide, short panel", () => {
    expect(levelsOrientation(900, 128, "vertical")).toBe("horizontal");
  });

  it("stands up in a tall, narrow panel", () => {
    expect(levelsOrientation(120, 700, "horizontal")).toBe("vertical");
  });

  it("keeps its direction near square", () => {
    expect(levelsOrientation(300, 320, "horizontal")).toBe("horizontal");
    expect(levelsOrientation(300, 320, "vertical")).toBe("vertical");
  });

  it("keeps its direction before the panel has a size", () => {
    expect(levelsOrientation(0, 0, "vertical")).toBe("vertical");
    expect(levelsOrientation(Number.NaN, 200, "horizontal")).toBe("horizontal");
  });
});
