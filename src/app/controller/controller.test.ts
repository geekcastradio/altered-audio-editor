// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it, vi } from "vitest";
import type { Op, SessionDoc } from "../../stores/session";
import type { AutoTake } from "../auto-take";
import { Controller, IDLE_RELEASE_MS } from "./controller";
import { mackiePreset, type Message } from "./map";

const msg = (kind: Message["kind"], channel: number, number: number, value: number): Message => ({ port: "p", kind, channel, number, value });

function setup(trackCount = 10) {
  const tracks = Array.from({ length: trackCount }, (_, i) => ({ id: `t${i}`, name: `T${i}`, volume_db: 0, pan: 0, muted: false, solo: false, record_armed: false, hidden: false }));
  const doc = { sample_rate: 48_000, tracks, buses: [], master: { volume_db: 0 } } as unknown as SessionDoc;
  const edits: { label: string; op: Op }[] = [];
  const control = vi.fn((..._a: unknown[]) => false);
  const actions: string[] = [];
  const timers: { fn: () => void; ms: number }[] = [];
  const c = new Controller({
    doc: () => doc,
    edit: async (label, op) => void edits.push({ label, op }),
    autoTake: { control } as unknown as AutoTake,
    runAction: (id) => void actions.push(id),
    setTimer: (fn, ms) => {
      timers.push({ fn, ms });
      return timers.length;
    },
    clearTimer: () => {},
  });
  c.setBindings(mackiePreset());
  return { c, doc, edits, control, actions, timers };
}

describe("Controller", () => {
  it("a Mackie fader sets that track's volume", () => {
    const s = setup();
    expect(s.c.handle(msg("bend", 2, 0, 16383))).toBe(true);
    expect(s.edits[0]!.label).toBe("Track Volume");
    expect(s.edits[0]!.op).toEqual({ op: "set_track", id: "t2", changes: { volume_db: 12 } });
  });

  it("the ninth fader is the master", () => {
    const s = setup();
    s.c.handle(msg("bend", 8, 0, 0));
    expect(s.edits[0]!.op).toEqual({ op: "set_master_volume", volume_db: -96 });
  });

  it("a relative knob turns pan by its steps, both ways", () => {
    const s = setup();
    s.c.handle(msg("cc", 0, 16, 3));
    expect(s.edits[0]!.op).toEqual({ op: "set_track", id: "t0", changes: { pan: 6 } });
    s.c.handle(msg("cc", 0, 17, 66));
    expect(s.edits[1]!.op).toEqual({ op: "set_track", id: "t1", changes: { pan: -4 } });
  });

  it("buttons toggle on press and ignore release", () => {
    const s = setup();
    s.c.handle(msg("note", 0, 16, 127)); // mute track 1
    s.c.handle(msg("note", 0, 16, 0));
    s.c.handle(msg("note", 0, 8 + 1, 127)); // solo track 2
    s.c.handle(msg("note", 0, 3, 127)); // arm track 4
    expect(s.edits.map((e) => e.label)).toEqual(["Mute Track", "Solo Track", "Arm Track"]);
    expect(s.edits[0]!.op).toEqual({ op: "set_track", id: "t0", changes: { muted: true } });
  });

  it("transport buttons run commands, once per press", () => {
    const s = setup();
    s.c.handle(msg("note", 0, 94, 127));
    s.c.handle(msg("note", 0, 94, 0));
    s.c.handle(msg("note", 0, 93, 127));
    expect(s.actions).toEqual(["multitrack.play-pause", "multitrack.stop"]);
  });

  it("bank buttons page the surface and stop at the ends", () => {
    const s = setup(10);
    s.c.handle(msg("note", 0, 47, 127)); // next 8
    s.c.handle(msg("bend", 0, 0, 16383)); // fader 1 is now track 9
    expect(s.edits[0]!.op).toMatchObject({ id: "t8" });
    s.c.handle(msg("note", 0, 47, 127));
    s.c.handle(msg("note", 0, 47, 127));
    expect(s.c.bank).toBe(9);
    s.c.handle(msg("note", 0, 46, 127));
    s.c.handle(msg("note", 0, 46, 127));
    expect(s.c.bank).toBe(0);
  });

  it("a recording take gets the fader instead of an edit", () => {
    const s = setup();
    s.control.mockImplementation((_o, _t, phase) => phase === "move");
    s.c.handle(msg("bend", 0, 0, 8000));
    expect(s.edits).toEqual([]);
    expect(s.control.mock.calls.map((c) => c[2])).toEqual(["grab", "move"]);
  });

  it("a fader with no touch sensor is let go after it stops moving", () => {
    const s = setup();
    s.c.handle(msg("bend", 0, 0, 8000));
    s.c.handle(msg("bend", 0, 0, 9000));
    expect(s.control.mock.calls.filter((c) => c[2] === "grab")).toHaveLength(1);
    expect(s.timers.at(-1)!.ms).toBe(IDLE_RELEASE_MS);
    s.timers.at(-1)!.fn();
    expect(s.control.mock.calls.at(-1)![2]).toBe("release");
  });

  it("a Mackie touch sensor grabs and lets go exactly", () => {
    const s = setup();
    s.c.handle(msg("note", 0, 104, 127));
    s.c.handle(msg("bend", 0, 0, 8000));
    expect(s.timers).toHaveLength(0);
    s.c.handle(msg("note", 0, 104, 0));
    expect(s.control.mock.calls.map((c) => c[2])).toEqual(["grab", "move", "release"]);
  });

  it("learning takes the next control and does nothing else", () => {
    const s = setup();
    const got: unknown[] = [];
    s.c.learn((src) => got.push(src));
    expect(s.c.handle(msg("note", 0, 16, 0))).toBe(true);
    expect(got).toEqual([]);
    s.c.handle(msg("note", 0, 16, 127));
    expect(got).toEqual([{ kind: "note", channel: 0, number: 16 }]);
    expect(s.edits).toEqual([]);
    // Learning is over: the same button now works.
    s.c.handle(msg("note", 0, 16, 127));
    expect(s.edits).toHaveLength(1);
  });

  it("messages with nothing bound are left alone", () => {
    const s = setup();
    expect(s.c.handle(msg("cc", 5, 99, 1))).toBe(false);
  });
});

describe("Controller feedback", () => {
  function surface(feedback = true) {
    const tracks = Array.from({ length: 10 }, (_, i) => ({ id: `t${i}`, name: `T${i}`, volume_db: 0, pan: 0, muted: false, solo: false, record_armed: false, hidden: false }));
    const doc = { sample_rate: 48_000, tracks, buses: [], master: { volume_db: 0 } } as unknown as SessionDoc;
    const sent: { port: string; messages: number[][] }[] = [];
    const timers: { fn: () => void; ms: number }[] = [];
    const c = new Controller({
      doc: () => doc,
      edit: async (_label, op) => {
        // The screen follows the edit, as the session does.
        if (op.op === "set_track" && typeof op.changes.volume_db === "number") tracks.find((t) => t.id === op.id)!.volume_db = op.changes.volume_db;
      },
      autoTake: { control: () => false } as unknown as AutoTake,
      runAction: () => {},
      send: (port, messages) => void sent.push({ port, messages }),
      feedback: () => feedback,
      ports: () => ["p"],
      setTimer: (fn, ms) => {
        timers.push({ fn, ms });
        return timers.length;
      },
      clearTimer: () => {},
    });
    c.setBindings(mackiePreset());
    return { c, tracks, sent, timers };
  }
  const fader = (s: ReturnType<typeof surface>, ch: number) => s.sent.flatMap((x) => x.messages).filter((m) => m[0] === (0xe0 | ch));

  it("sends nothing when feedback is off", () => {
    const s = surface(false);
    s.c.sync(true);
    expect(s.sent).toEqual([]);
  });

  it("sends every fader, light and ring once, then only what changed", () => {
    const s = surface();
    s.c.sync(true);
    expect(s.sent.length).toBe(1);
    const first = s.sent[0]!.messages.length;
    expect(first).toBeGreaterThan(8);
    s.c.sync();
    expect(s.sent.length).toBe(1);
    s.tracks[1]!.volume_db = -96;
    s.c.sync();
    expect(fader(s, 1).at(-1)).toEqual([0xe1, 0, 0]);
    expect(s.sent[1]!.messages.length).toBe(1);
  });

  it("leaves a touched fader alone and puts it where the screen has it when it is let go", () => {
    const s = surface();
    s.c.sync(true);
    s.sent.length = 0;
    s.c.handle(msg("note", 0, 104, 127)); // touch fader 1
    s.c.handle(msg("bend", 0, 0, 8000));
    s.tracks[0]!.volume_db = -20;
    s.c.sync();
    expect(fader(s, 0)).toEqual([]);
    s.c.handle(msg("note", 0, 104, 0)); // let go
    expect(fader(s, 0).length).toBe(1);
  });

  it("a fader without a touch sensor is left alone while it moves and sent its position after it stops", () => {
    const s = surface();
    s.c.sync(true);
    s.sent.length = 0;
    s.c.handle(msg("bend", 3, 0, 9000));
    s.c.sync();
    expect(fader(s, 3)).toEqual([]);
    s.timers.at(-1)!.fn(); // the idle timer runs out
    expect(fader(s, 3).length).toBe(1);
  });

  it("paging the tracks shows the new tracks on the surface", () => {
    const s = surface();
    s.tracks[8]!.volume_db = 12;
    s.c.sync(true);
    s.sent.length = 0;
    s.c.handle(msg("note", 0, 47, 127)); // next 8 tracks
    expect(fader(s, 0).at(-1)).toEqual([0xe0, 127, 127]); // track 9, at +12 dB
  });
});
