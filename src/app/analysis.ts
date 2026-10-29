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
  cancelled: boolean;
}

export type AnalysisTab = "frequency" | "statistics" | "histogram";

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
