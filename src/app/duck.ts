// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * Multitrack > Duck Music Under Voice: puts a Dynamics effect on a music
 * track, keyed by a voice track, so the music drops while the voice talks
 * and comes back after. It is nothing but plain Dynamics slots with a
 * sidechain, so everything can be edited afterward in the Voice Chain.
 * With more than one voice, each voice gets its own slot on the music
 * track (the reductions add up when two people talk at once).
 */

import type { EffectChainDoc, EffectSlotDoc, SessionDoc, TrackDoc } from "../stores/session";
import { isVoiceTrack } from "./speaker-balance";

/** The name stored on the slots this feature makes, so running it again replaces them. */
export const DUCK_PRESET = "Duck under voice";

export interface DuckDepth {
  id: string;
  label: string;
  /** How far the music drops while a voice speaks at a normal level, in dB. */
  db: number;
}

export const DUCK_DEPTHS: readonly DuckDepth[] = [
  { id: "light", label: "Light (6 dB)", db: 6 },
  { id: "medium", label: "Medium (10 dB)", db: 10 },
  { id: "deep", label: "Deep (15 dB)", db: 15 },
];

/** Where the detector starts to react. Room noise in a voice track sits well below this. */
export const DUCK_THRESHOLD_DB = -45;
/** A normal speaking level in the voice track (about -20 dBFS), in dB over the threshold, used to turn depth into a ratio. */
const NOMINAL_OVER_DB = 25;

/** The Dynamics settings that drop the music by about `db` while a voice speaks at a normal level. */
export function duckParams(db: number): Record<string, number> {
  const d = Math.max(1, Math.min(db, 18));
  const ratio = Math.min(20, Math.max(1, 1 / (1 - d / NOMINAL_OVER_DB)));
  return {
    threshold: DUCK_THRESHOLD_DB,
    ratio: Math.round(ratio * 10) / 10,
    attack: 30,
    release: 500,
    knee: 6,
    makeup: 0,
    linked: 1,
  };
}

/** One keyed Dynamics slot for one voice track. */
export function duckSlot(id: string, voiceId: string, db: number): EffectSlotDoc {
  return {
    id,
    effect: "altaudio.dynamics",
    enabled: true,
    params: duckParams(db),
    preset: DUCK_PRESET,
    sidechain: voiceId,
  };
}

/** True for a slot a previous run made. */
export const isDuckSlot = (s: EffectSlotDoc): boolean => s.preset === DUCK_PRESET && s.effect === "altaudio.dynamics" && !!s.sidechain;

/** Voice tracks that can key a duck on `music`: not the music itself. */
export function duckKeys(doc: SessionDoc, music: TrackDoc): TrackDoc[] {
  return doc.tracks.filter((t) => t.id !== music.id && isVoiceTrack(t));
}

/** True when the track already carries duck slots. */
export const hasDuck = (music: TrackDoc): boolean => (music.effects?.slots ?? []).some(isDuckSlot);

/**
 * The music track's chain with its old duck slots replaced by new ones
 * (one per voice), or null when the chain would pass 16 effects.
 */
export function withDuck(chain: EffectChainDoc, voiceIds: readonly string[], db: number, newId: () => string, max = 16): EffectChainDoc | null {
  const kept = chain.slots.filter((s) => !isDuckSlot(s));
  const added = voiceIds.map((v) => duckSlot(newId(), v, db));
  if (kept.length + added.length > max) return null;
  return { ...chain, slots: [...kept, ...added] };
}

/** The chain with every duck slot taken out. */
export function withoutDuck(chain: EffectChainDoc): EffectChainDoc {
  return { ...chain, slots: chain.slots.filter((s) => !isDuckSlot(s)) };
}
