// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * Loudness targets and pass/fail for the Check Loudness card.
 *
 * Podcast targets follow common platform guidance: -16 LUFS integrated
 * for stereo, -19 LUFS for mono, true peaks at or below -1 dBTP. Spotify
 * normalizes podcasts nearer -14 LUFS; broadcast (EBU R128) uses -23.
 * Mastering louder than the target only gets turned down by the player
 * and costs dynamic range.
 */

export interface LoudnessReport {
  integrated: number;
  integratedMono: number;
  range: number;
  truePeak: number;
  samplePeak: number;
  seconds: number;
  cancelled: boolean;
}

export interface LoudnessTarget {
  id: string;
  name: string;
  lufs: number;
  /** Measure the mix folded to mono (for mono shows). */
  mono: boolean;
  /** Highest allowed true peak, dBTP. */
  peak: number;
}

export const TARGETS: readonly LoudnessTarget[] = [
  { id: "podcast-stereo", name: "Podcast, stereo (-16 LUFS)", lufs: -16, mono: false, peak: -1 },
  { id: "podcast-mono", name: "Podcast, mono (-19 LUFS)", lufs: -19, mono: true, peak: -1 },
  { id: "spotify", name: "Louder platforms (-14 LUFS)", lufs: -14, mono: false, peak: -1 },
  { id: "broadcast", name: "Broadcast, EBU R128 (-23 LUFS)", lufs: -23, mono: false, peak: -1 },
];

export const DEFAULT_TARGET = "podcast-stereo";

export function targetById(id: string): LoudnessTarget {
  return TARGETS.find((t) => t.id === id) ?? TARGETS[0]!;
}

/** Within this many LU of the target passes; within twice it is close. */
export const TOLERANCE_LU = 1;

export type Grade = "pass" | "close" | "fail";

export interface LoudnessVerdict {
  measured: number;
  /** Loudness minus target, LU. Positive is too loud. */
  offset: number;
  loudness: Grade;
  peak: Grade;
  /** Master gain change that would hit the target, dB. */
  gainToTarget: number;
  /** True when that gain would keep true peaks within the limit. */
  gainIsSafe: boolean;
  advice: string;
}

export function evaluate(r: LoudnessReport, t: LoudnessTarget): LoudnessVerdict {
  const measured = t.mono ? r.integratedMono : r.integrated;
  if (!Number.isFinite(measured)) {
    return { measured, offset: 0, loudness: "fail", peak: "pass", gainToTarget: 0, gainIsSafe: false, advice: "The mix is silent or too short to measure." };
  }
  const offset = Math.round((measured - t.lufs) * 10) / 10;
  const loudness: Grade = Math.abs(offset) <= TOLERANCE_LU ? "pass" : Math.abs(offset) <= 2 * TOLERANCE_LU ? "close" : "fail";
  const peak: Grade = r.truePeak <= t.peak ? "pass" : r.truePeak <= t.peak + 0.5 ? "close" : "fail";
  const gainToTarget = Math.round((t.lufs - measured) * 10) / 10;
  const peakAfter = r.truePeak + gainToTarget;
  const gainIsSafe = peakAfter <= t.peak;
  let advice: string;
  if (loudness === "pass" && peak === "pass") advice = "Ready to publish at this target.";
  else if (loudness !== "pass" && gainIsSafe)
    advice = `${gainToTarget > 0 ? "Raise" : "Lower"} the Master by ${Math.abs(gainToTarget).toFixed(1)} dB to hit the target.`;
  else if (loudness !== "pass")
    advice = `Raising the Master ${gainToTarget.toFixed(1)} dB would push peaks to ${peakAfter.toFixed(1)} dBTP. Limit the voice tracks harder first (a limiter around -3 to -6 dB on vocals), then check again.`;
  else advice = `Peaks reach ${r.truePeak.toFixed(1)} dBTP, over the ${t.peak} dBTP limit. Lower the loudest moments or limit them, then check again.`;
  return { measured, offset, loudness, peak, gainToTarget, gainIsSafe, advice };
}
