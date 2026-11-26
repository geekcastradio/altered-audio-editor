// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { SessionStore, type SessionDoc } from "./session";
import { meterFraction, TransportStore } from "./transport";

function doc(playhead: number): SessionDoc {
  return {
    format: "altaudio.session",
    schema_version: 1,
    id: "s",
    name: "s",
    sample_rate: 48000,
    bit_depth: 32,
    media: [],
    tracks: [],
    markers: [],
    view: { samples_per_pixel: 1024, scroll: 0, scroll_y: 0, playhead, selection: null },
    libraries: [],
  };
}

function setup() {
  const calls: string[] = [];
  let position = 0;
  const session = new SessionStore({
    get: async () => ({ session: doc(0), path: null }),
    apply: async () => {
      throw new Error("unused");
    },
    setView: async () => {},
  });
  session.set(doc(48_000));
  const t = new TransportStore(
    {
      play: async (from) => {
        calls.push(`play:${from}`);
        position = from;
      },
      stop: async () => {
        calls.push("stop");
        return position + 96_000;
      },
      status: async () => ({ playing: true, position, trackIds: [], meters: [], master: [0, 0], underruns: 0 }),
    },
    session,
    () => {},
    10_000,
  );
  return { t, calls, session };
}

describe("TransportStore", () => {
  it("plays from the playhead and pause leaves the playhead where it reached", async () => {
    const { t, calls, session } = setup();
    await t.toggle();
    expect(t.current.playing).toBe(true);
    expect(calls).toEqual(["play:48000"]);
    await t.toggle();
    expect(t.current.playing).toBe(false);
    expect(session.current?.view.playhead).toBe(144_000);
  });

  it("stop returns the playhead to where playback started", async () => {
    const { t, session } = setup();
    await t.play();
    await t.stop();
    expect(session.current?.view.playhead).toBe(48_000);
  });

  it("seeking while playing restarts there; while stopped it moves the playhead", async () => {
    const { t, calls, session } = setup();
    await t.seek(10);
    expect(session.current?.view.playhead).toBe(10);
    await t.play();
    await t.seek(500);
    expect(calls).toEqual(["play:10", "play:500"]);
    await t.stop();
    expect(session.current?.view.playhead).toBe(10);
  });

  it("meters rise at once, fall slowly, hold peaks and remember overs", () => {
    const { t } = setup();
    t.applyStatus({ playing: true, position: 0, trackIds: ["a"], meters: [[0.5, 1.2]], master: [0.5, 0.5], underruns: 0 });
    const a = t.current.tracks["a"]!;
    expect(a[0].level).toBe(0.5);
    expect(a[1].over).toBe(true);
    t.applyStatus({ playing: true, position: 0, trackIds: ["a"], meters: [[0, 0]], master: [0, 0], underruns: 0 });
    const b = t.current.tracks["a"]!;
    expect(b[0].level).toBeLessThan(0.5);
    expect(b[0].level).toBeGreaterThan(0.3);
    expect(b[0].peak).toBe(0.5);
    expect(b[1].over).toBe(true);
  });

  it("keeps over lights after stopping until cleared", async () => {
    const { t } = setup();
    await t.play();
    t.applyStatus({ playing: true, position: 0, trackIds: ["a", "b"], meters: [[1.5, 0], [0.4, 0.4]], master: [1.1, 0.2], underruns: 0 });
    await t.stop();
    expect(t.current.tracks["a"]![0]).toEqual({ level: 0, peak: 0, over: true });
    expect(t.current.tracks["b"]).toBeUndefined();
    expect(t.current.master[0].over).toBe(true);
    t.clearOver("a");
    t.clearOver();
    expect(t.current.tracks["a"]![0].over).toBe(false);
    expect(t.current.master[0].over).toBe(false);
  });

  it("maps peaks to a dB scale", () => {
    expect(meterFraction(1)).toBe(1);
    expect(meterFraction(0)).toBe(0);
    expect(meterFraction(0.001)).toBe(0);
    expect(meterFraction(10 ** (-30 / 20))).toBeCloseTo(0.5);
  });
  it("folds Voice Chain meters for tracks whose chain plays", () => {
    const t = new TransportStore({ play: async () => {}, stop: async () => null, status: async () => ({ playing: false, position: 0, trackIds: [], meters: [], master: [0, 0], underruns: 0 }) }, { current: null } as never);
    t.applyStatus({ playing: true, position: 0, trackIds: ["a", "b"], meters: [[0, 0], [0, 0]], master: [0, 0], underruns: 0, chainMeters: [[0.5, 0.4, 0.25, 0.2], null] });
    expect(t.current.chains.a!.map((m) => m.level)).toEqual([0.5, 0.4, 0.25, 0.2]);
    expect(t.current.chains.b).toBeUndefined();
  });
});

describe("where Play starts", () => {
  it("starts at the selection when the playhead is outside it", async () => {
    const { playStart } = await import("./transport");
    const sel = { start: 100, end: 200 };
    expect(playStart(500, sel, false)).toBe(100);
    expect(playStart(50, sel, false)).toBe(100);
    expect(playStart(150, sel, false)).toBe(150);
    expect(playStart(500, sel, true)).toBe(500);
    expect(playStart(500, null, false)).toBe(500);
  });
});

describe("TransportStore stop", () => {
  it("puts the playhead back where playback started", async () => {
    const { t, session } = setup();
    await t.play();
    await t.stop();
    expect(session.current?.view.playhead).toBe(48_000);
  });

  it("leaves the playhead where playback stopped when returnOnStop is off", async () => {
    const { t, session } = setup();
    t.returnOnStop = false;
    await t.play();
    await t.stop();
    expect(t.current.playing).toBe(false);
    expect(session.current?.view.playhead).toBe(144_000);
  });
});

describe("TransportStore stops at the end of the selection", () => {
  function selSetup(selection: { start: number; end: number }, stopAtSelection = true) {
    let position = 0;
    const calls: string[] = [];
    const d = doc(0);
    d.view = { ...d.view, selection };
    d.media = [];
    const session = new SessionStore({
      get: async () => ({ session: d, path: null }),
      apply: async () => {
        throw new Error("unused");
      },
      setView: async () => {},
    });
    session.set(d);
    const t = new TransportStore(
      {
        play: async (from) => {
          position = from;
          calls.push(`play:${from}`);
        },
        stop: async () => {
          calls.push("stop");
          return position;
        },
        status: async () => {
          position += 5_000;
          return { playing: true, position, trackIds: [], meters: [], master: [0, 0], underruns: 0 };
        },
        stopAtEnd: () => true,
      },
      session,
      () => {},
      1,
    );
    t.stopAtSelection = stopAtSelection;
    return { t, calls };
  }
  const settle = () => new Promise((r) => setTimeout(r, 60));

  it("stops when playback started inside the selection and reached its end", async () => {
    const { t, calls } = selSetup({ start: 1_000, end: 12_000 });
    await t.play();
    await settle();
    expect(t.current.playing).toBe(false);
    expect(calls).toContain("stop");
  });

  it("plays on when the setting is off", async () => {
    const { t, calls } = selSetup({ start: 1_000, end: 12_000 }, false);
    await t.play();
    await settle();
    expect(t.current.playing).toBe(true);
    expect(calls).not.toContain("stop");
    await t.pause();
  });
});

describe("TransportStore starts again from the top when the playhead is at the end", () => {
  function endSetup(playhead: number) {
    let position = 0;
    const calls: string[] = [];
    const d = doc(playhead);
    d.media = [{ id: "m", path: "a.wav", relative_path: null, sample_rate: 48000, channels: 2, length: 100_000 }];
    d.tracks = [
      { id: "t", clips: [{ id: "c", media_id: "m", start: 0, source_in: 0, source_out: 100_000 }] },
    ] as unknown as SessionDoc["tracks"];
    const session = new SessionStore({
      get: async () => ({ session: d, path: null }),
      apply: async () => {
        throw new Error("unused");
      },
      setView: async () => {},
    });
    session.set(d);
    const t = new TransportStore(
      {
        play: async (from) => {
          position = from;
          calls.push(`play:${from}`);
        },
        stop: async () => position,
        status: async () => ({ playing: true, position, trackIds: [], meters: [], master: [0, 0], underruns: 0 }),
        stopAtEnd: () => true,
      },
      session,
      () => {},
      10_000,
    );
    return { t, calls };
  }

  it("plays from 0 when the playhead sits at the end", async () => {
    const { t, calls } = endSetup(100_000);
    await t.play();
    expect(calls).toEqual(["play:0"]);
  });

  it("plays from the playhead when it is before the end", async () => {
    const { t, calls } = endSetup(40_000);
    await t.play();
    expect(calls).toEqual(["play:40000"]);
  });

  it("keeps an explicit start position", async () => {
    const { t, calls } = endSetup(100_000);
    await t.play(100_000);
    expect(calls).toEqual(["play:100000"]);
  });
});
