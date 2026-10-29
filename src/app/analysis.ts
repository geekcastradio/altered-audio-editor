// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/** What the core returns for Analysis > Frequency Analysis, Amplitude Statistics and Histogram. */
export interface ChannelStats {
  minSample: number;
  maxSample: number;
  peakDb: number;
  rmsDb: number;
  dcOffset: number;
  clipped: number;
}

export interface Problems {
  clicks: number;
  clickTimes: number[];
  clipRegions: number;
  clipSamples: number;
  clipTimes: number[];
  hum: { baseHz: number; strengthDb: number; tones: { hz: number; excessDb: number }[] } | null;
}

export interface AnalysisReport {
  seconds: number;
  sampleRate: number;
  left: ChannelStats;
  right: ChannelStats;
  minWindowDb: number;
  avgWindowDb: number;
  maxWindowDb: number;
  crestDb: number;
  silentSamples: number;
  /** Samples at each one-dB level: bin i holds levels in (-(i+1), -i] dBFS. */
  histogram: number[];
  spectrumHz: number[];
  spectrumDb: number[];
  problems: Problems;
  cancelled: boolean;
}

export type AnalysisTab = "frequency" | "statistics" | "histogram" | "problems";

/** "-12.3 dB", or a dash for silence and missing values. */
export function fmtDb(v: number | null | undefined, floor = -150): string {
  if (v === null || v === undefined || !Number.isFinite(v) || v <= floor) return "-inf dB";
  return `${v.toFixed(1)} dB`;
}

/** A sample value with its sign, six places. */
export function fmtSample(v: number): string {
  return v.toFixed(6);
}

/** Left edge of a point on a log frequency axis, 0 to 1. */
export function logX(hz: number, low = 20, high = 20000): number {
  const lo = Math.log(low);
  return Math.min(1, Math.max(0, (Math.log(Math.max(hz, low)) - lo) / (Math.log(high) - lo)));
}

/** Vertical position for a level between `top` and `bottom` dB, 0 (top) to 1 (bottom). */
export function dbY(db: number, top: number, bottom: number): number {
  return Math.min(1, Math.max(0, (top - db) / (top - bottom)));
}

/** Histogram bars scaled to the largest bin, 0 to 1, on a log scale so quiet detail still shows. */
export function histogramBars(bins: number[]): number[] {
  const top = Math.max(1, ...bins);
  return bins.map((n) => (n <= 0 ? 0 : Math.log10(1 + n) / Math.log10(1 + top)));
}

/** Level at the left edge of histogram bin i: 0 dB for the first. */
export function binDb(i: number): number {
  return -i;
}

/** The frequency labelled on the axis. */
export const FREQ_TICKS = [20, 50, 100, 200, 500, 1000, 2000, 5000, 10000, 20000];

export function fmtHz(hz: number): string {
  return hz >= 1000 ? `${hz / 1000}k` : `${hz}`;
}

/** Seconds as m:ss.mmm for a list of places to look. */
export function fmtAt(seconds: number): string {
  const ms = Math.round(Math.max(0, seconds) * 1000);
  const m = Math.floor(ms / 60000);
  const s = Math.floor((ms % 60000) / 1000);
  return `${m}:${String(s).padStart(2, "0")}.${String(ms % 1000).padStart(3, "0")}`;
}

export interface Finding {
  id: "clicks" | "clipping" | "hum" | "dc";
  /** False when nothing was found. */
  found: boolean;
  /** One line saying what was found, or that nothing was. */
  text: string;
  /** What to do about it, when something was found. */
  advice: string;
  /** Places to look, as m:ss.mmm. */
  at: string[];
}

const plural = (n: number, one: string, many = `${one}s`) => `${n.toLocaleString()} ${n === 1 ? one : many}`;

/** The click, clipping, hum and DC offset findings of a report, ready to show. */
export function findings(report: Pick<AnalysisReport, "problems" | "left" | "right">): Finding[] {
  const p = report.problems;
  const out: Finding[] = [];
  out.push(
    p.clicks === 0
      ? { id: "clicks", found: false, text: "No clicks found.", advice: "", at: [] }
      : {
          id: "clicks",
          found: true,
          text: `${plural(p.clicks, "click")} found${p.clicks > p.clickTimes.length ? `, the first ${p.clickTimes.length} listed` : ""}.`,
          advice: "Click Repair (Effects) fills clicks from the sound on each side, and QuickPass can repair them in a copy of the clip.",
          at: p.clickTimes.map(fmtAt),
        },
  );
  out.push(
    p.clipRegions === 0
      ? { id: "clipping", found: false, text: "No flattened peaks found.", advice: "", at: [] }
      : {
          id: "clipping",
          found: true,
          text: `${plural(p.clipRegions, "flattened peak")} (${plural(p.clipSamples, "sample")} at full scale)${p.clipRegions > p.clipTimes.length ? `, the first ${p.clipTimes.length} listed` : ""}.`,
          advice: "Clip Repair (Effects) and QuickPass rebuild flattened peaks from the shape on either side. Lower the clip gain a little afterward, and record lower next time.",
          at: p.clipTimes.map(fmtAt),
        },
  );
  const h = p.hum;
  out.push(
    h
      ? {
          id: "hum",
          found: true,
          text: `Possible ${h.baseHz} Hz mains hum: tones at ${h.tones.filter((t) => t.excessDb >= 8).map((t) => `${t.hz} Hz`).join(", ")}, the strongest ${h.strengthDb.toFixed(0)} dB above the sound around it.`,
          advice: `Hum Cut (Effects, Filter and EQ) set to ${h.baseHz} Hz removes it with narrow notches. Music with a steady bass note can look like hum, so listen first.`,
          at: [],
        }
      : { id: "hum", found: false, text: "No mains hum found.", advice: "", at: [] },
  );
  const dc = Math.max(Math.abs(report.left.dcOffset), Math.abs(report.right.dcOffset));
  out.push(
    dc > 0.005
      ? { id: "dc", found: true, text: `DC offset of ${dc.toFixed(4)} (a steady push off center).`, advice: "DC Offset Fix (Effects) removes it.", at: [] }
      : { id: "dc", found: false, text: "No DC offset.", advice: "", at: [] },
  );
  return out;
}
