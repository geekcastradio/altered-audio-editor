// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/** The right-click menu of one marker, shared by the ruler flag and the Markers panel. */

import type { ContextItem } from "../../app/ContextMenu.svelte";
import type { MarkerChange, MarkerDoc, MarkerKind } from "../../stores/session";
import { KIND_LABELS, kindChange, rangeLengthFor, togglePlaylist } from "./markers";

const KINDS = Object.keys(KIND_LABELS) as MarkerKind[];

export interface MarkerMenuDeps {
  rate: number;
  /** The time selection, used to size a marker that becomes a range. */
  selection: { start: number; end: number } | null | undefined;
  playlist: readonly string[] | undefined;
  /** Picks the marker and brings the Markers panel forward. Left out inside the panel itself. */
  showInPanel?: () => void;
  goTo(m: MarkerDoc): void;
  rename(m: MarkerDoc): void;
  change(label: string, m: MarkerDoc, change: Omit<MarkerChange, "id">): void;
  /** Left out where there is no playlist (the Waveform editor), which drops the playlist item. */
  setPlaylist?: (ids: string[]) => void;
  remove(m: MarkerDoc): void;
}

export function markerMenuItems(m: MarkerDoc, d: MarkerMenuDeps): ContextItem[] {
  const isRange = m.duration > 0;
  const inList = !!d.playlist?.includes(m.id);
  const setPlaylist = d.setPlaylist;
  const items: ContextItem[] = [{ kind: "item", label: "Go to Marker", run: () => d.goTo(m) }];
  if (d.showInPanel) items.push({ kind: "item", label: "Show in Markers Panel", run: d.showInPanel });
  items.push(
    { kind: "sep" },
    { kind: "item", label: "Rename Marker...", run: () => d.rename(m) },
    ...(setPlaylist
      ? [
          {
            kind: "item",
            label: inList ? "Remove from Playlist" : "Insert into Playlist",
            disabled: !isRange && !inList,
            reason: "Only a range can go in the playlist. Convert it to a range first.",
            run: () => setPlaylist(togglePlaylist(d.playlist, m)),
          } as ContextItem,
        ]
      : []),
    isRange
      ? { kind: "item", label: "Convert to Point", run: () => d.change("Convert to Point Marker", m, { duration: 0 }) }
      : { kind: "item", label: "Convert to Range", run: () => d.change("Convert to Range Marker", m, { duration: rangeLengthFor(m, d.selection, d.rate) }) },
    {
      kind: "sub",
      label: "Change Marker Type",
      items: KINDS.map((k): ContextItem => ({ kind: "item", label: KIND_LABELS[k], checked: m.kind === k, run: () => { if (m.kind !== k) d.change("Marker Type", m, kindChange(m, k, d.rate)); } })),
    },
    { kind: "sep" },
    { kind: "item", label: "Delete Marker", run: () => d.remove(m) },
  );
  return items;
}
