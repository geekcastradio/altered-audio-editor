// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * Effects > Pitch and Time > Pitch Tune and Pitch Curve: the choices the
 * dialog offers, and the edit sent to the engine for them. Pure, so it
 * can be tested.
 */

export type PitchMode = "tune" | "bend";
export type ScaleId = "chromatic" | "major" | "minor" | "majorPentatonic" | "minorPentatonic";

export const NOTES = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"] as const;
export const SCALES: { id: ScaleId; label: string }[] = [
  { id: "chromatic", label: "Chromatic (every note)" },
  { id: "major", label: "Major" },
  { id: "minor", label: "Minor" },
  { id: "majorPentatonic", label: "Major pentatonic" },
  { id: "minorPentatonic", label: "Minor pentatonic" },
];

/** The most the pitch may be moved, in cents. Matches the engine. */
export const MAX_CENTS = 700;

export interface PitchChoices {
  mode: PitchMode;
  key: number;
  scale: ScaleId;
  /** Percent of the way to the note that is corrected, 0 to 100. */
  strength: number;
  /** How fast the correction follows, in milliseconds. */
  speedMs: number;
  /** Notes further off than this are left alone, in cents. */
  maxCents: number;
  /** The bend at the start and the end of the selection, in cents. */
  bendStart: number;
  bendEnd: number;
  /** An optional extra point in the middle, in cents, or null. */
  bendMiddle: number | null;
}

export const DEFAULT_CHOICES: PitchChoices = {
  mode: "tune",
  key: 0,
  scale: "chromatic",
  strength: 100,
  speedMs: 40,
  maxCents: 200,
  bendStart: 0,
  bendEnd: 0,
  bendMiddle: null,
};

const num = (v: unknown, lo: number, hi: number, d: number): number => (typeof v === "number" && Number.isFinite(v) ? Math.min(hi, Math.max(lo, v)) : d);

/** Saved choices read back, keeping anything missing or out of range at its default. */
export function sanitizeChoices(raw: unknown): PitchChoices {
  const o = (raw && typeof raw === "object" ? raw : {}) as Record<string, unknown>;
  const d = DEFAULT_CHOICES;
  return {
    mode: o.mode === "bend" ? "bend" : "tune",
    key: Math.round(num(o.key, 0, 11, d.key)),
    scale: SCALES.some((s) => s.id === o.scale) ? (o.scale as ScaleId) : d.scale,
    strength: num(o.strength, 0, 100, d.strength),
    speedMs: num(o.speedMs, 0, 500, d.speedMs),
    maxCents: num(o.maxCents, 10, MAX_CENTS, d.maxCents),
    bendStart: num(o.bendStart, -MAX_CENTS, MAX_CENTS, d.bendStart),
    bendEnd: num(o.bendEnd, -MAX_CENTS, MAX_CENTS, d.bendEnd),
    bendMiddle: o.bendMiddle === null || o.bendMiddle === undefined ? null : num(o.bendMiddle, -MAX_CENTS, MAX_CENTS, 0),
  };
}

export type PitchEdit =
  | { kind: "tune"; key: number; scale: ScaleId; strength: number; speedMs: number; maxCents: number }
  | { kind: "bend"; points: [number, number][] };

/** The edit for these choices. */
export function editFor(c: PitchChoices): PitchEdit {
  if (c.mode === "tune") {
    return { kind: "tune", key: c.key, scale: c.scale, strength: c.strength / 100, speedMs: c.speedMs, maxCents: c.maxCents };
  }
  const points: [number, number][] = [[0, c.bendStart]];
  if (c.bendMiddle !== null) points.push([0.5, c.bendMiddle]);
  points.push([1, c.bendEnd]);
  return { kind: "bend", points };
}

/** True when the choices would change nothing, so there is no point running them. */
export function isNoChange(c: PitchChoices): boolean {
  if (c.mode === "tune") return c.strength <= 0;
  return [c.bendStart, c.bendEnd, c.bendMiddle ?? 0].every((v) => Math.abs(v) < 1);
}

/** What the engine found, as a sentence. */
export function describeReport(r: { frames: number; voiced: number; moved: number }): string {
  const share = r.frames > 0 ? Math.round((100 * r.voiced) / r.frames) : 0;
  if (r.moved === 0) return `Clear pitch was found in ${share} percent of the audio, but none of it needed to move.`;
  return `Clear pitch was found in ${share} percent of the audio, and the pitch was moved in ${Math.round((100 * r.moved) / Math.max(1, r.frames))} percent of it. The rest was left as it was.`;
}
