// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/** The scale marks drawn under or beside a level meter that reaches `range` dB below full scale. */
export function meterTicks(range: number): number[] {
  switch (range) {
    case 24:
      return [-24, -18, -12, -9, -6, -3, 0];
    case 48:
      return [-48, -36, -24, -18, -12, -6, -3, 0];
    case 72:
      return [-72, -60, -48, -36, -24, -12, -6, 0];
    case 96:
      return [-96, -84, -72, -60, -48, -36, -24, -12, 0];
    case 120:
      return [-120, -96, -72, -48, -24, -12, 0];
    default:
      return [-60, -48, -36, -24, -18, -12, -9, -6, -3, 0];
  }
}

/** Where a level sits along a meter that reaches `range` dB down, as a percentage (0 at the floor). */
export function meterPos(db: number, range: number): number {
  return Math.max(0, Math.min(100, ((db + range) / range) * 100));
}

/** The menu text for a meter depth. */
export function rangeLabel(range: number): string {
  return `-${range} dB`;
}
