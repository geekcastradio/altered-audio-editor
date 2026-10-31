// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/** An audio file open in the Waveform editor, shown in the Files panel. */
export interface WaveRow {
  path: string;
  name: string;
  /** Container, for example "WAV". */
  format: string;
  /** Never saved anywhere yet. */
  untitled: boolean;
  dirty: boolean;
  /** The file the Waveform editor shows now. False for files set aside. */
  front: boolean;
}

/** A file set aside while another one is open. */
export interface ParkedFile {
  path: string;
  format: string;
  untitled: boolean;
  dirty: boolean;
}

const base = (p: string) => p.split(/[\\/]/).pop() || p;
const same = (a: string, b: string) => a.toLowerCase() === b.toLowerCase();

/**
 * Rows for the audio files open in the Waveform editor: the one in front,
 * then the ones set aside. Files already listed as pool items (opened from
 * the list itself) are left out, so nothing shows twice.
 */
export function waveRows(
  current: { path: string; format: string; untitled: boolean } | null,
  currentDirty: boolean,
  parked: readonly ParkedFile[],
  listedPaths: readonly string[],
): WaveRow[] {
  const listed = (p: string) => listedPaths.some((x) => same(x, p));
  const rows: WaveRow[] = [];
  if (current && !listed(current.path)) {
    rows.push({ path: current.path, name: base(current.path), format: current.format, untitled: current.untitled, dirty: currentDirty, front: true });
  }
  for (const p of parked) {
    if (listed(p.path)) continue;
    rows.push({ path: p.path, name: base(p.path), format: p.format, untitled: p.untitled, dirty: p.dirty, front: false });
  }
  return rows;
}
