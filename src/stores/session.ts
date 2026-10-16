// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * The open session as the interface sees it. The core owns the session;
 * edits are sent as ops (see src-tauri/src/session/ops.rs) and the core
 * returns the new session plus the op that undoes the edit.
 *
 * Field names match the `.altaudio` file format (snake_case).
 */

import type { Command } from "../commands";
import type { CommandRunner } from "./media-pool";

export interface MediaRefDoc {
  id: string;
  path: string;
  relative_path: string | null;
  sample_rate: number;
  channels: number;
  length: number;
}

export interface FadeDoc {
  length: number;
  shape: string;
  curve: number;
}

export interface ClipDoc {
  id: string;
  name: string;
  media_id: string;
  start: number;
  source_in: number;
  source_out: number;
  gain_db: number;
  muted: boolean;
  locked: boolean;
  color: string | null;
  fade_in: FadeDoc;
  fade_out: FadeDoc;
  /** Clips sharing a group id are selected and moved together. */
  group?: string | null;
  envelopes?: EnvelopeDoc[];
  /** Loop Clip: the source region repeats to fill `length` session samples, starting `phase` source samples into it. */
  looped?: LoopDoc | null;
  [key: string]: unknown;
}

export interface LoopDoc {
  length: number;
  phase: number;
}

export interface EnvelopePointDoc {
  /** Session samples from the clip start. */
  position: number;
  /** Volume: dB from -60 (silent) to +12. */
  value: number;
  curve: "linear" | "hold" | "spline";
}

export interface EnvelopeDoc {
  id: string;
  target: { type: "volume" } | { type: "pan" } | { type: "effect_param"; slot_id: string; param: string };
  enabled: boolean;
  points: EnvelopePointDoc[];
}

export interface TrackDoc {
  id: string;
  name: string;
  role: "standard" | "guest";
  channels: "mono" | "stereo";
  color: string | null;
  height: number;
  volume_db: number;
  pan: number;
  muted: boolean;
  solo: boolean;
  record_armed: boolean;
  /** Missing in files where it is off. */
  polarity_inverted?: boolean;
  /** Folded to its name row. Missing in files where it is off. */
  collapsed?: boolean;
  monitor_input?: boolean;
  input?: string | null;
  clips: ClipDoc[];
  [key: string]: unknown;
}

export interface ViewDoc {
  samples_per_pixel: number;
  scroll: number;
  scroll_y: number;
  playhead: number;
  selection: { start: number; end: number } | null;
}

export interface SessionDoc {
  format: "altaudio.session";
  schema_version: number;
  id: string;
  name: string;
  sample_rate: number;
  bit_depth: number;
  media: MediaRefDoc[];
  tracks: TrackDoc[];
  markers: MarkerDoc[];
  view: ViewDoc;
  libraries: string[];
  master?: { volume_db: number; channels: string; height: number; [key: string]: unknown };
  [key: string]: unknown;
}

export type MarkerKind = "cue" | "subclip" | "chapter" | "ad_slot" | "social_clip";

export interface MarkerDoc {
  id: string;
  name: string;
  kind: MarkerKind;
  /** Session samples. */
  position: number;
  /** Session samples. 0 is a point marker. */
  duration: number;
  description: string;
  chapter: unknown;
  contracted_duration: number | null;
}

export interface MarkerChange {
  id: string;
  name?: string;
  kind?: MarkerKind;
  position?: number;
  duration?: number;
  description?: string;
}

/** Changes to one clip. Fields left out are unchanged; `track_id` moves it. */
export interface ClipChange {
  id: string;
  track_id?: string;
  name?: string;
  start?: number;
  source_in?: number;
  source_out?: number;
  gain_db?: number;
  muted?: boolean;
  locked?: boolean;
  fade_in?: FadeDoc;
  fade_out?: FadeDoc;
  /** `#rrggbb`, or "" for the track color. */
  color?: string;
  group?: string;
  ungroup?: boolean;
  /** Replaces the volume envelope points. Empty removes the envelope. */
  volume?: EnvelopePointDoc[];
  /** Replaces the pan envelope points (-100 left to 100 right). Empty removes the envelope. */
  pan?: EnvelopePointDoc[];
  /** Loops the clip, or changes its loop. */
  looped?: LoopDoc;
  /** Stops looping. */
  unloop?: boolean;
}

export interface ClipSpec {
  id: string;
  name: string;
  media_id: string;
  start: number;
  source_in: number;
  source_out: number;
}

export interface TrackChanges {
  name?: string;
  muted?: boolean;
  solo?: boolean;
  record_armed?: boolean;
  height?: number;
  color?: string | null;
  volume_db?: number;
  pan?: number;
  polarity_inverted?: boolean;
  collapsed?: boolean;
  monitor_input?: boolean;
  input?: string | null;
}

/** An edit the core understands. */
export type Op =
  | { op: "add_track"; id: string; name: string; stereo: boolean; index: number }
  | { op: "restore_track"; track: TrackDoc; index: number }
  | { op: "remove_track"; id: string }
  | { op: "move_track"; id: string; to: number }
  | { op: "set_track"; id: string; changes: TrackChanges }
  | { op: "add_media"; media: MediaRefDoc }
  | { op: "remove_media"; id: string }
  | { op: "add_clips"; track_id: string; clips: ClipSpec[] }
  | { op: "restore_clips"; clips: [string, ClipDoc][] }
  | { op: "remove_clips"; ids: string[] }
  | { op: "set_sample_rate"; sample_rate: number; exact?: number[] }
  | { op: "set_session_info"; name?: string; bit_depth?: number }
  | { op: "set_clips"; changes: ClipChange[] }
  | { op: "add_markers"; markers: MarkerDoc[] }
  | { op: "remove_markers"; ids: string[] }
  | { op: "set_markers"; changes: MarkerChange[] }
  | { op: "set_master_volume"; volume_db: number }
  | { op: "batch"; ops: Op[] };

export interface SessionBackend {
  get(): Promise<{ session: SessionDoc; path: string | null }>;
  apply(op: Op): Promise<{ session: SessionDoc; inverse: Op }>;
  setView(view: ViewDoc): Promise<void>;
}

type Listener = (doc: SessionDoc | null) => void;

/** Timeline length of a clip, in session samples. */
export function clipLength(clip: ClipDoc, media: MediaRefDoc | undefined, sessionRate: number): number {
  if (clip.looped) return clip.looped.length;
  const srcRate = media?.sample_rate || sessionRate;
  return Math.round(((clip.source_out - clip.source_in) * sessionRate) / srcRate);
}

/** End of the last clip, in session samples. */
export function sessionEnd(doc: SessionDoc): number {
  const media = new Map(doc.media.map((m) => [m.id, m]));
  let end = 0;
  for (const t of doc.tracks) {
    for (const c of t.clips) end = Math.max(end, c.start + clipLength(c, media.get(c.media_id), doc.sample_rate));
  }
  return end;
}

export class SessionStore {
  private doc: SessionDoc | null = null;
  private filePath: string | null = null;
  private readonly listeners = new Set<Listener>();
  private viewTimer: ReturnType<typeof setTimeout> | undefined;

  constructor(private readonly backend: SessionBackend) {}

  subscribe(fn: Listener): () => void {
    this.listeners.add(fn);
    fn(this.doc);
    return () => {
      this.listeners.delete(fn);
    };
  }

  get current(): SessionDoc | null {
    return this.doc;
  }

  /** The session's file, or null before the first save. */
  get path(): string | null {
    return this.filePath;
  }

  async load(): Promise<SessionDoc> {
    const r = await this.backend.get();
    this.set(r.session, r.path);
    return this.doc!;
  }

  /** Replaces the whole document, for example after opening a file. */
  set(doc: SessionDoc, path?: string | null): void {
    this.doc = doc;
    if (path !== undefined) this.filePath = path;
    for (const fn of this.listeners) fn(doc);
  }

  /** Runs an op as one undoable step. */
  edit(label: string, op: Op, runner: CommandRunner): Promise<void> {
    return runner.execute(new OpCommand(this, op, label));
  }

  /** Applies an op directly. For commands only. */
  async apply(op: Op): Promise<Op> {
    const r = await this.backend.apply(op);
    this.set(r.session);
    return r.inverse;
  }

  /** View changes (zoom, scroll, playhead) are not edits; sent to the core after a pause. */
  setView(view: ViewDoc): void {
    if (!this.doc) return;
    this.doc = { ...this.doc, view };
    for (const fn of this.listeners) fn(this.doc);
    clearTimeout(this.viewTimer);
    this.viewTimer = setTimeout(() => void this.backend.setView(view).catch(console.error), 300);
  }
}

/** Track settings a fader-style drag changes many times in a row. */
function mergeKey(op: Op): string | null {
  if (op.op === "set_master_volume") return "master";
  if (op.op !== "set_track") return null;
  const keys = Object.keys(op.changes);
  if (keys.length !== 1 || !["volume_db", "pan", "height"].includes(keys[0]!)) return null;
  return `${op.id}:${keys[0]}`;
}

class OpCommand implements Command {
  private inverse: Op | null = null;

  constructor(
    private readonly store: SessionStore,
    private op: Op,
    readonly label: string,
  ) {}

  /** A run of drags on one fader is one undo step back to where it started. */
  mergeWith(next: Command): boolean {
    if (!(next instanceof OpCommand)) return false;
    const key = mergeKey(this.op);
    if (key === null || key !== mergeKey(next.op)) return false;
    this.op = next.op;
    return true;
  }

  async do(): Promise<void> {
    this.inverse = await this.store.apply(this.op);
  }

  async undo(): Promise<void> {
    if (this.inverse) await this.store.apply(this.inverse);
  }
}
