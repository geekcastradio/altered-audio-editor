// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { fader14, frames, ringByte, type StripState } from "./feedback";
import { mackiePreset } from "./map";

const strip = (id: string, over: Partial<StripState> = {}): StripState => ({ id, volume_db: 0, pan: 0, muted: false, solo: false, record_armed: false, ...over });

describe("feedback", () => {
  it("fader positions use the full 14-bit range", () => {
    expect(fader14(0)).toBe(0);
    expect(fader14(1)).toBe(16383);
    expect(fader14(2)).toBe(16383);
    expect(fader14(-1)).toBe(0);
  });

  it("the pan ring runs from the first lit point on the left to the eleventh on the right", () => {
    expect(ringByte(-100)).toBe(0x11);
    expect(ringByte(0)).toBe(0x16);
    expect(ringByte(100)).toBe(0x1b);
  });

  it("each bound fader gets its strip's position, the master gets its own, and a channel with no strip sits at the bottom", () => {
    const f = frames(mackiePreset(), [strip("a", { volume_db: 12 }), strip("b", { volume_db: -96 })], 12);
    const bendOf = (ch: number) => f.find((x) => x.key === `v:${ch}`)!.bytes;
    expect(bendOf(0)).toEqual([0xe0, 127, 127]);
    expect(bendOf(1)).toEqual([0xe1, 0, 0]);
    expect(bendOf(2)).toEqual([0xe2, 0, 0]);
    expect(f.find((x) => x.key === "v:0")!.hold).toBe("a:volume");
    expect(f.find((x) => x.key === "v:2")!.hold).toBeUndefined();
    expect(f.find((x) => x.key === "m:8")!.bytes).toEqual([0xe8, 127, 127]);
  });

  it("button lights follow mute, solo and arm, and pan rings follow pan", () => {
    const f = frames(mackiePreset(), [strip("a", { muted: true, solo: false, record_armed: true, pan: 100 })], 0);
    const bytes = (key: string) => f.find((x) => x.key === key)!.bytes;
    expect(bytes("n:0:16")).toEqual([0x90, 16, 127]); // mute lit
    expect(bytes("n:0:8")).toEqual([0x90, 8, 0]); // solo off
    expect(bytes("n:0:0")).toEqual([0x90, 0, 127]); // arm lit
    expect(bytes("r:0:16")).toEqual([0xb0, 48, 0x1b]);
    expect(bytes("r:0:17")).toEqual([0xb0, 49, 0]); // no strip: ring off
  });

  it("controls that are not bound get nothing", () => {
    expect(frames([], [strip("a")], 0)).toEqual([]);
    expect(frames([{ source: { kind: "cc", channel: 0, number: 7 }, target: { type: "volume", track: 0 } }], [strip("a")], 0)).toEqual([]);
  });
});
