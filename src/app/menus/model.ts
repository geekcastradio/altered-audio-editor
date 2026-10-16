// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * The menu bar as data: File, Edit, Multitrack, Clip, Effects, View,
 * Window, Help. Item contents and grouping are our own. Every item names
 * an action id; items with no registered action show disabled with the
 * reason in `pending`. Shortcuts come from the keymap, not from here.
 */

import { PANELS } from "../docking/registry";

export type MenuItem =
  | { kind: "item"; id: string; label: string; pending?: string; placeholder?: boolean }
  | { kind: "submenu"; label: string; items: MenuItem[] }
  | { kind: "sep" };

export interface Menu {
  label: string;
  /** Letter that opens the menu with Alt. Must appear in the label. */
  mnemonic: string;
  items: MenuItem[];
}

const item = (id: string, label: string, opts: { pending?: string; placeholder?: boolean } = {}): MenuItem => ({
  kind: "item",
  id,
  label,
  ...opts,
});
const sub = (label: string, items: MenuItem[]): MenuItem => ({ kind: "submenu", label, items });
const sep: MenuItem = { kind: "sep" };

const P2 = "Arrives with playback (P2).";
const P3 = "Arrives with editing (P3).";
const P5 = "Arrives with effects (P5).";
const P7 = "Arrives with podcast publishing (P7).";
const LATER = "Not built yet.";

/** Open Recent slots. Unused slots are hidden. */
export const RECENT_SLOTS = 10;
/** Menu slots for workspaces: the built-in ones plus the user's own. */
export const WORKSPACE_SLOTS = 20;

export const MENUS: readonly Menu[] = [
  {
    label: "File",
    mnemonic: "F",
    items: [
      item("file.new-session", "New Session..."),
      item("file.new-audio", "New Audio File..."),
      item("file.open", "Open..."),
      sub("Open Recent", [
        ...Array.from({ length: RECENT_SLOTS }, (_, i) => item(`file.recent.${i + 1}`, `Recent ${i + 1}`, { placeholder: true })),
        item("file.recent.none", "No recent sessions", { pending: "Sessions you save or open appear here.", placeholder: true }),
      ]),
      sep,
      item("file.import-files", "Import Files..."),
      item("file.import-folder", "Import Folder..."),
      item("file.import-sesx", "Import Session (.sesx)..."),
      sep,
      item("file.save", "Save"),
      item("file.save-as", "Save As..."),
      item("file.save-template", "Save as Session Template..."),
      item("file.save-all", "Save All"),
      sep,
      item("file.close", "Close"),
      item("file.close-all", "Close All"),
      item("file.close-unused", "Close Unused Media"),
      sep,
      sub("Export", [
        item("file.export.mixdown", "Session Mixdown..."),
        item("file.export.selection", "Time Selection..."),
        item("file.export.stems", "Track Stems..."),
      ]),
      item("file.check-loudness", "Check Loudness..."),
      item("file.publish-package", "Publish Package...", { pending: P7 }),
      sep,
      item("file.reveal", "Show in File Manager"),
      sep,
      item("file.exit", "Exit"),
    ],
  },
  {
    label: "Edit",
    mnemonic: "E",
    items: [
      item("edit.undo", "Undo"),
      item("edit.redo", "Redo"),
      sep,
      item("edit.cut", "Cut"),
      item("edit.copy", "Copy"),
      item("edit.paste", "Paste"),
      item("edit.paste-new", "Paste to New File"),
      item("edit.mix-paste", "Mix Paste..."),
      item("edit.delete", "Delete"),
      item("edit.crop", "Crop"),
      sub("Tool", [
        item("tool.move", "Move"),
        item("tool.razor", "Razor"),
        item("tool.slip", "Slip"),
        item("tool.time", "Time Selection"),
        sep,
        item("tool.box", "Box Select (Spectrum)"),
        item("tool.lasso", "Lasso Select (Spectrum)"),
        item("tool.brush", "Brush Select (Spectrum)"),
        item("tool.spot", "Spot Repair (Spectrum)"),
      ]),
      sub("Ripple Delete", [
        item("edit.ripple.clips", "Selected Clips"),
        item("edit.ripple.time-track", "Time Selection in Selected Track"),
        item("edit.ripple.time-all", "Time Selection in All Tracks"),
        item("edit.ripple.gap", "Gap Between Clips"),
      ]),
      sep,
      sub("Select", [
        item("edit.select-all", "Select All"),
        item("edit.deselect", "Deselect All"),
        item("edit.select-track-clips", "All Clips in Track"),
        item("edit.select-next-clip", "Next Clip in Selected Track"),
        item("edit.select-clips-track-end", "Clips to End of Track"),
        item("edit.select-clips-session-start", "Clips to Start of Session"),
        item("edit.select-clips-session-end", "Clips to End of Session"),
        sep,
        item("edit.select-view-time", "Select Current View Time"),
        item("edit.select-all-time", "Select All Time"),
        item("edit.select-time-around-clips", "Select Time Around Selected Clips"),
        item("edit.clear-time-selection", "Clear Time Selection"),
        item("edit.select-to-end", "Playhead to End"),
      ]),
      item("edit.find", "Find..."),
      sub("Markers", [
        item("edit.marker.cue", "Add Cue Marker"),
        item("edit.marker.range", "Add Range Marker"),
        item("edit.marker.chapter", "Add Chapter Marker", { pending: P7 }),
        item("edit.marker.ad-slot", "Add Ad Slot", { pending: P5 }),
        item("edit.marker.social", "Add Social Clip", { pending: P7 }),
      ]),
      sub("Snapping", [
        item("edit.snap.on", "Snap On"),
        sep,
        item("edit.snap.clips", "To Clips"),
        item("edit.snap.playhead", "To Playhead"),
        item("edit.snap.markers", "To Markers"),
        item("edit.snap.ruler", "To Ruler Ticks"),
      ]),
      sep,
      item("edit.keyboard-shortcuts", "Keyboard Shortcuts..."),
      item("edit.settings", "Settings...", { pending: LATER }),
    ],
  },
  {
    label: "Multitrack",
    mnemonic: "M",
    items: [
      sub("Add Track", [
        item("multitrack.add.mono", "Mono Track"),
        item("multitrack.add.stereo", "Stereo Track"),
        item("multitrack.add.guest", "Guest Track", { pending: P5 }),
        item("multitrack.add.bus", "Bus Track", { pending: "Arrives with the mixer (P2)." }),
      ]),
      item("multitrack.duplicate-track", "Duplicate Track"),
      item("multitrack.delete-track", "Delete Track"),
      item("multitrack.delete-empty", "Delete Empty Tracks"),
      sep,
      item("multitrack.insert-files", "Insert Files at Playhead..."),
      item("multitrack.insert-tracks", "Insert Files as Tracks..."),
      item("multitrack.insert-selected-tracks", "Insert Selected Files as Tracks"),
      sep,
      item("multitrack.play-pause", "Play / Pause"),
      item("multitrack.stop", "Stop"),
      item("multitrack.record", "Record"),
      sep,
      item("multitrack.speaker-balance", "Speaker Balance", { pending: P5 }),
      item("multitrack.transcribe", "Transcribe Session...", { pending: LATER }),
      item("multitrack.remove-fillers", "Remove Filler Words...", { pending: P7 }),
      sep,
      sub("Mix Down to New File", [
        item("multitrack.mixdown.all", "Entire Session"),
        item("multitrack.mixdown.selection", "Time Selection"),
        item("multitrack.mixdown.clips", "Selected Clips"),
      ]),
      sub("Bounce to New Track", [
        item("multitrack.bounce.selection", "Time Selection"),
        item("multitrack.bounce.clips", "Selected Clips"),
      ]),
      sep,
      item("multitrack.properties", "Session Properties..."),
    ],
  },
  {
    label: "Clip",
    mnemonic: "C",
    items: [
      item("clip.split", "Split at Playhead"),
      item("clip.split-all", "Split All Clips at Playhead"),
      item("clip.merge", "Merge Clips"),
      item("clip.unique", "Make Unique Copy"),
      item("clip.edit-source", "Open in Waveform Editor"),
      sep,
      item("clip.gain", "Clip Gain..."),
      item("clip.color", "Clip Color..."),
      item("clip.lock", "Lock Position"),
      item("clip.mute", "Mute Clip"),
      item("clip.loop", "Loop Clip"),
      sep,
      sub("Fades", [
        item("clip.fade-in", "Fade In"),
        item("clip.fade-out", "Fade Out"),
        sep,
        item("clip.auto-crossfade", "Automatic Crossfades"),
      ]),
      sub("Trim", [
        item("clip.trim.selection", "To Time Selection"),
        item("clip.trim.start", "Start to Playhead"),
        item("clip.trim.end", "End to Playhead"),
      ]),
      sub("Group", [
        item("clip.group", "Group Clips"),
        item("clip.ungroup", "Ungroup Clips"),
      ]),
      item("clip.stretch", "Time Stretch...", { pending: LATER }),
      sep,
      item("clip.remove-music", "Remove Music...", { pending: P7 }),
      item("clip.reveal", "Show in Files Panel"),
    ],
  },
  {
    label: "Effects",
    mnemonic: "T",
    items: [
      item("window.panel.voice-chain", "Voice Chain"),
      item("window.panel.audio-fx", "Audio FX Panel"),
      sep,
      item("fx.invert", "Invert", { pending: P5 }),
      item("fx.reverse", "Reverse", { pending: P5 }),
      item("fx.silence", "Silence"),
      sub("Spectrum", [
        item("fx.spectrum.reduce-12", "Reduce Selected Area 12 dB"),
        item("fx.spectrum.reduce-24", "Reduce Selected Area 24 dB"),
        item("fx.spectrum.remove", "Remove Selected Area"),
        item("fx.spectrum.repair", "Repair Selected Area"),
      ]),
      sub("Amplitude", [
        item("fx.amplify", "Amplify..."),
        item("fx.normalize", "Normalize..."),
        item("fx.fade-in", "Fade In"),
        item("fx.fade-out", "Fade Out"),
      ]),
      sub("Generate", [
        item("fx.gen.tone", "Tone...", { pending: P5 }),
        item("fx.gen.noise", "Noise...", { pending: P5 }),
        item("fx.gen.silence", "Silence...", { pending: P5 }),
      ]),
      item("fx.loudness-match", "Loudness Match...", { pending: P5 }),
      sep,
      sub("Dynamics", [item("fx.dynamics", "Effects arrive in P5", { pending: P5, placeholder: true })]),
      sub("EQ and Filters", [item("fx.eq", "Effects arrive in P5", { pending: P5, placeholder: true })]),
      sub("Noise and Repair", [item("fx.repair", "Effects arrive in P5", { pending: P5, placeholder: true })]),
      sub("Delay and Reverb", [item("fx.space", "Effects arrive in P5", { pending: P5, placeholder: true })]),
      sub("Pitch and Time", [item("fx.pitch", "Effects arrive in P5", { pending: P5, placeholder: true })]),
      sub("Stereo and Channels", [item("fx.stereo", "Effects arrive in P5", { pending: P5, placeholder: true })]),
      sub("QuickPass", [item("fx.quickpass", "Presets arrive in P5", { pending: P5, placeholder: true })]),
      sep,
      sub("Plug-ins", [item("fx.plugins.none", "No plug-ins scanned", { pending: P5, placeholder: true })]),
      item("fx.plugin-manager", "Plug-in Manager...", { pending: P5 }),
    ],
  },
  {
    label: "View",
    mnemonic: "V",
    items: [
      item("view.editor.multitrack", "Multitrack Editor"),
      item("view.editor.waveform", "Waveform Editor"),
      item("view.spectrum", "Show Spectrum"),
      item("view.pitch", "Show Pitch"),
      sep,
      item("view.zoom-in", "Zoom In"),
      item("view.zoom-out", "Zoom Out"),
      item("view.zoom-session", "Zoom to Session"),
      item("view.zoom-selection", "Zoom to Selection"),
      sub("Time Format", [
        item("view.time.decimal", "Minutes and Seconds"),
        item("view.time.samples", "Samples"),
        item("view.time.timecode", "Timecode (30 fps)"),
      ]),
      sub("Envelopes", [
        item("view.env.volume", "Clip Volume"),
        item("view.env.pan", "Clip Pan"),
      ]),
      sep,
      item("view.mode.podcast", "Podcast Mode", { pending: "Its panels arrive with P7." }),
      item("view.mode.voiceover", "Voiceover Mode", { pending: LATER }),
    ],
  },
  {
    label: "Window",
    mnemonic: "W",
    items: [
      sub("Workspace", [
        ...Array.from({ length: WORKSPACE_SLOTS }, (_, i) => item(`window.workspace.${i + 1}`, `Workspace ${i + 1}`, { placeholder: true })),
        sep,
        item("window.workspace.reset", "Reset Workspace"),
        item("window.workspace.save-changes", "Save Changes to Workspace"),
        item("window.workspace.save-as", "Save as New Workspace..."),
        item("window.workspace.delete", "Delete Workspace"),
      ]),
      sep,
      ...PANELS.map((p) => item(`window.panel.${p.id}`, p.title)),
      sep,
      item("window.reset-layout", "Reset to Default Layout"),
      item("window.minimize", "Minimize"),
    ],
  },
  {
    label: "Help",
    mnemonic: "H",
    items: [
      item("help.guide", "User Guide", { pending: LATER }),
      item("help.shortcuts", "Keyboard Shortcuts"),
      sep,
      item("help.website", "The GCRN Website"),
      item("help.plugins", "GCRN Plugins", { pending: "The plugins page on thegcrn.com is not live yet." }),
      sep,
      item("help.book", "Book: Altered Audio, Podcasting Demystified"),
      item("help.donate", "Support Altered Audio Editor..."),
      sep,
      item("help.logs", "Show Log Files", { pending: LATER }),
      item("help.report", "Report a Problem...", { pending: LATER }),
      sep,
      item("help.about", "About Altered Audio Editor"),
    ],
  },
];

/** Every action id the menus refer to. */
export function menuActionIds(menus: readonly Menu[] = MENUS): string[] {
  const ids: string[] = [];
  const walk = (items: readonly MenuItem[]) => {
    for (const it of items) {
      if (it.kind === "item") ids.push(it.id);
      else if (it.kind === "submenu") walk(it.items);
    }
  };
  for (const m of menus) walk(m.items);
  return ids;
}

/** A command that can take a shortcut, with where it lives in the menus. */
export interface CommandEntry {
  id: string;
  /** Menu path, for example "Edit > Select > Select All". */
  path: string;
}

/** Every real command in the menus, once each, in menu order. */
export function commandList(menus: readonly Menu[] = MENUS): CommandEntry[] {
  const out: CommandEntry[] = [];
  const seen = new Set<string>();
  const walk = (items: readonly MenuItem[], trail: string[]) => {
    for (const it of items) {
      if (it.kind === "submenu") walk(it.items, [...trail, it.label]);
      else if (it.kind === "item" && !it.placeholder && !seen.has(it.id)) {
        seen.add(it.id);
        out.push({ id: it.id, path: [...trail, it.label.replace(/\.\.\.$/, "")].join(" > ") });
      }
    }
  };
  for (const m of menus) walk(m.items, [m.label]);
  return out;
}
