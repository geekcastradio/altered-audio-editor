// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import type { ClipDoc, SessionDoc, TrackDoc } from "../stores/session";
import { tracksWithRoom } from "./place-track";

const fade = () => ({ length: 0, shape: "linear" as const, curve: 0 });
// One second at 48 kHz.
const clip = (id: string, start: number): ClipDoc => ({ id, name: id, media_id: "m", start, source_in: 0, source_out: 48_000, gain_db: 0, muted: false, locked: false, color: null, fade_in: fade(), fade_out: fade() });
const track = (id: string, clips: ClipDoc[], kind = "audio"): TrackDoc => ({ id, name: id, kind, clips }) as unknown as TrackDoc;
const session = (tracks: TrackDoc[]) =>
  ({ sample_rate: 48_000, media: [{ id: "m", path: "a.wav", relative_path: null, sample_rate: 48_000, channels: 1, length: 480_000 }], tracks, markers: [] }) as unknown as SessionDoc;

describe("tracksWithRoom", () => {
  it("lists every empty audio track in order, so a new session uses Track 1", () => {
    const doc = session(["1", "2", "3", "4", "5", "6"].map((n) => track(n, [])));
    expect(tracksWithRoom(doc, 0, 480_000)).toEqual(["1", "2", "3", "4", "5", "6"]);
  });

  it("skips a track that is busy where the audio would land", () => {
    const doc = session([track("1", [clip("a", 0)]), track("2", [])]);
    expect(tracksWithRoom(doc, 24_000, 96_000)).toEqual(["2"]);
  });

  it("uses a track whose clips end before the new audio starts", () => {
    const doc = session([track("1", [clip("a", 0)])]);
    expect(tracksWithRoom(doc, 48_000, 96_000)).toEqual(["1"]);
    expect(tracksWithRoom(doc, 47_999, 96_000)).toEqual([]);
  });

  it("never offers a MIDI track, and offers nothing when every track is busy", () => {
    const doc = session([track("m", [], "midi"), track("1", [clip("a", 0)])]);
    expect(tracksWithRoom(doc, 0, 10_000)).toEqual([]);
  });
});
