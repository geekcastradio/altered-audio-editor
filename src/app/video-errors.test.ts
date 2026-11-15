// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { decodePicture, errorText } from "./video";
import { isVideoFile } from "./session-files";

function picture(): ArrayBuffer {
  const b = new ArrayBuffer(16 + 8);
  const v = new DataView(b);
  v.setUint32(0, 2, true);
  v.setUint32(4, 1, true);
  v.setFloat64(8, 1.25, true);
  new Uint8Array(b, 16).set([1, 2, 3, 255, 4, 5, 6, 255]);
  return b;
}

describe("decodePicture accepts every form the bridge may hand back", () => {
  it("reads an ArrayBuffer", () => expect(decodePicture(picture())?.width).toBe(2));
  it("reads a typed array", () => expect(decodePicture(new Uint8Array(picture()))?.time).toBe(1.25));
  it("reads a plain number array", () => expect(decodePicture(Array.from(new Uint8Array(picture())))?.rgba[4]).toBe(4));
  it("reads a view into a larger buffer", () => {
    const big = new Uint8Array(40);
    big.set(new Uint8Array(picture()), 8);
    expect(decodePicture(new Uint8Array(big.buffer, 8, 24))?.height).toBe(1);
  });
  it("says null for anything else", () => expect(decodePicture("nope" as never)).toBeNull());
});

describe("errorText", () => {
  it("keeps a text error as it is", () => expect(errorText("Video: no picture.", "x")).toBe("Video: no picture."));
  it("reads the message of an Error or an error-like object", () => {
    expect(errorText(new Error("boom"), "x")).toBe("boom");
    expect(errorText({ message: "from the bridge" }, "x")).toBe("from the bridge");
  });
  it("falls back, keeping the detail when there is any", () => {
    expect(errorText(undefined, "Could not read a picture.")).toBe("Could not read a picture.");
    expect(errorText({ code: 7 }, "Could not read a picture.")).toBe('Could not read a picture. ({"code":7})');
  });
});

describe("isVideoFile", () => {
  it("is true for files that may carry a picture, in any case", () => {
    expect(isVideoFile("C:\\x\\ep.MP4")).toBe(true);
    expect(isVideoFile("/x/a.mov")).toBe(true);
  });
  it("is false for audio and unknown types", () => {
    expect(isVideoFile("/x/a.flac")).toBe(false);
    expect(isVideoFile("/x/notes.txt")).toBe(false);
  });
});
