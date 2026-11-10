// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { clearIn, clearOut, markIn, markOut } from "./in-out";

describe("In and Out points", () => {
  it("In with no selection runs to the end of the content", () => {
    expect(markIn(null, 100, 1000)).toEqual({ start: 100, end: 1000 });
  });

  it("In keeps an Out that is still ahead of it", () => {
    expect(markIn({ start: 0, end: 500 }, 200, 1000)).toEqual({ start: 200, end: 500 });
  });

  it("In past the old Out falls back to the end of the content", () => {
    expect(markIn({ start: 0, end: 500 }, 700, 1000)).toEqual({ start: 700, end: 1000 });
  });

  it("In at the very end leaves nothing selected", () => {
    expect(markIn(null, 1000, 1000)).toBeNull();
  });

  it("Out with no selection starts at zero", () => {
    expect(markOut(null, 400)).toEqual({ start: 0, end: 400 });
  });

  it("Out keeps an In that is still behind it", () => {
    expect(markOut({ start: 200, end: 900 }, 600)).toEqual({ start: 200, end: 600 });
  });

  it("Out before the old In falls back to zero", () => {
    expect(markOut({ start: 500, end: 900 }, 300)).toEqual({ start: 0, end: 300 });
  });

  it("Out at zero leaves nothing selected", () => {
    expect(markOut(null, 0)).toBeNull();
  });

  it("clearing the In moves the start back to zero", () => {
    expect(clearIn({ start: 200, end: 600 }, 1000)).toEqual({ start: 0, end: 600 });
  });

  it("clearing the Out moves the end back to the content end", () => {
    expect(clearOut({ start: 200, end: 600 }, 1000)).toEqual({ start: 200, end: 1000 });
  });

  it("clearing the only mark clears the selection", () => {
    expect(clearIn({ start: 200, end: 1000 }, 1000)).toBeNull();
    expect(clearOut({ start: 0, end: 600 }, 1000)).toBeNull();
    expect(clearIn(null, 1000)).toBeNull();
    expect(clearOut(null, 1000)).toBeNull();
  });
});
