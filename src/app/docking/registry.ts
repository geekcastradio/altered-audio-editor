// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * Every dockable panel, where it lives in the Default layout, and where
 * it returns to when the user turns it back on.
 *
 * Pure data and logic, no DOM, so it can be unit tested.
 */

export type PanelId =
  | "files"
  | "voice-chain"
  | "batch-convert"
  | "quickpass"
  | "editor"
  | "mixer"
  | "levels"
  | "audio-fx"
  | "sound-types"
  | "time-info"
  | "history"
  | "drive-manager"
  | "libraries"
  | "markers"
  | "metadata"
  | "playlist"
  | "loudness-match"
  | "properties"
  | "phase-wheel"
  | "space";

export type Side = "left" | "right" | "above" | "below";

/** Where to put a panel: next to a panel, inside its tab group, or at an edge of the window. */
export type Placement =
  { reference: PanelId; direction: Side | "within" } | { edge: Side };

export interface PanelDef {
  id: PanelId;
  title: string;
  /** Shown in the Default layout. */
  inDefault: boolean;
  /** Where the panel goes when turned on, first match wins. An edge always matches. */
  home: Placement[];
  /** Short note shown while the panel has no content yet. Null when built. */
  pending: string | null;
}

const LEFT_TABS: Placement[] = [
  { reference: "voice-chain", direction: "within" },
  { reference: "batch-convert", direction: "within" },
  { reference: "quickpass", direction: "within" },
  { reference: "files", direction: "below" },
  { reference: "editor", direction: "left" },
  { edge: "left" },
];

export const PANELS: readonly PanelDef[] = [
  {
    id: "files",
    title: "Files",
    inDefault: true,
    home: [
      { reference: "voice-chain", direction: "above" },
      { reference: "editor", direction: "left" },
      { edge: "left" },
    ],
    pending: null,
  },
  {
    id: "voice-chain",
    title: "Voice Chain",
    inDefault: true,
    home: LEFT_TABS,
    pending: null,
  },
  {
    id: "batch-convert",
    title: "Batch Convert",
    inDefault: true,
    home: LEFT_TABS,
    pending: null,
  },
  {
    id: "quickpass",
    title: "QuickPass",
    inDefault: true,
    home: LEFT_TABS,
    pending: null,
  },
  {
    id: "editor",
    title: "Editor",
    inDefault: true,
    home: [{ reference: "mixer", direction: "within" }, { edge: "right" }],
    pending: "The multitrack view arrives in P1 task 10.",
  },
  {
    id: "mixer",
    title: "Mixer",
    inDefault: true,
    home: [{ reference: "editor", direction: "within" }, { edge: "right" }],
    pending: null,
  },
  {
    id: "levels",
    title: "Levels",
    inDefault: true,
    home: [{ edge: "below" }],
    pending: null,
  },
  {
    id: "audio-fx",
    title: "Audio FX",
    inDefault: true,
    home: [
      { reference: "libraries", direction: "above" },
      { reference: "editor", direction: "right" },
      { edge: "right" },
    ],
    pending: null,
  },
  {
    id: "sound-types",
    title: "Sound Types",
    inDefault: false,
    home: [{ reference: "audio-fx", direction: "within" }, { reference: "editor", direction: "right" }, { edge: "right" }],
    pending: null,
  },
  {
    id: "libraries",
    title: "Libraries",
    inDefault: true,
    home: [
      { reference: "audio-fx", direction: "below" },
      { reference: "editor", direction: "right" },
      { edge: "right" },
    ],
    pending: null,
  },
  {
    id: "time-info",
    title: "Time Info",
    inDefault: true,
    home: [{ reference: "levels", direction: "right" }, { edge: "below" }],
    pending: null,
  },
  {
    id: "history",
    title: "History",
    inDefault: false,
    home: LEFT_TABS,
    pending: null,
  },
  {
    id: "drive-manager",
    title: "Drive Manager",
    inDefault: false,
    home: LEFT_TABS,
    pending: null,
  },
  {
    id: "markers",
    title: "Markers",
    inDefault: false,
    home: LEFT_TABS,
    pending: null,
  },
  {
    id: "metadata",
    title: "Metadata",
    inDefault: false,
    home: LEFT_TABS,
    pending: null,
  },
  {
    id: "playlist",
    title: "Playlist",
    inDefault: false,
    home: LEFT_TABS,
    pending: null,
  },
  {
    id: "loudness-match",
    title: "Loudness Match",
    inDefault: false,
    home: LEFT_TABS,
    pending: "Loudness Match is in the Effects menu.",
  },
  {
    id: "properties",
    title: "Properties",
    inDefault: false,
    home: LEFT_TABS,
    pending: null,
  },
  {
    id: "phase-wheel",
    title: "Phase Wheel",
    inDefault: false,
    home: [{ reference: "audio-fx", direction: "within" }, { reference: "levels", direction: "right" }, { edge: "right" }],
    pending: null,
  },
  {
    id: "space",
    title: "3D Space",
    inDefault: false,
    // Beside the Editor and Mixer: the room needs room.
    home: [{ reference: "editor", direction: "within" }, { reference: "mixer", direction: "within" }, { edge: "right" }],
    pending: null,
  },
];

const BY_ID = new Map(PANELS.map((p) => [p.id, p]));

export function panelDef(id: string): PanelDef | undefined {
  return BY_ID.get(id as PanelId);
}

/** Picks the first home placement that is possible with the panels currently open. */
export function resolveHome(id: PanelId, open: ReadonlySet<string>): Placement {
  const def = BY_ID.get(id);
  for (const place of def?.home ?? []) {
    if ("edge" in place) return place;
    if (place.reference !== id && open.has(place.reference)) return place;
  }
  return { edge: "right" };
}

/** Default layout sizes in pixels. */
export const DEFAULT_SIZES = {
  leftWidth: 270,
  rightWidth: 270,
  levelsHeight: 128,
  timeInfoWidth: 270,
} as const;

/** One step of building the Default layout. */
export interface LayoutStep {
  id: PanelId;
  placement: Placement | null;
  inactive?: boolean;
  width?: number;
  height?: number;
}

/**
 * The Default layout as an ordered list of add operations.
 * Order matters: the center and right columns and the Levels row under
 * them are built first, then the left column is added at the window's
 * left edge so it runs the full height. Levels sits
 * only under the center and right columns, with Time Info at its right
 * end under the right column.
 */
export const DEFAULT_LAYOUT: readonly LayoutStep[] = [
  { id: "editor", placement: null },
  {
    id: "mixer",
    placement: { reference: "editor", direction: "within" },
    inactive: true,
  },
  {
    id: "audio-fx",
    placement: { reference: "editor", direction: "right" },
    width: DEFAULT_SIZES.rightWidth,
  },
  { id: "libraries", placement: { reference: "audio-fx", direction: "below" } },
  {
    id: "levels",
    placement: { edge: "below" },
    height: DEFAULT_SIZES.levelsHeight,
  },
  {
    id: "time-info",
    placement: { reference: "levels", direction: "right" },
    width: DEFAULT_SIZES.timeInfoWidth,
  },
  { id: "files", placement: { edge: "left" }, width: DEFAULT_SIZES.leftWidth },
  { id: "voice-chain", placement: { reference: "files", direction: "below" } },
  {
    id: "batch-convert",
    placement: { reference: "voice-chain", direction: "within" },
    inactive: true,
  },
  {
    id: "quickpass",
    placement: { reference: "voice-chain", direction: "within" },
    inactive: true,
  },
];

/**
 * Bump when DEFAULT_LAYOUT changes. A saved layout from an older
 * Default is replaced by the new Default on next launch.
 */
export const DEFAULT_LAYOUT_VERSION = 6;

/** A workspace that ships with the app. */
export interface WorkspacePreset {
  id: string;
  name: string;
  /** One line for menus and tooltips. */
  description: string;
  steps: readonly LayoutStep[];
  /** Bump when `steps` changes, so saved arrangements of it are rebuilt. */
  version: number;
}

const S = DEFAULT_SIZES;

/**
 * Built-in workspaces. Each is built like the Default: center, right
 * and bottom first, then the left column at the window edge so it runs
 * the full height. Names describe the job, not another product.
 */
export const WORKSPACE_PRESETS: readonly WorkspacePreset[] = [
  {
    id: "default",
    name: "Default",
    description:
      "Everything for a podcast edit: files, effects, editor, libraries and levels.",
    steps: DEFAULT_LAYOUT,
    version: DEFAULT_LAYOUT_VERSION,
  },
  {
    id: "focused-edit",
    name: "Focused Edit",
    description:
      "As much room as possible for the timeline. Files on the left, levels below.",
    version: 1,
    steps: [
      { id: "editor", placement: null },
      {
        id: "mixer",
        placement: { reference: "editor", direction: "within" },
        inactive: true,
      },
      { id: "levels", placement: { edge: "below" }, height: S.levelsHeight },
      {
        id: "time-info",
        placement: { reference: "levels", direction: "right" },
        width: S.timeInfoWidth,
      },
      { id: "files", placement: { edge: "left" }, width: 220 },
    ],
  },
  {
    id: "mixing",
    name: "Mixing",
    description:
      "The Mixer up front with tall meters, files and markers on the left.",
    version: 2,
    steps: [
      // Editor stays the first tab; the Mixer is added after it and comes to the front.
      { id: "editor", placement: null },
      { id: "mixer", placement: { reference: "editor", direction: "within" } },
      {
        id: "libraries",
        placement: { reference: "editor", direction: "right" },
        width: S.rightWidth,
      },
      { id: "levels", placement: { edge: "below" }, height: 170 },
      {
        id: "time-info",
        placement: { reference: "levels", direction: "right" },
        width: S.timeInfoWidth,
      },
      { id: "files", placement: { edge: "left" }, width: S.leftWidth },
      { id: "markers", placement: { reference: "files", direction: "below" } },
    ],
  },
  {
    id: "voiceover",
    name: "Voiceover",
    description:
      "Takes and markers on the left, the editor in the middle, libraries on the right.",
    version: 1,
    steps: [
      { id: "editor", placement: null },
      {
        id: "mixer",
        placement: { reference: "editor", direction: "within" },
        inactive: true,
      },
      {
        id: "libraries",
        placement: { reference: "editor", direction: "right" },
        width: S.rightWidth,
      },
      { id: "levels", placement: { edge: "below" }, height: S.levelsHeight },
      {
        id: "time-info",
        placement: { reference: "levels", direction: "right" },
        width: S.timeInfoWidth,
      },
      { id: "files", placement: { edge: "left" }, width: S.leftWidth },
      { id: "markers", placement: { reference: "files", direction: "below" } },
      {
        id: "voice-chain",
        placement: { reference: "markers", direction: "within" },
        inactive: true,
      },
    ],
  },
  {
    id: "music",
    name: "Music",
    description:
      "For music and MIDI work: the editor and Mixer, properties on the right, voice chain on the left. Turns on Music Mode.",
    version: 1,
    steps: [
      { id: "editor", placement: null },
      {
        id: "mixer",
        placement: { reference: "editor", direction: "within" },
        inactive: true,
      },
      {
        id: "properties",
        placement: { reference: "editor", direction: "right" },
        width: S.rightWidth,
      },
      { id: "levels", placement: { edge: "below" }, height: S.levelsHeight },
      {
        id: "time-info",
        placement: { reference: "levels", direction: "right" },
        width: S.timeInfoWidth,
      },
      { id: "files", placement: { edge: "left" }, width: S.leftWidth },
      { id: "voice-chain", placement: { reference: "files", direction: "below" } },
    ],
  },
  {
    id: "review",
    name: "Review and Export",
    description:
      "For the final listen: markers and history on the left, wide meters below.",
    version: 1,
    steps: [
      { id: "editor", placement: null },
      {
        id: "mixer",
        placement: { reference: "editor", direction: "within" },
        inactive: true,
      },
      { id: "levels", placement: { edge: "below" }, height: 150 },
      {
        id: "time-info",
        placement: { reference: "levels", direction: "right" },
        width: S.timeInfoWidth,
      },
      { id: "markers", placement: { edge: "left" }, width: S.leftWidth },
      {
        id: "history",
        placement: { reference: "markers", direction: "below" },
      },
    ],
  },
];

export function presetById(id: string): WorkspacePreset | undefined {
  return WORKSPACE_PRESETS.find((p) => p.id === id);
}

/** A file-safe id for a new workspace name that does not clash with `taken`. */
export function workspaceId(name: string, taken: ReadonlySet<string>): string {
  const base =
    name
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-+|-+$/g, "")
      .slice(0, 48) || "workspace";
  let id = base;
  for (let n = 2; taken.has(id); n++) id = `${base}-${n}`;
  return id;
}

/**
 * The smallest a panel is allowed to get, in pixels, so its controls never
 * squeeze together. A group holding several panels takes the largest of them.
 */
export const MIN_SIZE: Record<PanelId, { w: number; h: number }> = {
  files: { w: 280, h: 150 },
  "voice-chain": { w: 300, h: 260 },
  "batch-convert": { w: 300, h: 220 },
  quickpass: { w: 300, h: 220 },
  editor: { w: 520, h: 260 },
  mixer: { w: 360, h: 220 },
  levels: { w: 260, h: 70 },
  "audio-fx": { w: 280, h: 160 },
  "sound-types": { w: 280, h: 160 },
  "time-info": { w: 220, h: 110 },
  history: { w: 220, h: 120 },
  "drive-manager": { w: 260, h: 140 },
  libraries: { w: 260, h: 200 },
  markers: { w: 260, h: 140 },
  metadata: { w: 280, h: 200 },
  playlist: { w: 260, h: 140 },
  "loudness-match": { w: 280, h: 160 },
  properties: { w: 260, h: 160 },
  "phase-wheel": { w: 200, h: 160 },
  space: { w: 260, h: 220 },
};

/** The size limit for a group of panels. */
export function groupMin(ids: readonly PanelId[]): { w: number; h: number } {
  return ids.reduce((m, id) => ({ w: Math.max(m.w, MIN_SIZE[id]?.w ?? 0), h: Math.max(m.h, MIN_SIZE[id]?.h ?? 0) }), { w: 0, h: 0 });
}
