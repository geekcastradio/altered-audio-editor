// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/** Choices and edit building for the Session Properties dialog. */

import type { Op, PanLawId, SessionDoc } from "../stores/session";

export const SAMPLE_RATES = [22_050, 32_000, 44_100, 48_000, 88_200, 96_000] as const;

export const BIT_DEPTHS: { value: number; label: string }[] = [
  { value: 16, label: "16-bit" },
  { value: 24, label: "24-bit" },
  { value: 32, label: "32-bit float" },
];

/** The ways a pan setting can divide a sound, for the Session Properties dialog. */
export const PAN_LAWS: { value: PanLawId; label: string; hint: string }[] = [
  { value: "constant_power", label: "-3 dB at center", hint: "A sound keeps the same loudness as it moves across the mix." },
  { value: "balance", label: "Balance (full level at center)", hint: "Centered plays both sides at full level; panning turns the far side down." },
];

/** What Check Loudness measures this session against. An empty id uses the choice in Settings. */
export const LOUDNESS_CHOICES: { id: string; label: string }[] = [
  { id: "", label: "Use my Settings choice" },
  { id: "podcast-stereo", label: "Podcast, stereo (-16 LUFS)" },
  { id: "podcast-mono", label: "Podcast, mono (-19 LUFS)" },
  { id: "spotify", label: "Louder platforms (-14 LUFS)" },
  { id: "broadcast", label: "Broadcast (your region)" },
  { id: "custom", label: "Custom (from Settings)" },
];

export interface SessionProperties {
  name: string;
  sampleRate: number;
  bitDepth: number;
  /** Left out by callers that do not change it. */
  panLaw?: PanLawId;
  /** Target id, or "" for the Settings choice. Left out by callers that do not change it. */
  loudnessTarget?: string;
}

export function currentProperties(doc: SessionDoc): SessionProperties {
  return {
    name: doc.name,
    sampleRate: doc.sample_rate,
    bitDepth: doc.bit_depth,
    panLaw: doc.pan_law ?? "balance",
    loudnessTarget: doc.loudness_target ?? "",
  };
}

/** The edit for the changed values, or null when nothing changed. */
export function propertiesOp(doc: SessionDoc, next: SessionProperties): Op | null {
  const ops: Op[] = [];
  const name = next.name.trim();
  const info: { op: "set_session_info"; name?: string; bit_depth?: number } = { op: "set_session_info" };
  if (name && name !== doc.name) info.name = name;
  if (next.bitDepth !== doc.bit_depth) info.bit_depth = next.bitDepth;
  if (info.name !== undefined || info.bit_depth !== undefined) ops.push(info);
  if (next.sampleRate !== doc.sample_rate) ops.push({ op: "set_sample_rate", sample_rate: next.sampleRate });
  if (next.panLaw !== undefined && next.panLaw !== (doc.pan_law ?? "balance")) ops.push({ op: "set_pan_law", pan_law: next.panLaw });
  if (next.loudnessTarget !== undefined && next.loudnessTarget !== (doc.loudness_target ?? "")) {
    ops.push({ op: "set_loudness_target", target: next.loudnessTarget || null });
  }
  if (ops.length === 0) return null;
  return ops.length === 1 ? ops[0]! : { op: "batch", ops };
}

/** Bytes for a mixdown of `samples` at the session rate. */
export function mixdownBytes(samples: number, bitDepth: number, channels = 2): number {
  return samples * channels * (bitDepth / 8);
}
