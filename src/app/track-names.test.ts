// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { trackNameFromFile } from "./track-names";

describe("trackNameFromFile", () => {
  it("drops the extension and a recorder's speaker number", () => {
    expect(trackNameFromFile("1-host.flac")).toBe("host");
    expect(trackNameFromFile("C:\\calls\\12-guest_two.ogg")).toBe("guest two");
    expect(trackNameFromFile("2024 Interview.wav")).toBe("2024 Interview");
    expect(trackNameFromFile("3-.wav")).toBe("3-");
  });
});
