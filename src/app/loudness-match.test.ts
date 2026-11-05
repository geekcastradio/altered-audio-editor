// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { targetById, type LoudnessReport } from "./loudness";
import type { AnalysisReport } from "./analysis";
import { changeFor, levelRow, levelsOf, matchChanges, matchItems, matchRow, truePeakRow } from "./loudness-match";
import type { SessionDoc } from "../stores/session";

const rep = (i: number, tp: number, mono = i - 3): LoudnessReport => ({ integrated: i, integratedMono: mono, range: 5, truePeak: tp, samplePeak: tp, seconds: 60, cancelled: false });
const doc = {
  tracks: [
    { id: "h", name: "Host", muted: false, clips: [{ id: "a", name: "a", gain_db: 0 }, { id: "b", name: "b", gain_db: -2 }] },
    { id: "g", name: "Guest", muted: false, clips: [{ id: "c", name: "c", gain_db: 20 }] },
    { id: "e", name: "Empty", muted: false, clips: [] },
  ],
} as unknown as SessionDoc;

describe("loudness match", () => {
  it("works out the gain to the target", () => {
    const r = matchRow({ key: "h", label: "Host", clipIds: ["a"] }, rep(-22.4, -6), targetById("podcast-stereo"));
    expect(r.change).toBe(6.4);
    expect(r.peakAfter).toBe(0.4);
    expect(r.peakWarning).toBe(true);
  });
  it("uses the mono reading for mono targets", () => {
    const r = matchRow({ key: "h", label: "Host", clipIds: ["a"] }, rep(-16, -10, -20), targetById("podcast-mono"));
    expect(r.change).toBe(1);
  });
  it("skips silence", () => {
    const r = matchRow({ key: "h", label: "Host", clipIds: ["a"] }, rep(-Infinity, -Infinity), targetById("podcast-stereo"));
    expect(r.measurable).toBe(false);
    expect(matchChanges(doc, [r])).toEqual([]);
  });
  it("moves every clip on a track together and keeps gain in range", () => {
    const t = targetById("podcast-stereo");
    const host = matchRow({ key: "h", label: "Host", clipIds: ["a", "b"] }, rep(-19, -9), t);
    const guest = matchRow({ key: "g", label: "Guest", clipIds: ["c"] }, rep(-26, -20), t);
    expect(matchChanges(doc, [host, guest])).toEqual([
      { id: "a", gain_db: 3 },
      { id: "b", gain_db: 1 },
      { id: "c", gain_db: 24 },
    ]);
  });
  it("measures selected clips one by one, otherwise each track with clips", () => {
    expect(matchItems(doc, new Set(["c"])).items.map((i) => i.key)).toEqual(["c"]);
    const all = matchItems(doc, new Set());
    expect(all.mode).toBe("tracks");
    expect(all.items.map((i) => i.key)).toEqual(["h", "g"]);
  });
  it("matches by the loudest sample or the average level", () => {
    const side = (peakDb: number, rmsDb: number) => ({ peakDb, rmsDb, minSample: 0, maxSample: 0, dcOffset: 0, clipped: 0 });
    const stats = (l: [number, number], r: [number, number]) => ({ left: side(...l), right: side(...r) }) as unknown as AnalysisReport;
    const item = { key: "a", label: "a", clipIds: ["a"] };
    // The louder channel sets the peak; the two RMS values combine as power.
    expect(levelsOf(stats([-6, -20], [-3, -20]))).toEqual({ peakDb: -3, rmsDb: expect.closeTo(-20, 5) });
    const p = levelRow(item, "peak", -1, stats([-6, -20], [-3, -20]));
    expect(p).toMatchObject({ measured: -3, change: 2, peakAfter: -1, peakWarning: false });
    const r = levelRow(item, "rms", -18, stats([-4, -24], [-4, -24]));
    expect(r.change).toBe(6);
    expect(r.peakAfter).toBe(2);
    expect(r.peakWarning).toBe(true);
    expect(levelRow(item, "peak", -1, stats([-200, -200], [-200, -200])).measurable).toBe(false);
  });
  it("matches by true peak", () => {
    const item = { key: "a", label: "a", clipIds: ["a"] };
    const r = truePeakRow(item, -1, rep(-20, -4));
    expect(r).toMatchObject({ measured: -4, change: 3, peakAfter: -1, measurable: true, peakWarning: false });
    expect(truePeakRow(item, 1, rep(-20, -4)).peakWarning).toBe(true);
    expect(truePeakRow(item, -1, rep(-Infinity, -Infinity)).measurable).toBe(false);
  });
  it("leaves items alone that are already within the tolerance", () => {
    expect(changeFor(-16, -16.4, 0.5)).toBe(0);
    expect(changeFor(-16, -16.5, 0.5)).toBe(0);
    expect(changeFor(-16, -16.6, 0.5)).toBe(0.6);
    expect(changeFor(-16, -16.1, 0)).toBe(0.1);
    expect(changeFor(-16, -17, -1)).toBe(1);
    const t = targetById("podcast-stereo");
    const near = matchRow({ key: "h", label: "Host", clipIds: ["a"] }, rep(-16.3, -6), t, 0.5);
    expect(near.change).toBe(0);
    expect(near.measurable).toBe(true);
    expect(matchChanges(doc, [near])).toEqual([]);
  });
});
