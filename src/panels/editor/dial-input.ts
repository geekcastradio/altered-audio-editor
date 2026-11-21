// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/** A typed number such as "-2", "+3.5", "−2.0 dB" or "-6dB". Null when there is no number. */
export function parseNumber(text: string): number | null {
  const t = text.trim().replace(/−/g, "-").replace(/,/g, ".");
  const m = /^([+-]?\d+(?:\.\d+)?|[+-]?\.\d+)\s*(?:db|%)?$/i.exec(t);
  if (!m) return null;
  const v = Number(m[1]);
  return Number.isFinite(v) ? v : null;
}

/** A typed pan: "C", "L30", "R20", or a plain number from -100 (left) to 100 (right). */
export function parsePan(text: string): number | null {
  const t = text.trim();
  if (/^c(enter|entre)?$/i.test(t)) return 0;
  const m = /^([lr])\s*(\d+(?:\.\d+)?)$/i.exec(t);
  if (m) return (m[1]!.toLowerCase() === "l" ? -1 : 1) * Number(m[2]);
  return parseNumber(t);
}
