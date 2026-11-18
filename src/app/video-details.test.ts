// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { describeVideo } from "./video";

const meta = { width: 1920, height: 1080, fps: 29.97, duration: 60, has_audio: true, codec: "H.264" };

describe("describeVideo", () => {
  it("names the codec, size and rate, and says software when no hardware decoder is in use", () => {
    expect(describeVideo(meta, null)).toBe("H.264, 1920x1080 at 29.97 fps | software decoding");
  });

  it("adds how decoding is going once pictures have come", () => {
    const line = describeVideo(meta, { frames: 120, slow: 3, average_ms: 12.4, max_ms: 40.2, hardware: true });
    expect(line).toContain("12 ms per picture (slowest 40)");
    expect(line).toContain("3 of 120 slower than one frame");
    expect(line).toContain("hardware decoder in use");
  });

  it("copes with an unknown codec and rate", () => {
    expect(describeVideo({ ...meta, codec: "", fps: 0 }, null)).toBe("1920x1080 | software decoding");
    expect(describeVideo({ width: 640, height: 360, fps: 0, duration: 0, has_audio: false }, null)).toBe(
      "640x360 | software decoding",
    );
  });
});
