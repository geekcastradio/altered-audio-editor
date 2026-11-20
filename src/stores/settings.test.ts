// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { colorVars, DEFAULT_SETTINGS, meterPercent, sanitizeSettings, settings, zoomRatios } from "./settings";

describe("app settings", () => {
  it("keeps defaults for anything missing or wrong", () => {
    expect(sanitizeSettings(null)).toEqual(DEFAULT_SETTINGS);
    const s = sanitizeSettings({ zoomStep: 500, fadeShape: "wobbly", colors: { wave: "#ABCDEF", accent: "red" }, output: "  ", autosaveMinutes: 0, meterYellowDb: -10, meterRedDb: -20 });
    expect(s.zoomStep).toBe(100);
    expect(s.fadeShape).toBe("cosine");
    expect(s.colors).toEqual({ wave: "#abcdef", playhead: "", accent: "", laneSelected: "", spectrumLow: "", spectrumHigh: "", spectrumScheme: "", phaseLow: "", phaseHigh: "" });
    expect(s.output).toBeNull();
    expect(s.autosaveMinutes).toBe(1);
    expect(s.meterRedDb).toBe(-9);
    expect(s.musicMode).toBe(false);
    expect(s.returnOnStop).toBe(true);
    expect(sanitizeSettings({ returnOnStop: false }).returnOnStop).toBe(false);
    expect(sanitizeSettings({ returnOnStop: "no" }).returnOnStop).toBe(true);
    expect(DEFAULT_SETTINGS.scrubAudio).toBe(false);
    expect(sanitizeSettings({ scrubAudio: true }).scrubAudio).toBe(true);
    expect(sanitizeSettings({ scrubAudio: "yes" }).scrubAudio).toBe(false);
    expect(sanitizeSettings({ musicMode: true }).musicMode).toBe(true);
    expect(sanitizeSettings({ ampModelAlways: true }).ampModelAlways).toBe(true);
    expect(sanitizeSettings({}).ampModelAlways).toBe(false);
    expect(sanitizeSettings({ musicMode: "yes" }).musicMode).toBe(false);
  });

  it("puts each dropped file on its own track unless one after another was chosen", () => {
    expect(DEFAULT_SETTINGS.dropMany).toBe("tracks");
    expect(sanitizeSettings({}).dropMany).toBe("tracks");
    expect(sanitizeSettings({ dropMany: "sideways" }).dropMany).toBe("tracks");
    expect(sanitizeSettings({ dropMany: "sequence" }).dropMany).toBe("sequence");
    expect(sanitizeSettings({ dropMany: "tracks" }).dropMany).toBe("tracks");
  });

  it("turns settings into zoom ratios, colors and meter marks", () => {
    expect(zoomRatios(25)).toEqual({ zoomIn: 0.8, zoomOut: 1.25 });
    expect(colorVars({ wave: "#112233", playhead: "", accent: "", laneSelected: "", spectrumLow: "", spectrumHigh: "", spectrumScheme: "", phaseLow: "", phaseHigh: "" })).toEqual({ "--wave": "#112233" });
    expect(meterPercent(-12)).toBeCloseTo(80);
    expect(meterPercent(-3)).toBeCloseTo(95);
    let seen = 0;
    const off = settings.subscribe((s) => (seen = s.zoomStep));
    settings.set({ ...DEFAULT_SETTINGS, zoomStep: 40 });
    expect(seen).toBe(40);
    off();
    settings.set(DEFAULT_SETTINGS);
  });

  it("keeps playback speed off until it is turned on", () => {
    expect(DEFAULT_SETTINGS.playbackSpeed).toBe(false);
    expect(sanitizeSettings({ playbackSpeed: true }).playbackSpeed).toBe(true);
    expect(sanitizeSettings({ playbackSpeed: "yes" }).playbackSpeed).toBe(false);
  });
});

describe("stop at selection end setting", () => {
  it("is on by default, as in Audition, and keeps only booleans", () => {
    expect(DEFAULT_SETTINGS.stopAtSelectionEnd).toBe(true);
    expect(sanitizeSettings({ stopAtSelectionEnd: false }).stopAtSelectionEnd).toBe(false);
    expect(sanitizeSettings({ stopAtSelectionEnd: "no" }).stopAtSelectionEnd).toBe(true);
  });
});

describe("update notice setting", () => {
  it("is off by default, weekly, and keeps only the two known frequencies", () => {
    expect(DEFAULT_SETTINGS.updateCheck).toBe(false);
    expect(DEFAULT_SETTINGS.updateEvery).toBe("week");
    expect(DEFAULT_SETTINGS.updateAutoInstall).toBe(false);
    expect(sanitizeSettings({ updateAutoInstall: true }).updateAutoInstall).toBe(true);
    expect(sanitizeSettings({ updateAutoInstall: "yes" }).updateAutoInstall).toBe(false);
    expect(sanitizeSettings({ updateEvery: "launch" }).updateEvery).toBe("launch");
    expect(sanitizeSettings({ updateEvery: "daily" }).updateEvery).toBe("week");
    expect(sanitizeSettings({ updateEvery: 3 }).updateEvery).toBe("week");
  });
});

describe("buffer size setting", () => {
  it("defaults to the driver and keeps only sensible values", () => {
    expect(DEFAULT_SETTINGS.bufferFrames).toBe(0);
    expect(DEFAULT_SETTINGS.fillClickTexture).toBe("natural");
    expect(sanitizeSettings({ fillClickTexture: "smooth" }).fillClickTexture).toBe("smooth");
    expect(sanitizeSettings({ fillClickTexture: "natural" }).fillClickTexture).toBe("natural");
    expect(sanitizeSettings({ fillClickTexture: "grainy" }).fillClickTexture).toBe("natural");
    expect(sanitizeSettings({ fillClickTexture: 1 }).fillClickTexture).toBe("natural");
    expect(sanitizeSettings({ bufferFrames: 256 }).bufferFrames).toBe(256);
    expect(sanitizeSettings({ bufferFrames: -1 }).bufferFrames).toBe(0);
    expect(sanitizeSettings({ bufferFrames: 100_000 }).bufferFrames).toBe(0);
    expect(sanitizeSettings({ bufferFrames: "256" }).bufferFrames).toBe(0);
  });
});
