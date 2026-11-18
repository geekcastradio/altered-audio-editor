// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/** Video reference helpers: time mapping, timecode and the frame bytes from the backend. */

export interface VideoMeta {
  width: number;
  height: number;
  fps: number;
  duration: number;
  has_audio: boolean;
  /** The picture codec by its common name; empty or missing when unknown. */
  codec?: string;
}

/** How pictures have been coming for an open video. */
export interface VideoStats {
  frames: number;
  /** Pictures that took longer than one frame of the video to get. */
  slow: number;
  average_ms: number;
  max_ms: number;
  /** True when a hardware decoder was asked for and the system accepted it. */
  hardware: boolean;
}

/** One line for the Video panel: what the file is and how decoding is going. */
export function describeVideo(meta: VideoMeta, stats: VideoStats | null): string {
  const rate = meta.fps > 0 ? ` at ${Math.round(meta.fps * 100) / 100} fps` : "";
  const parts = [`${meta.codec ? `${meta.codec}, ` : ""}${meta.width}x${meta.height}${rate}`];
  if (stats && stats.frames > 0) {
    parts.push(`${Math.round(stats.average_ms)} ms per picture (slowest ${Math.round(stats.max_ms)})`);
    parts.push(`${stats.slow} of ${stats.frames} slower than one frame`);
  }
  parts.push(stats?.hardware ? "hardware decoder in use" : "software decoding");
  return parts.join(" | ");
}

export interface VideoPicture {
  width: number;
  height: number;
  /** Time of the picture in the video, in seconds. */
  time: number;
  rgba: Uint8ClampedArray<ArrayBuffer>;
}

const HEADER = 16;

/** Reads the bytes `video_frame` returns: width u32, height u32, time f64, then RGBA. */
export function decodePicture(input: ArrayBuffer | ArrayBufferView | readonly number[]): VideoPicture | null {
  const bytes = toBuffer(input);
  if (!bytes || bytes.byteLength < HEADER) return null;
  const head = new DataView(bytes, 0, HEADER);
  const width = head.getUint32(0, true);
  const height = head.getUint32(4, true);
  const time = head.getFloat64(8, true);
  if (width === 0 || height === 0 || bytes.byteLength - HEADER !== width * height * 4) return null;
  return { width, height, time, rgba: new Uint8ClampedArray(bytes.slice(HEADER)) };
}

/**
 * The time in the video that plays at `sessionTime`, or null when the video
 * has not started yet or has ended there (show black, not a stale picture).
 */
export function videoTime(sessionTime: number, offset: number, duration: number): number | null {
  const t = sessionTime - offset;
  if (!Number.isFinite(t) || t < 0) return null;
  if (duration > 0 && t > duration) return null;
  return t;
}

/** Hours:minutes:seconds:frames, the way picture editors read time. */
export function timecode(seconds: number, fps: number): string {
  const rate = fps > 0 ? fps : 25;
  const whole = Math.max(0, Math.round(seconds * rate));
  const base = Math.round(rate);
  const frames = whole % base;
  const total = Math.floor(whole / base);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${pad(Math.floor(total / 3600))}:${pad(Math.floor(total / 60) % 60)}:${pad(total % 60)}:${pad(frames)}`;
}

/** Whether the picture should be fetched again: the playhead moved a frame or more. */
export function needsFrame(last: number | null, next: number | null, fps: number): boolean {
  if (next === null) return last !== null;
  if (last === null) return true;
  const frame = 1 / (fps > 0 ? fps : 25);
  return Math.abs(next - last) >= frame * 0.9;
}

/** Width to ask for: the panel's width in device pixels, held to a sensible range. */
export function askWidth(panelWidth: number, ratio: number, videoWidth: number): number {
  const want = Math.round(Math.max(1, panelWidth) * Math.max(1, ratio));
  return Math.min(Math.max(want, 160), Math.max(160, videoWidth), 1920);
}

/** The bytes of whatever form the bridge handed back (a buffer, a typed array or a plain array), or null. */
function toBuffer(input: unknown): ArrayBuffer | null {
  if (input instanceof ArrayBuffer || Object.prototype.toString.call(input) === "[object ArrayBuffer]") return input as ArrayBuffer;
  if (ArrayBuffer.isView(input)) {
    const v = input as ArrayBufferView;
    return new Uint8Array(v.buffer as ArrayBuffer, v.byteOffset, v.byteLength).slice().buffer;
  }
  if (Array.isArray(input)) return Uint8Array.from(input as number[]).buffer;
  return null;
}

/** A message for an error of any shape; the bridge does not always hand back plain text. */
export function errorText(e: unknown, fallback: string): string {
  if (typeof e === "string" && e) return e;
  if (e instanceof Error && e.message) return e.message;
  if (e && typeof e === "object") {
    const m = (e as { message?: unknown }).message;
    if (typeof m === "string" && m) return m;
    try {
      const j = JSON.stringify(e);
      if (j && j !== "{}") return `${fallback} (${j})`;
    } catch {
      // Not printable; the plain fallback below.
    }
  }
  return fallback;
}
