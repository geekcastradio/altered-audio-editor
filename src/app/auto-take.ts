// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * Recording automation while playing: sits between the controls (Mixer
 * faders and pans, track and bus dials) and the session. A track or bus in
 * Write, Latch or Touch mode takes its moves while the transport plays:
 * they are heard at once (applied without history, and held over the
 * lane in the engine), and written into the lane as one undoable step
 * when playback stops. See `auto-record.ts` for what each mode writes.
 */

import type { BusDoc, EnvelopeDoc, Op, SessionDoc, TrackDoc } from "../stores/session";
import type { EffectInfo } from "./core";
import { autoRange, autoRest, isOrphan, newLane, putLane, targetKey, type AutoOwner, type AutoTarget } from "../panels/editor/automation";
import { Take, toleranceFor, type RecordMode } from "./auto-record";

export interface AutoTakeDeps {
  doc: () => SessionDoc | null;
  /** Playhead, session samples. */
  position: () => number;
  /** Share of a lane's range under which a recorded point is dropped. */
  thin: () => number;
  /** Applies an op at once without an undo step. */
  applyDirect: (op: Op) => Promise<unknown>;
  /** Applies an op as one undoable step. */
  commit: (label: string, op: Op) => Promise<void>;
  /** Holds a setting over its lane in the engine (null gives it back). */
  live: (owner: string, target: AutoTarget, value: number | null) => void;
  /** Gives every held setting back. */
  clearLive: () => void;
  /** The built-in effects with their controls, for the range and rest value of an effect lane. */
  catalog?: () => readonly EffectInfo[];
}

type Owner = TrackDoc | BusDoc;

/** Settings a take can record: the fader, the pan, and a built-in effect's controls. */
const recordable = (t: AutoTarget) => t.type === "volume" || t.type === "pan" || t.type === "effect_param";

interface Armed {
  owner: string;
  kind: "track" | "bus";
  target: AutoTarget;
  /** What the control read when the take began; put back at the end. */
  original: number;
}

export class AutoTake {
  private take: Take | null = null;
  private armed = new Map<string, Armed>();

  constructor(private readonly deps: AutoTakeDeps) {}

  /** True from Play to Stop while a take runs. */
  get active(): boolean {
    return this.take !== null;
  }

  private find(id: string): { owner: Owner; kind: "track" | "bus" } | null {
    const doc = this.deps.doc();
    const t = doc?.tracks.find((x) => x.id === id);
    if (t) return { owner: t, kind: "track" };
    const b = doc?.buses?.find((x) => x.id === id);
    return b ? { owner: b, kind: "bus" } : null;
  }

  /** The mode a track or bus records in, or null when it only plays its lanes. */
  modeOf(id: string): RecordMode | null {
    return this.find(id)?.owner.record_mode ?? null;
  }

  /** Playback started: a take begins. Write-mode lanes that exist start writing at once. */
  start(): void {
    const doc = this.deps.doc();
    if (!doc) return;
    this.take = new Take(doc.sample_rate);
    this.armed.clear();
    const pos = this.deps.position();
    for (const o of [...doc.tracks, ...(doc.buses ?? [])]) {
      if (o.record_mode !== "write") continue;
      for (const lane of (o.envelopes ?? []) as EnvelopeDoc[]) {
        if (!lane.enabled || lane.points.length === 0) continue;
        if (!recordable(lane.target) || isOrphan(o as AutoOwner, this.deps.catalog?.() ?? [], lane.target)) continue;
        const value = this.restOf(o, lane.target);
        this.arm(o.id, lane.target, pos, value);
        // Write holds the control's own value over the old lane from the start.
        this.deps.live(o.id, lane.target, value);
      }
    }
  }

  private restOf(o: Owner, target: AutoTarget): number {
    return autoRest(o as AutoOwner, this.deps.catalog?.() ?? [], target);
  }

  private arm(owner: string, target: AutoTarget, pos: number, value: number): string | null {
    const found = this.find(owner);
    if (!found || !this.take) return null;
    const mode = found.owner.record_mode;
    if (!mode) return null;
    const key = `${owner}:${targetKey(target)}`;
    if (!this.take.has(key)) {
      const lane = ((found.owner.envelopes ?? []) as EnvelopeDoc[]).find((e) => targetKey(e.target) === targetKey(target));
      const range = autoRange(found.owner as AutoOwner, this.deps.catalog?.() ?? [], target);
      this.take.arm(key, mode, lane?.points ?? [], this.restOf(found.owner, target), toleranceFor(range, this.deps.thin()), pos, value);
      this.armed.set(key, { owner, kind: found.kind, target, original: this.restOf(found.owner, target) });
    }
    return key;
  }

  private directOp(a: Armed, value: number): Op {
    if (a.target.type === "effect_param") {
      const { slot_id, param } = a.target;
      const chain = this.find(a.owner)?.owner.effects;
      const next = chain && { ...chain, slots: chain.slots.map((sl) => (sl.id === slot_id ? { ...sl, params: { ...sl.params, [param]: value }, preset: null } : sl)) };
      return { op: "set_effects", target: { kind: a.kind, id: a.owner }, chain: next } as Op;
    }
    const changes = a.target.type === "volume" ? { volume_db: value } : { pan: Math.round(value) };
    return { op: a.kind === "bus" ? "set_bus" : "set_track", id: a.owner, changes } as Op;
  }

  /**
   * A control of `owner` was touched (`phase` "grab"), moved ("move") or
   * let go ("release"). Returns true when a take handled it: the caller
   * then makes no edit of its own.
   */
  control(owner: string, target: AutoTarget, phase: "grab" | "move" | "release", value: number): boolean {
    if (!this.take || !recordable(target)) return false;
    const key = this.arm(owner, target, this.deps.position(), value);
    if (!key) return false;
    const pos = this.deps.position();
    const a = this.armed.get(key)!;
    if (phase === "grab") {
      this.take.grab(key, pos, value);
    } else if (phase === "release") {
      this.take.release(key, pos);
      if (!this.take.writing(key)) this.deps.live(owner, target, null);
      return true;
    }
    // Heard at once, whatever the lane says; no undo step for each move.
    void this.deps.applyDirect(this.directOp(a, value));
    if (this.take.writing(key)) {
      this.take.set(key, pos, value);
      this.deps.live(owner, target, value);
    } else if (phase === "move") {
      this.take.set(key, pos, value);
    }
    return true;
  }

  /** Playback stopped: the controls go back where they were and the lanes take what was written, in one undoable step. */
  async stop(): Promise<void> {
    const take = this.take;
    if (!take) return;
    this.take = null;
    this.deps.clearLive();
    const results = take.finish(this.deps.position());
    const armed = [...this.armed.values()];
    this.armed.clear();
    // Controls first return to where they were, so undo brings back the lanes and nothing else.
    for (const a of armed) await this.deps.applyDirect(this.directOp(a, a.original));
    if (results.size === 0) return;
    const doc = this.deps.doc();
    if (!doc) return;
    const ops: Op[] = [];
    const byOwner = new Map<string, EnvelopeDoc[]>();
    for (const [key, points] of results) {
      const a = this.armed.get(key) ?? armed.find((x) => `${x.owner}:${targetKey(x.target)}` === key);
      if (!a) continue;
      const found = this.find(a.owner);
      if (!found) continue;
      const envs = byOwner.get(a.owner) ?? [];
      const base: AutoOwner = { ...(found.owner as AutoOwner), envelopes: envs.length > 0 ? envs : ((found.owner.envelopes ?? []) as EnvelopeDoc[]) };
      const existing = (base.envelopes ?? []).find((e) => targetKey(e.target) === targetKey(a.target));
      byOwner.set(a.owner, putLane(base, { ...(existing ?? newLane(a.target)), enabled: true, points }));
    }
    for (const [owner, envelopes] of byOwner) ops.push({ op: "set_envelopes", owner, envelopes } as Op);
    if (ops.length > 0) await this.deps.commit("Record Automation", ops.length === 1 ? ops[0]! : ({ op: "batch", ops } as Op));
  }
}
