// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { describeLooks, isPanelSlot, lookEffects, LOOKS, NO_LOOKS, readLooks, sameLooks, withLooks, withoutLooks, type Looks } from "./audiofx";
import { chainOf } from "../voice-chain/chain";

let n = 0;
const nid = () => `id${++n}`;
const mine = (user = "altaudio.limiter") => ({ id: "u", effect: user, enabled: true, params: {}, preset: null });

describe("looks to effects", () => {
  it("makes real effects for every look at every strength", () => {
    for (const l of LOOKS) {
      for (const s of [1, 2, 3] as const) {
        const e = lookEffects(l.id, s);
        expect(e.length).toBeGreaterThan(0);
        expect(e.every((x) => x.effect.startsWith("altaudio."))).toBe(true);
      }
    }
  });

  it("asks for more as the strength goes up", () => {
    const red = ([1, 2, 3] as const).map((s) => lookEffects("restoration", s)[0]!.params.reduction!);
    expect(red).toEqual([6, 10, 14]);
    const ceil = ([1, 2, 3] as const).map((s) => lookEffects("polish", s)[0]!.params.ceiling!);
    expect(ceil[0]).toBeLessThan(ceil[1]!);
    expect(ceil[1]).toBeLessThan(ceil[2]!);
  });
});

describe("writing and reading the chain", () => {
  const looks: Looks = { restoration: 2, level: 0, clarity: 1, polish: 3 };

  it("puts the panel's effects first, in chain order, and keeps the user's own after them", () => {
    const c = withLooks({ ...chainOf(null), slots: [mine()] }, looks, nid)!;
    const order = c.slots.map((s) => s.preset);
    expect(order).toEqual(["Audio FX: restoration/2", "Audio FX: clarity/1", "Audio FX: clarity/1", "Audio FX: polish/3", null]);
    expect(c.slots[c.slots.length - 1]!.id).toBe("u");
  });

  it("reads the same looks back, and replaces rather than stacks on a second apply", () => {
    const first = withLooks(chainOf(null), looks, nid)!;
    expect(readLooks(first)).toEqual(looks);
    const next: Looks = { ...looks, restoration: 0, level: 1 };
    const second = withLooks(first, next, nid)!;
    expect(readLooks(second)).toEqual(next);
    expect(second.slots.filter(isPanelSlot).every((s) => !s.preset!.includes("restoration"))).toBe(true);
  });

  it("takes only its own slots out", () => {
    const c = withLooks({ ...chainOf(null), slots: [mine()] }, looks, nid)!;
    expect(withoutLooks(c).slots).toEqual([mine()]);
    expect(readLooks(withoutLooks(c))).toEqual(NO_LOOKS);
  });

  it("refuses a chain that would pass 16 effects", () => {
    const full = { ...chainOf(null), slots: Array.from({ length: 15 }, (_, i) => ({ ...mine(), id: `f${i}` })) };
    expect(withLooks(full, { ...NO_LOOKS, clarity: 2 }, nid)).toBeNull();
  });

  it("names the History entry after the looks", () => {
    expect(describeLooks(looks)).toBe("Dialogue: Restoration, Clarity, Polish");
    expect(describeLooks(NO_LOOKS)).toBe("Dialogue: Off");
    expect(sameLooks(looks, { ...looks })).toBe(true);
    expect(sameLooks(looks, NO_LOOKS)).toBe(false);
  });
});
