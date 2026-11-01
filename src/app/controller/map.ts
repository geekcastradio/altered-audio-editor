// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * Hardware controllers: what a message from a fader box or control
 * surface is bound to. A binding pairs a source (a control change, a note
 * or a fader's pitch bend, on one MIDI channel) with a target (a track's
 * fader, pan, mute, solo or arm, the master, a menu command, or paging
 * the tracks the surface shows).
 */

export type SourceKind = "cc" | "note" | "bend";

export interface Source {
  kind: SourceKind;
  /** MIDI channel, 0 to 15. */
  channel: number;
  /** The controller or note; 0 for a fader's pitch bend. */
  number: number;
}

type OnTrack<T extends string> = { type: T; track: number };

export type Target =
  | OnTrack<"volume">
  | OnTrack<"pan">
  | OnTrack<"mute">
  | OnTrack<"solo">
  | OnTrack<"arm">
  | OnTrack<"touch">
  | { type: "master" }
  | { type: "action"; action: string }
  | { type: "bank"; by: number };

export interface Binding {
  source: Source;
  target: Target;
  /** An encoder that sends how far it turned, not where it is. */
  relative?: boolean;
}

/** One message from the core. */
export interface Message {
  port: string;
  kind: "cc" | "note" | "bend";
  channel: number;
  number: number;
  value: number;
}

export const sourceKey = (s: Source) => `${s.kind}:${s.channel}:${s.number}`;

export function matches(b: Binding, m: Message): boolean {
  return b.source.kind === m.kind && b.source.channel === m.channel && b.source.number === m.number;
}

/** A message's position: 0 to 1. */
export function position(m: Message): number {
  return m.kind === "bend" ? m.value / 16383 : m.value / 127;
}

/** How far a relative encoder turned: 1 to 63 is up, 65 to 127 is down (sign and size). */
export function turn(value: number): number {
  if (value === 0 || value === 64) return 0;
  return value < 64 ? value : -(value - 64);
}

/** A pan from a 0 to 1 position, with a small dead zone so center is easy to find. */
export function panAt(pos: number): number {
  const p = Math.round(Math.min(1, Math.max(0, pos)) * 200 - 100);
  return Math.abs(p) <= 3 ? 0 : p;
}

/** The name of a source for the list: "Fader 3 (channel 3)", "Knob 16 (channel 1)". */
export function sourceName(s: Source): string {
  const ch = `channel ${s.channel + 1}`;
  if (s.kind === "bend") return `Fader, ${ch}`;
  if (s.kind === "note") return `Button ${s.number}, ${ch}`;
  return `Knob ${s.number}, ${ch}`;
}

export const ACTIONS: { id: string; label: string }[] = [
  { id: "multitrack.play-pause", label: "Play / Stop" },
  { id: "multitrack.stop", label: "Stop" },
  { id: "multitrack.record", label: "Record" },
  { id: "transport.go-start", label: "Go to Start" },
  { id: "transport.go-end", label: "Go to End" },
  { id: "transport.prev-marker", label: "Previous Marker" },
  { id: "transport.next-marker", label: "Next Marker" },
  { id: "edit.undo", label: "Undo" },
  { id: "edit.redo", label: "Redo" },
  { id: "file.save", label: "Save" },
];

const TRACK_LABEL: Record<string, string> = { volume: "Volume", pan: "Pan", mute: "Mute", solo: "Solo", arm: "Arm", touch: "Fader Touch" };

export function targetName(t: Target): string {
  switch (t.type) {
    case "master":
      return "Master Volume";
    case "bank":
      return t.by === 8 ? "Next 8 Tracks" : t.by === -8 ? "Previous 8 Tracks" : t.by === 1 ? "Next Track" : t.by === -1 ? "Previous Track" : `Move ${t.by} Tracks`;
    case "action":
      return ACTIONS.find((a) => a.id === t.action)?.label ?? t.action;
    default:
      return `Track ${t.track + 1} ${TRACK_LABEL[t.type]}`;
  }
}

/** Every target a person can pick in the list, for tracks 1 to `tracks`. */
export function targetChoices(tracks: number): Target[] {
  const out: Target[] = [{ type: "master" }];
  for (let i = 0; i < tracks; i++) for (const type of ["volume", "pan", "mute", "solo", "arm"] as const) out.push({ type, track: i });
  for (const a of ACTIONS) out.push({ type: "action", action: a.id });
  out.push({ type: "bank", by: -8 }, { type: "bank", by: 8 }, { type: "bank", by: -1 }, { type: "bank", by: 1 });
  return out;
}

/** The standard Mackie Control layout for eight channel strips: what a BCF2000 in its Mackie mode sends. */
export function mackiePreset(): Binding[] {
  const out: Binding[] = [];
  for (let i = 0; i < 8; i++) {
    out.push({ source: { kind: "bend", channel: i, number: 0 }, target: { type: "volume", track: i } });
    out.push({ source: { kind: "note", channel: 0, number: 104 + i }, target: { type: "touch", track: i } });
    out.push({ source: { kind: "cc", channel: 0, number: 16 + i }, target: { type: "pan", track: i }, relative: true });
    out.push({ source: { kind: "note", channel: 0, number: i }, target: { type: "arm", track: i } });
    out.push({ source: { kind: "note", channel: 0, number: 8 + i }, target: { type: "solo", track: i } });
    out.push({ source: { kind: "note", channel: 0, number: 16 + i }, target: { type: "mute", track: i } });
  }
  out.push({ source: { kind: "bend", channel: 8, number: 0 }, target: { type: "master" } });
  const b = (number: number, target: Target): Binding => ({ source: { kind: "note", channel: 0, number }, target });
  out.push(
    b(46, { type: "bank", by: -8 }),
    b(47, { type: "bank", by: 8 }),
    b(48, { type: "bank", by: -1 }),
    b(49, { type: "bank", by: 1 }),
    b(91, { type: "action", action: "transport.go-start" }),
    b(92, { type: "action", action: "transport.go-end" }),
    b(93, { type: "action", action: "multitrack.stop" }),
    b(94, { type: "action", action: "multitrack.play-pause" }),
    b(95, { type: "action", action: "multitrack.record" }),
  );
  return out;
}

const int = (v: unknown, lo: number, hi: number): number | null => (typeof v === "number" && Number.isInteger(v) && v >= lo && v <= hi ? v : null);

function cleanTarget(t: unknown): Target | null {
  if (!t || typeof t !== "object") return null;
  const o = t as Record<string, unknown>;
  if (o.type === "master") return { type: "master" };
  if (o.type === "bank") {
    const by = int(o.by, -64, 64);
    return by === null || by === 0 ? null : { type: "bank", by };
  }
  if (o.type === "action") return typeof o.action === "string" && o.action.length > 0 && o.action.length < 80 ? { type: "action", action: o.action } : null;
  if (o.type === "volume" || o.type === "pan" || o.type === "mute" || o.type === "solo" || o.type === "arm" || o.type === "touch") {
    const track = int(o.track, 0, 255);
    return track === null ? null : ({ type: o.type, track } as Target);
  }
  return null;
}

/** Bindings read from saved settings: anything damaged is dropped, a source bound twice keeps the last. */
export function cleanBindings(raw: unknown): Binding[] {
  if (!Array.isArray(raw)) return [];
  const seen = new Map<string, Binding>();
  for (const r of raw.slice(0, 1024)) {
    if (!r || typeof r !== "object") continue;
    const o = r as Record<string, unknown>;
    const s = o.source as Record<string, unknown> | undefined;
    const kind = s?.kind === "cc" || s?.kind === "note" || s?.kind === "bend" ? s.kind : null;
    const channel = int(s?.channel, 0, 15);
    const number = int(s?.number, 0, 127);
    const target = cleanTarget(o.target);
    if (!kind || channel === null || number === null || !target) continue;
    const b: Binding = { source: { kind, channel, number }, target };
    if (o.relative === true) b.relative = true;
    seen.set(sourceKey(b.source), b);
  }
  return [...seen.values()];
}
