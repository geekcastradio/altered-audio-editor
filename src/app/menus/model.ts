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
const NEXT_FX = "Arrives in the next effects batch.";
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
        sep,
        item("file.export.markers", "Markers and Chapters..."),
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
        sub("Clips of Audio Type", [
          item("edit.select-type.dialogue", "Dialogue"),
          item("edit.select-type.music", "Music"),
          item("edit.select-type.sfx", "Sound Effects"),
          item("edit.select-type.ambience", "Ambience"),
        ]),
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
        item("edit.marker.chapter", "Add Chapter Marker"),
        item("edit.marker.ad-slot", "Add Ad Slot..."),
        item("edit.marker.social", "Add Social Clip"),
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
        item("multitrack.add-tracks", "Add Tracks..."),
        item("multitrack.add.guest", "Guest Track", { pending: P5 }),
        item("multitrack.add.bus", "Bus Track", { pending: P5 }),
      ]),
      item("multitrack.duplicate-track", "Duplicate Track"),
      item("multitrack.delete-track", "Delete Track"),
      item("multitrack.delete-empty", "Delete Empty Tracks"),
      sep,
      item("multitrack.insert-files", "Insert Files at Playhead..."),
      item("multitrack.insert-tracks", "Insert Files as Tracks..."),
      item("multitrack.insert-selected-tracks", "Insert Selected Files as Tracks"),
      item("multitrack.insert-silence", "Insert Silence at Playhead..."),
      sep,
      item("multitrack.play-pause", "Play / Pause"),
      item("multitrack.stop", "Stop"),
      item("multitrack.record", "Record"),
      item("multitrack.record-options", "Recording Options..."),
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
      sub("Audio Type", [
        item("clip.type.dialogue", "Dialogue"),
        item("clip.type.music", "Music"),
        item("clip.type.sfx", "Sound Effects"),
        item("clip.type.ambience", "Ambience"),
        sep,
        item("clip.type.none", "None"),
      ]),
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
      item("clip.stretch", "Time Stretch..."),
      item("clip.music-fit", "Fit Music to Length..."),
      item("clip.stretch-reset", "Remove Stretch"),
      item("clip.phase-align", "Phase Align"),
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
        item("fx.add.gain", "Gain (in the Voice Chain)"),
        item("fx.amplify", "Amplify..."),
        item("fx.normalize", "Normalize..."),
        item("fx.fade-in", "Fade In"),
        item("fx.fade-out", "Fade Out"),
      ]),
      sub("Generate", [
        item("fx.gen.tone", "Tone..."),
        item("fx.gen.noise", "Noise..."),
        item("fx.gen.silence", "Silence..."),
      ]),
      item("fx.loudness-match", "Loudness Match..."),
      sep,
      sub("Dynamics", [
        item("fx.add.dynamics", "Dynamics"),
        item("fx.add.limiter", "Limiter"),
        item("fx.add.breath_gate", "Breath Gate"),
        item("fx.add.deesser", "De-Esser"),
        item("fx.add.speech_leveler", "Speech Leveler"),
      ]),
      sub("EQ and Filters", [
        item("fx.add.parametric_eq", "Parametric EQ"),
        item("fx.add.band_eq_10", "Band EQ (10 bands)"),
        item("fx.add.band_eq_20", "Band EQ (20 bands)"),
        item("fx.add.band_eq_30", "Band EQ (30 bands)"),
        item("fx.add.curve_eq", "Curve EQ"),
        item("fx.add.precision_filter", "Precision Filter"),
        item("fx.add.notch", "Notch Filter"),
        item("fx.add.hum_cut", "Hum Cut"),
      ]),
      sub("Noise and Repair", [
        item("fx.add.noise_reducer", "Noise Reducer"),
        item("fx.add.noise_print", "Noise Print Reducer"),
        item("fx.add.hiss_reducer", "Hiss Reducer"),
        item("fx.add.room_reducer", "Room Reducer"),
        item("fx.add.click_repair", "Click Repair"),
        item("fx.add.clip_repair", "Clip Repair"),
        item("fx.add.dc_offset", "DC Offset Fix"),
        item("fx.add.sound_eraser", "Sound Eraser"),
      ]),
      sub("Delay and Reverb", [item("fx.space", "Effects arrive in P5", { pending: P5, placeholder: true })]),
      sub("Pitch and Time", [
        item("fx.add.pitch_shift", "Pitch Shift"),
        item("clip.stretch", "Time Stretch..."),
        item("clip.music-fit", "Fit Music to Length..."),
        item("fx.pitch-tune", "Pitch Tune (automatic correction)", { pending: NEXT_FX }),
        item("fx.pitch-curve", "Pitch Curve (pitch over time)", { pending: NEXT_FX }),
      ]),
      sub("Stereo and Channels", [item("fx.add.center_remover", "Center Remover (remove vocals)")]),
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
