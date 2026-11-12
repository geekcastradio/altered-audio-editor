// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { askWidth, decodePicture, needsFrame, timecode, videoTime } from "./video";

function bytes(w: number, h: number, time: number, rgba: number[]): ArrayBuffer {
  const b = new ArrayBuffer(16 + rgba.length);
  const v = new DataView(b);
  v.setUint32(0, w, true);
  v.setUint32(4, h, true);
  v.setFloat64(8, time, true);
  new Uint8Array(b, 16).set(rgba);
  return b;
}

describe("decodePicture", () => {
  it("reads the header and the pixels", () => {
    const p = decodePicture(bytes(2, 1, 1.25, [1, 2, 3, 255, 4, 5, 6, 255]));
    expect(p).not.toBeNull();
    expect(p?.width).toBe(2);
    expect(p?.height).toBe(1);
    expect(p?.time).toBe(1.25);
    expect(Array.from(p?.rgba ?? [])).toEqual([1, 2, 3, 255, 4, 5, 6, 255]);
  });
  it("refuses a short or inconsistent buffer", () => {
    expect(decodePicture(new ArrayBuffer(8))).toBeNull();
    expect(decodePicture(bytes(2, 2, 0, [0, 0, 0, 255]))).toBeNull();
    expect(decodePicture(bytes(0, 1, 0, []))).toBeNull();
  });
});

describe("videoTime", () => {
  it("is null before the video starts and after it ends", () => {
    expect(videoTime(1, 2, 10)).toBeNull();
    expect(videoTime(13, 2, 10)).toBeNull();
    expect(videoTime(5, 2, 10)).toBe(3);
  });
  it("handles a negative offset and an unknown length", () => {
    expect(videoTime(0, -4, 10)).toBe(4);
    expect(videoTime(500, 0, 0)).toBe(500);
    expect(videoTime(Number.NaN, 0, 10)).toBeNull();
  });
});

describe("timecode", () => {
  it("counts frames inside the second", () => {
    expect(timecode(0, 25)).toBe("00:00:00:00");
    expect(timecode(1.04, 25)).toBe("00:00:01:01");
    expect(timecode(3661.5, 30)).toBe("01:01:01:15");
  });
  it("falls back to 25 fps and never goes negative", () => {
    expect(timecode(-3, 0)).toBe("00:00:00:00");
    expect(timecode(2, 0)).toBe("00:00:02:00");
  });
});

describe("needsFrame", () => {
  it("fetches on the first picture, on a frame step, and on leaving the video", () => {
    expect(needsFrame(null, 1, 25)).toBe(true);
    expect(needsFrame(1, 1.001, 25)).toBe(false);
    expect(needsFrame(1, 1.04, 25)).toBe(true);
    expect(needsFrame(1, null, 25)).toBe(true);
    expect(needsFrame(null, null, 25)).toBe(false);
  });
});

describe("askWidth", () => {
  it("scales to the panel but stays in range", () => {
    expect(askWidth(400, 2, 3840)).toBe(800);
    expect(askWidth(50, 1, 3840)).toBe(160);
    expect(askWidth(4000, 2, 3840)).toBe(1920);
    expect(askWidth(400, 2, 320)).toBe(320);
  });
});
