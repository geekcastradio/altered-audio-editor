// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * What the screen tells a control surface: where each motorized fader
 * should sit, and which buttons and pan rings are lit. Only controls that
 * have a binding get anything, and nothing is sent unless the person
 * switched feedback on for their devices.
 *
 * The messages follow the Mackie Control layout the preset uses: a
 * fader's position is a pitch bend on its channel, a button light is the
 * note the button sends (velocity 127 on, 0 off), and a pan ring is a
 * control change 32 above the knob's own number.
 */

import { dbToFader } from "../../panels/mixer/fader";
import type { Binding } from "./map";

/** The parts of a track or bus a surface shows. */
export interface StripState {
  id: string;
  volume_db: number;
  pan: number;
  muted?: boolean;
  solo?: boolean;
  record_armed?: boolean;
}

/** One message to send, and what it describes. */
export interface Frame {
  /** The same control always has the same key, so a repeat can be skipped. */
  key: string;
  bytes: number[];
  /** For a fader: the strip key (`<id>:volume`) that must be let go before this is sent. */
  hold?: string;
}

/** A fader position, 0 to 1, as a 14-bit pitch bend value. */
export function fader14(pos: number): number {
  return Math.round(Math.min(1, Math.max(0, pos)) * 16383);
}

/** The Mackie pan ring value for a pan of -100 to 100: boost/cut display, a lit point from 1 (left) to 11 (right). */
export function ringByte(pan: number): number {
  const p = Math.min(100, Math.max(-100, pan));
  return (1 << 4) | (Math.round(((p + 100) / 200) * 10) + 1);
}

const bend = (channel: number, value: number): number[] => [0xe0 | channel, value & 0x7f, value >> 7];

/**
 * Everything the surface should show for these bindings. `strips[i]` is
 * the strip on the surface's channel `i` (already offset by the bank);
 * a channel with no strip gets its fader at the bottom and its lights off.
 */
export function frames(bindings: readonly Binding[], strips: readonly (StripState | undefined)[], masterDb: number): Frame[] {
  const out: Frame[] = [];
  for (const b of bindings) {
    const t = b.target;
    const s = "track" in t ? strips[t.track] : undefined;
    const { kind, channel, number } = b.source;
    if (t.type === "volume" && kind === "bend" && !b.relative) {
      out.push({ key: `v:${channel}`, bytes: bend(channel, fader14(s ? dbToFader(s.volume_db) : 0)), ...(s ? { hold: `${s.id}:volume` } : {}) });
    } else if (t.type === "master" && kind === "bend" && !b.relative) {
      out.push({ key: `m:${channel}`, bytes: bend(channel, fader14(dbToFader(masterDb))) });
    } else if ((t.type === "mute" || t.type === "solo" || t.type === "arm") && kind === "note") {
      const on = t.type === "mute" ? s?.muted === true : t.type === "solo" ? s?.solo === true : s?.record_armed === true;
      out.push({ key: `n:${channel}:${number}`, bytes: [0x90 | channel, number, on ? 127 : 0] });
    } else if (t.type === "pan" && kind === "cc" && b.relative && number >= 16 && number <= 23) {
      out.push({ key: `r:${channel}:${number}`, bytes: [0xb0 | channel, number + 32, s ? ringByte(s.pan) : 0] });
    }
  }
  return out;
}
