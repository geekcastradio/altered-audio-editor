// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { cleanBindings, mackiePreset, matches, panAt, position, sourceKey, sourceName, targetChoices, targetName, turn, type Message } from "./map";

const msg = (kind: Message["kind"], channel: number, number: number, value: number): Message => ({ port: "p", kind, channel, number, value });

describe("controller map", () => {
  it("reads positions and encoder turns", () => {
    expect(position(msg("cc", 0, 7, 127))).toBe(1);
    expect(position(msg("bend", 0, 0, 16383))).toBe(1);
    expect(position(msg("bend", 0, 0, 0))).toBe(0);
    expect(turn(1)).toBe(1);
    expect(turn(3)).toBe(3);
    expect(turn(65)).toBe(-1);
    expect(turn(127)).toBe(-63);
    expect(turn(0)).toBe(0);
    expect(turn(64)).toBe(0);
  });

  it("pan has a dead zone at the center and reaches both ends", () => {
    expect(panAt(0)).toBe(-100);
    expect(panAt(1)).toBe(100);
    expect(panAt(0.5)).toBe(0);
    expect(panAt(0.51)).toBe(0);
    expect(panAt(0.6)).toBe(20);
  });

  it("matches a binding by kind, channel and number", () => {
    const b = { source: { kind: "cc" as const, channel: 1, number: 7 }, target: { type: "master" as const } };
    expect(matches(b, msg("cc", 1, 7, 5))).toBe(true);
    expect(matches(b, msg("cc", 0, 7, 5))).toBe(false);
    expect(matches(b, msg("note", 1, 7, 5))).toBe(false);
  });

  it("names things for the list", () => {
    expect(sourceName({ kind: "bend", channel: 2, number: 0 })).toBe("Fader, channel 3");
    expect(sourceName({ kind: "note", channel: 0, number: 94 })).toBe("Button 94, channel 1");
    expect(targetName({ type: "volume", track: 0 })).toBe("Track 1 Volume");
    expect(targetName({ type: "bank", by: 8 })).toBe("Next 8 Tracks");
    expect(targetName({ type: "action", action: "multitrack.play-pause" })).toBe("Play / Stop");
    expect(targetChoices(2).filter((t) => t.type === "volume")).toHaveLength(2);
  });

  it("the Mackie layout has eight strips, a master fader and transport, and no source twice", () => {
    const p = mackiePreset();
    expect(p.filter((b) => b.target.type === "volume")).toHaveLength(8);
    expect(p.some((b) => b.target.type === "master")).toBe(true);
    expect(p.filter((b) => b.source.kind === "cc").every((b) => b.relative)).toBe(true);
    expect(new Set(p.map((b) => sourceKey(b.source))).size).toBe(p.length);
    expect(cleanBindings(p)).toEqual(p);
  });

  it("drops damaged bindings and keeps the last of a source bound twice", () => {
    const good = { source: { kind: "cc", channel: 0, number: 1 }, target: { type: "volume", track: 0 } };
    const again = { source: { kind: "cc", channel: 0, number: 1 }, target: { type: "pan", track: 3 }, relative: true };
    const got = cleanBindings([good, again, null, 5, { source: { kind: "cc", channel: 99, number: 1 }, target: { type: "master" } }, { source: { kind: "cc", channel: 0, number: 2 }, target: { type: "bank", by: 0 } }, { source: { kind: "cc", channel: 0, number: 3 }, target: { type: "nope" } }]);
    expect(got).toEqual([again]);
    expect(cleanBindings("x")).toEqual([]);
  });
});
