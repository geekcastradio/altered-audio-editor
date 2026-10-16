// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * Recording state for the transport bars. App owns the logic and sets
 * `toggle`; the bars only show the state and call it.
 */

import { writable } from "svelte/store";

export interface RecordingState {
  active: boolean;
  /** Seconds recorded so far. */
  seconds: number;
  /** Input peak, 0 to 1. */
  peak: number;
}

export const recording = writable<RecordingState>({ active: false, seconds: 0, peak: 0 });

/** Input level per track id (0 to 1, falling smoothly), while inputs are open. */
export const inputLevels = writable<Record<string, number>>({});

/** The waveform of takes in progress, drawn on their tracks while recording. */
export interface LiveTakes {
  /** Session sample where the takes start. */
  at: number;
  /** Device sample rate. */
  rate: number;
  /** Per track id: min and max per block of LIVE_BLOCK frames. */
  blocks: Record<string, [number, number][]>;
}
export const liveTakes = writable<LiveTakes | null>(null);

/** Starts or stops recording. Set by App. */
export const recordControl: { toggle: (() => void) | null } = { toggle: null };
