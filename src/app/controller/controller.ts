// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * Hardware controllers at work: takes messages from fader boxes and
 * control surfaces and turns them into the same edits the mouse makes.
 * A fader moved while a take records goes through the take (so Write,
 * Latch and Touch work with a real fader); otherwise it is an ordinary
 * undoable edit, merged with the moves before it.
 *
 * A surface shows eight tracks at a time. `bank` is how far along the
 * tracks it is; Mackie surfaces page with their bank buttons.
 */

import type { BusDoc, Op, SessionDoc, TrackDoc } from "../../stores/session";
import { faderToDb } from "../../panels/mixer/fader";
import { busOp } from "../../panels/mixer/routing";
import { trackOp } from "../../panels/mixer/solo";
import type { AutoTake } from "../auto-take";
import { frames } from "./feedback";
import { matches, panAt, position, turn, type Binding, type Message, type Source, type Target } from "./map";

export interface ControllerDeps {
  doc: () => SessionDoc | null;
  /** Applies an op as an undoable edit. */
  edit: (label: string, op: Op) => Promise<void>;
  autoTake: AutoTake;
  /** Runs a menu command by its id. */
  runAction: (id: string) => void;
  /** Called when the bank changes, so the screen can say where the surface is. */
  onBank?: (bank: number) => void;
  /** Sends MIDI messages back to a device (by its input name). Missing: nothing is sent. */
  send?: (port: string, messages: number[][]) => void;
  /** Whether the person switched feedback on. */
  feedback?: () => boolean;
  /** The devices listened to, which feedback is sent to. */
  ports?: () => string[];
  /** For tests: the clock and timers. */
  setTimer?: (fn: () => void, ms: number) => unknown;
  clearTimer?: (t: unknown) => void;
}

/** A moved fader counts as let go this long after its last move, when the surface sends no touch message. */
export const IDLE_RELEASE_MS = 400;

type Strip = { kind: "track"; doc: TrackDoc } | { kind: "bus"; doc: BusDoc };

export class Controller {
  private bindings: Binding[] = [];
  bank = 0;
  /** The latest message, for the Controllers dialog to show so a person can see their device is heard. */
  lastMessage: Message | null = null;
  private learning: ((s: Source) => void) | null = null;
  private idle = new Map<string, unknown>();
  private held = new Set<string>();
  /** What was last sent for each control, so a repeat is skipped. */
  private sent = new Map<string, string>();
  /** Faders let go since the last sync: send their position even if it matches what was sent. */
  private flush = new Set<string>();

  constructor(private readonly deps: ControllerDeps) {}

  setBindings(b: Binding[]): void {
    this.bindings = b;
    this.sent.clear();
  }

  /**
   * Tells the surface where things are now: motorized fader positions and
   * button and ring lights. Only what changed is sent (`force` sends it all).
   * A fader being touched or moved is left alone, so it is never fought; it
   * gets its position once it is let go.
   */
  sync(force = false): void {
    const send = this.deps.send;
    const ports = this.deps.ports?.() ?? [];
    if (!send || !this.deps.feedback?.() || ports.length === 0) {
      this.flush.clear();
      return;
    }
    const doc = this.deps.doc();
    const strips = this.strips().map((s) => s.doc);
    const out: number[][] = [];
    for (const f of frames(this.bindings, strips.slice(this.bank, this.bank + 64), doc?.master?.volume_db ?? 0)) {
      if (f.hold !== undefined && (this.held.has(f.hold) || this.idle.has(f.hold))) continue;
      const sig = f.bytes.join(",");
      const released = f.hold !== undefined && this.flush.has(f.hold);
      if (!force && !released && this.sent.get(f.key) === sig) continue;
      this.sent.set(f.key, sig);
      out.push(f.bytes);
    }
    this.flush.clear();
    if (out.length > 0) for (const p of ports) send(p, out);
  }

  /** A fader was let go: put the surface's fader where the screen has it. */
  private letGo(key: string): void {
    this.flush.add(key);
    this.sync();
  }

  /** The next control touched is reported to `fn` and does nothing else. Returns a way to stop waiting. */
  learn(fn: (s: Source) => void): () => void {
    this.learning = fn;
    return () => {
      if (this.learning === fn) this.learning = null;
    };
  }

  /** The strips a surface shows: tracks not hidden, then buses (the Mixer's order). */
  private strips(): Strip[] {
    const doc = this.deps.doc();
    if (!doc) return [];
    return [...doc.tracks.filter((t) => !t.hidden).map((t): Strip => ({ kind: "track", doc: t })), ...(doc.buses ?? []).map((b): Strip => ({ kind: "bus", doc: b }))];
  }

  private strip(track: number): Strip | null {
    return this.strips()[track + this.bank] ?? null;
  }

  /** Handles one message. Returns true when something was bound to it (or it was taken for learning). */
  handle(m: Message): boolean {
    this.lastMessage = m;
    if (this.learning) {
      // A fader or knob sends a stream of messages and a button a press and a release: take the first press or move.
      if (m.kind === "note" && m.value === 0) return true;
      const fn = this.learning;
      this.learning = null;
      fn({ kind: m.kind, channel: m.channel, number: m.number });
      return true;
    }
    let used = false;
    for (const b of this.bindings) {
      if (!matches(b, m)) continue;
      used = true;
      this.apply(b, m);
    }
    return used;
  }

  private apply(b: Binding, m: Message): void {
    const t = b.target;
    switch (t.type) {
      case "volume":
      case "pan":
        return this.level(t, b, m);
      case "touch":
        return this.touch(t.track, m.value > 0);
      case "master":
        return this.master(b, m);
      case "mute":
      case "solo":
      case "arm":
        if (m.value > 0) this.toggle(t);
        return;
      case "action":
        if (m.value > 0) this.deps.runAction(t.action);
        return;
      case "bank":
        if (m.value > 0) this.page(t.by);
    }
  }

  private page(by: number): void {
    const total = this.strips().length;
    this.bank = Math.max(0, Math.min(Math.max(0, total - 1), this.bank + by));
    this.deps.onBank?.(this.bank);
    this.sync(true);
  }

  private level(t: Extract<Target, { type: "volume" | "pan" }>, b: Binding, m: Message): void {
    const s = this.strip(t.track);
    if (!s) return;
    const cur = t.type === "volume" ? s.doc.volume_db : s.doc.pan;
    let value: number;
    if (b.relative) {
      const d = turn(m.value);
      if (d === 0) return;
      value = t.type === "volume" ? Math.max(-96, Math.min(12, Math.round((cur + d * 0.5) * 10) / 10)) : Math.max(-100, Math.min(100, cur + d * 2));
    } else {
      const pos = position(m);
      value = t.type === "volume" ? faderToDb(pos) : panAt(pos);
    }
    this.move(s, t.type, value);
  }

  /** Moves a strip's fader or pan: through a take when one is recording, else as an edit. */
  private move(s: Strip, type: "volume" | "pan", value: number): void {
    const id = s.doc.id;
    const key = `${id}:${type}`;
    const cur = type === "volume" ? s.doc.volume_db : s.doc.pan;
    if (!this.held.has(key) && !this.idle.has(key)) this.deps.autoTake.control(id, { type }, "grab", cur);
    if (!this.held.has(key)) this.rearm(key, id, type, value);
    if (this.deps.autoTake.control(id, { type }, "move", value)) return;
    const changes = type === "volume" ? { volume_db: value } : { pan: value };
    const op = s.kind === "bus" ? busOp(this.deps.doc(), id, changes) : trackOp(this.deps.doc(), id, changes);
    void this.deps.edit(type === "volume" ? (s.kind === "bus" ? "Bus Volume" : "Track Volume") : s.kind === "bus" ? "Bus Pan" : "Track Pan", op);
  }

  private rearm(key: string, id: string, type: "volume" | "pan", value: number): void {
    const clear = this.deps.clearTimer ?? ((h) => clearTimeout(h as ReturnType<typeof setTimeout>));
    const set = this.deps.setTimer ?? ((fn, ms) => setTimeout(fn, ms));
    const old = this.idle.get(key);
    if (old !== undefined) clear(old);
    this.idle.set(
      key,
      set(() => {
        this.idle.delete(key);
        this.deps.autoTake.control(id, { type }, "release", value);
        if (type === "volume") this.letGo(key);
      }, IDLE_RELEASE_MS),
    );
  }

  /** A Mackie fader's touch sensor: exact grab and release, so no idle timer. */
  private touch(track: number, down: boolean): void {
    const s = this.strip(track);
    if (!s) return;
    const key = `${s.doc.id}:volume`;
    if (down) {
      this.held.add(key);
      this.deps.autoTake.control(s.doc.id, { type: "volume" }, "grab", s.doc.volume_db);
    } else if (this.held.delete(key)) {
      this.deps.autoTake.control(s.doc.id, { type: "volume" }, "release", s.doc.volume_db);
      this.letGo(key);
    }
  }

  private master(b: Binding, m: Message): void {
    const doc = this.deps.doc();
    if (!doc) return;
    const cur = doc.master?.volume_db ?? 0;
    let db: number;
    if (b.relative) {
      const d = turn(m.value);
      if (d === 0) return;
      db = Math.max(-60, Math.min(12, Math.round((cur + d * 0.5) * 10) / 10));
    } else db = faderToDb(position(m));
    void this.deps.edit("Master Volume", { op: "set_master_volume", volume_db: db });
  }

  private toggle(t: Extract<Target, { type: "mute" | "solo" | "arm" }>): void {
    const s = this.strip(t.track);
    if (!s) return;
    const doc = this.deps.doc();
    if (t.type === "arm") {
      if (s.kind === "track") void this.deps.edit("Arm Track", trackOp(doc, s.doc.id, { record_armed: !s.doc.record_armed }));
      return;
    }
    const key = t.type === "mute" ? "muted" : "solo";
    const next = !(s.doc as unknown as Record<string, boolean>)[key];
    const label = `${next ? "" : "Un"}${t.type === "mute" ? "Mute" : "Solo"} ${s.kind === "bus" ? "Bus" : "Track"}`;
    void this.deps.edit(label, s.kind === "bus" ? busOp(doc, s.doc.id, { [key]: next }) : trackOp(doc, s.doc.id, { [key]: next }));
  }
}
