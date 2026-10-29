// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { DEFAULT_FORMAT, depthsFor, describeFormat, describeSampleType, describeSettings, estimateBytes, kindOfPath, sanitizeFormat, sanitizeMemory, switchFormat } from "./output-format";

const src = { sampleRate: 48_000, channels: 2, bitDepth: 24 };

describe("output format", () => {
  it("describes what the file will be, like the core does", () => {
    expect(describeFormat(DEFAULT_FORMAT, src)).toBe("WAV 24-bit, 48000 Hz stereo");
    expect(describeFormat({ ...DEFAULT_FORMAT, kind: "mp3", sampleRate: 96_000 }, src)).toBe("MP3 192 kbps constant, 48000 Hz stereo");
    expect(describeFormat({ ...DEFAULT_FORMAT, kind: "flac", bitDepth: null }, { ...src, bitDepth: 32 })).toBe("FLAC 24-bit, 48000 Hz stereo");
    expect(describeFormat({ ...DEFAULT_FORMAT, kind: "mp3", mp3Mode: "vbr", channels: 1 }, src)).toBe("MP3 variable, quality 2, 48000 Hz mono");
  });

  it("keeps only valid saved settings", () => {
    expect(sanitizeFormat(null)).toEqual(DEFAULT_FORMAT);
    expect(sanitizeFormat({ kind: "flac", bitDepth: 32, sampleRate: 12_345, channels: 3, mp3Bitrate: 100, mp3Quality: 42 })).toEqual({
      ...DEFAULT_FORMAT,
      kind: "flac",
      mp3Quality: 9,
    });
    expect(sanitizeFormat({ kind: "mp3", mp3Mode: "vbr", mp3Bitrate: 320, sampleRate: 44_100 })).toMatchObject({ mp3Mode: "vbr", mp3Bitrate: 320, sampleRate: 44_100 });
  });

  it("offers the depths each format writes", () => {
    expect(depthsFor("wav").map((d) => d.value)).toEqual([4, 8, 16, 24, 32]);
    expect(describeFormat({ ...DEFAULT_FORMAT, bitDepth: 4 }, src)).toBe("WAV 4-bit IMA ADPCM, 48000 Hz stereo");
    expect(describeFormat({ ...DEFAULT_FORMAT, bitDepth: 8 }, src)).toBe("WAV 8-bit, 48000 Hz stereo");
    expect(estimateBytes({ ...DEFAULT_FORMAT, bitDepth: 4 }, 10, src)).toBe(48_000 * 2 * 0.5 * 10);
    expect(depthsFor("flac").map((d) => d.value)).toEqual([16, 24]);
    expect(depthsFor("mp3")).toEqual([]);
    expect(depthsFor("m4a")).toEqual([]);
    expect(describeFormat({ ...DEFAULT_FORMAT, kind: "m4a", aacBitrate: 160 }, { sampleRate: 96_000, channels: 6, bitDepth: 24 })).toBe("AAC 160 kbps, 48000 Hz stereo");
    expect(kindOfPath("Episode.M4A")).toBe("m4a");
    expect(sanitizeFormat({ kind: "m4a", aacBitrate: 999 }).aacBitrate).toBe(128);
  });

  it("estimates sizes and reads formats from names", () => {
    expect(estimateBytes({ ...DEFAULT_FORMAT, kind: "mp3" }, 60, src)).toBe(1_440_000);
    expect(estimateBytes(DEFAULT_FORMAT, 1, src)).toBe(288_000);
    expect(kindOfPath("C:\\a\\Ep 1.FLAC")).toBe("flac");
    expect(kindOfPath("ep.m4a")).toBe("m4a");
  });
});


describe("export dialog summaries and per-format memory", () => {
  it("describes the sample type as the file will have it", () => {
    expect(describeSampleType({ ...DEFAULT_FORMAT }, src)).toBe("48000 Hz, stereo, 24-bit");
    expect(describeSampleType({ ...DEFAULT_FORMAT, kind: "mp3", sampleRate: 96_000, channels: 1 }, src)).toBe("48000 Hz, mono");
    expect(describeSampleType({ ...DEFAULT_FORMAT, kind: "wav", bitDepth: 32 }, src)).toBe("48000 Hz, stereo, 32-bit float");
  });

  it("describes format settings, and says nothing for formats without any", () => {
    expect(describeSettings({ ...DEFAULT_FORMAT, kind: "mp3" })).toBe("MP3, 192 kbps, constant");
    expect(describeSettings({ ...DEFAULT_FORMAT, kind: "aac", aacBitrate: 96 })).toBe("Raw AAC, 96 kbps");
    expect(describeSettings({ ...DEFAULT_FORMAT, kind: "wav" })).toBeNull();
  });

  it("keeps settings per format", () => {
    const mp3 = { ...DEFAULT_FORMAT, kind: "mp3" as const, mp3Bitrate: 128, sampleRate: 44_100 };
    const a = switchFormat({}, mp3, "wav");
    expect(a.format.kind).toBe("wav");
    expect(a.format.mp3Bitrate).toBe(192);
    const wav = { ...a.format, bitDepth: 24 };
    const b = switchFormat(a.memory, wav, "mp3");
    expect(b.format.mp3Bitrate).toBe(128);
    expect(b.format.sampleRate).toBe(44_100);
    expect(switchFormat(b.memory, b.format, "wav").format.bitDepth).toBe(24);
  });

  it("cleans a stored memory and knows the raw AAC extension", () => {
    expect(sanitizeMemory(null)).toEqual({});
    const m = sanitizeMemory({ mp3: { kind: "wav", mp3Bitrate: 12345 }, nope: {} });
    expect(m.mp3?.kind).toBe("mp3");
    expect(m.mp3?.mp3Bitrate).toBe(192);
    expect(Object.keys(m)).toEqual(["mp3"]);
    expect(kindOfPath("a/b.AAC")).toBe("aac");
  });
});
