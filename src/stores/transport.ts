// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * Play, pause and stop, plus the live playhead and level meters.
 *
 * While playing, the core is asked for its position and peak levels
 * about 30 times a second. The playhead in the session view is only
 * written when playback stops, so playing never marks the session as
 * changed.
 */

import type { TransportStatus } from "../app/core";
import type { SessionStore } from "./session";

export interface TransportBackend {
  play(from: number): Promise<void>;
  stop(): Promise<number | null>;
  status(): Promise<TransportStatus>;
}

/** Meter level with a slow fall and a held peak, in 0..1 (above 1 means over). */
export interface MeterLevel {
  level: number;
  peak: number;
  /** True once any sample went over full scale, until reset. */
  over: boolean;
}

export interface TransportState {
  playing: boolean;
  /** Session samples. */
  position: number;
  master: [MeterLevel, MeterLevel];
  tracks: Record<string, [MeterLevel, MeterLevel]>;
  /** Track Voice Chain meters by track id: left in, right in, left out, right out. Only tracks whose chain does something. */
  chains: Record<string, [MeterLevel, MeterLevel, MeterLevel, MeterLevel]>;
}

const FALL_PER_TICK = 0.86;
const PEAK_HOLD_TICKS = 30;

const quiet = (): MeterLevel => ({ level: 0, peak: 0, over: false });

/** One meter reading folded into the last: slow fall, held peak. `hold` counts ticks left. */
export function foldMeter(prev: MeterLevel, reading: number, hold: number): { meter: MeterLevel; hold: number } {
  const level = Math.max(reading, prev.level * FALL_PER_TICK);
  let peak = prev.peak;
  if (reading >= peak) {
    peak = reading;
    hold = PEAK_HOLD_TICKS;
  } else if (hold > 0) {
    hold--;
  } else {
    peak = Math.max(level, peak * FALL_PER_TICK);
  }
  return { meter: { level, peak, over: prev.over || reading > 1 }, hold };
}

type Listener = (s: TransportState) => void;

export class TransportStore {
  private state: TransportState = { playing: false, position: 0, master: [quiet(), quiet()], tracks: {}, chains: {} };
  private readonly listeners = new Set<Listener>();
  private startedAt = 0;
  private timer: ReturnType<typeof setTimeout> | null = null;
  private holds = new Map<string, number>();
  private busy: Promise<unknown> = Promise.resolve();

  constructor(
    private readonly backend: TransportBackend,
    private readonly session: SessionStore,
    private readonly onError: (e: unknown) => void = console.error,
    private readonly tickMs = 33,
  ) {}

  subscribe(fn: Listener): () => void {
    this.listeners.add(fn);
    fn(this.state);
    return () => {
      this.listeners.delete(fn);
    };
  }

  get current(): TransportState {
    return this.state;
  }

  /** Plays from the playhead, or from `from` (session samples). */
  play(from?: number): Promise<void> {
    return this.serial(async () => {
      const at = Math.max(0, Math.round(from ?? this.session.current?.view.playhead ?? 0));
      await this.backend.play(at);
      if (!this.state.playing) this.startedAt = at;
      this.set({ playing: true, position: at });
      this.loop();
    });
  }

  /** Stops and leaves the playhead where playback reached. */
  pause(): Promise<void> {
    return this.serial(async () => {
      const pos = await this.backend.stop();
      this.halt(pos ?? this.state.position);
    });
  }

  /** Stops and puts the playhead back where playback started. */
  stop(): Promise<void> {
    return this.serial(async () => {
      const was = this.state.playing;
      await this.backend.stop();
      this.halt(was ? this.startedAt : (this.session.current?.view.playhead ?? 0));
    });
  }

  toggle(): Promise<void> {
    return this.state.playing ? this.pause() : this.play();
  }

  /** Moves playback to a new spot while playing, or the playhead while stopped. */
  seek(to: number): Promise<void> {
    if (this.state.playing) return this.play(to);
    const v = this.session.current?.view;
    if (v) this.session.setView({ ...v, playhead: Math.max(0, Math.round(to)) });
    return Promise.resolve();
  }

  private halt(position: number): void {
    if (this.timer) clearTimeout(this.timer);
    this.timer = null;
    const v = this.session.current?.view;
    if (v) this.session.setView({ ...v, playhead: Math.max(0, Math.round(position)) });
    // Levels drop at once; over lights stay lit until clicked.
    const rest = (m: MeterLevel): MeterLevel => ({ level: 0, peak: 0, over: m.over });
    const tracks: Record<string, [MeterLevel, MeterLevel]> = {};
    for (const [id, [l, r]] of Object.entries(this.state.tracks)) if (l.over || r.over) tracks[id] = [rest(l), rest(r)];
    this.set({ playing: false, position, master: [rest(this.state.master[0]), rest(this.state.master[1])], tracks, chains: {} });
  }

  /** Clears the over lights of one track, or of the master with no id. */
  clearOver(trackId?: string): void {
    const off = (m: MeterLevel): MeterLevel => ({ ...m, over: false });
    if (trackId === undefined) {
      this.set({ master: [off(this.state.master[0]), off(this.state.master[1])] });
      return;
    }
    const t = this.state.tracks[trackId];
    if (t) this.set({ tracks: { ...this.state.tracks, [trackId]: [off(t[0]), off(t[1])] } });
  }

  private loop(): void {
    if (this.timer) clearTimeout(this.timer);
    const tick = async () => {
      if (!this.state.playing) return;
      try {
        const s = await this.backend.status();
        if (!this.state.playing) return;
        if (!s.playing) {
          this.halt(this.state.position);
          return;
        }
        this.applyStatus(s);
      } catch (e) {
        this.onError(e);
      }
      if (this.state.playing) this.timer = setTimeout(() => void tick(), this.tickMs);
    };
    this.timer = setTimeout(() => void tick(), this.tickMs);
  }

  /** Folds a status reading into the meters. Public for tests. */
  applyStatus(s: TransportStatus): void {
    const tracks: Record<string, [MeterLevel, MeterLevel]> = {};
    s.trackIds.forEach((id, i) => {
      const m = s.meters[i] ?? [0, 0];
      const prev = this.state.tracks[id] ?? [quiet(), quiet()];
      tracks[id] = [this.meter(`${id}:0`, prev[0], m[0]), this.meter(`${id}:1`, prev[1], m[1])];
    });
    const chains: TransportState["chains"] = {};
    s.trackIds.forEach((id, i) => {
      const m = s.chainMeters?.[i];
      if (!m) return;
      const prev = this.state.chains[id] ?? [quiet(), quiet(), quiet(), quiet()];
      chains[id] = [0, 1, 2, 3].map((k) => this.meter(`${id}:fx${k}`, prev[k]!, m[k]!)) as TransportState["chains"][string];
    });
    this.set({
      position: s.position,
      tracks,
      chains,
      master: [this.meter("m:0", this.state.master[0], s.master[0]), this.meter("m:1", this.state.master[1], s.master[1])],
    });
  }

  private meter(key: string, prev: MeterLevel, reading: number): MeterLevel {
    const r = foldMeter(prev, reading, this.holds.get(key) ?? 0);
    this.holds.set(key, r.hold);
    return r.meter;
  }

  private set(patch: Partial<TransportState>): void {
    this.state = { ...this.state, ...patch };
    for (const fn of this.listeners) fn(this.state);
  }

  /** Runs transport calls one at a time so fast key presses stay in order. */
  private serial(fn: () => Promise<void>): Promise<void> {
    const run = this.busy.then(fn, fn);
    this.busy = run.catch((e: unknown) => this.onError(e));
    return run;
  }
}

/** Meter position for a 0..1 peak on a -60 dB to 0 dB scale. */
export function meterFraction(peak: number, floorDb = -60): number {
  if (peak <= 0) return 0;
  const db = 20 * Math.log10(peak);
  return Math.max(0, Math.min(1, (db - floorDb) / -floorDb));
}
