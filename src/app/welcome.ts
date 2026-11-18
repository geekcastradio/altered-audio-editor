// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * The welcome dialog's pages. It shows on a fresh install (every page), and again only when a
 * release adds a page with a higher `rev` than the one the person last saw (just the new pages).
 * Most releases add no page; add one, and raise WELCOME_REV, only for a change worth stopping for.
 *
 * A page's picture is a PNG in `src/assets/welcome/` named by `image` (for example
 * "automation" is `automation.png`). A page whose picture file is missing shows no picture.
 */
export interface WelcomePage {
  /** The release that added the page. 0 means install only, never shown on an upgrade. */
  rev: number;
  title: string;
  /** Short paragraphs. */
  body: string[];
  /** Picture name, without extension. */
  image?: string;
}

export const WELCOME_PAGES: readonly WelcomePage[] = [
  {
    rev: 0,
    title: "Welcome to Altered Audio Editor",
    body: [
      "A free audio editor for podcasters and voiceover. Your first episode: File > New Session, then File > Import Files (or drag files in). Space bar plays and stops, and Clip > Split at Playhead cuts.",
      "Add effects in the Voice Chain (Window > Voice Chain). When it sounds right, File > Export > Session Mixdown.",
      "No account, no tracking. Your audio stays on your computer.",
    ],
  },
  {
    rev: 1,
    title: "Record automation by moving a control",
    body: [
      "Set a track, bus or effect to Write, Latch or Touch, press play, and move the volume, pan or an effect slider. The moves become automation lines you can edit afterwards.",
      "Auto-thin in Settings keeps the lines light. The User Guide has a section on it.",
    ],
    image: "automation",
  },
  {
    rev: 2,
    title: "Hardware controllers",
    body: [
      "Plug in a MIDI control surface and open Edit > Controllers. Faders, knobs and buttons can run volume, pan, mute, solo, arm and transport. Mackie layout is built in, and Learn binds anything else.",
    ],
    image: "controllers",
  },
  {
    rev: 3,
    title: "Comp the best takes",
    body: [
      "Record a line more than once, select the clips and choose Clip > Make Takes from Selected Clips. The Takes panel lets you pick which take plays over each stretch, and Clip > Flatten Takes keeps your picks.",
      "The User Guide has a section on takes and comping, and one on multi-mic shows.",
    ],
  },
  {
    rev: 4,
    title: "Chains and batches",
    body: [
      "A process chain is a short list of steps run on a session as one undo step: Multitrack > Run Process Chain. The Batch Convert panel runs a chain over a whole list of files, to a loudness target if you want one.",
    ],
  },
  {
    rev: 5,
    title: "Cleaner voices",
    body: [
      "Speech Denoiser (High Quality), in the Restoration group, is a larger network for hard noise such as keyboards and traffic. It is built in: nothing is downloaded and nothing leaves your computer.",
    ],
  },
  {
    rev: 6,
    title: "Quicker clean-up, tidier views",
    body: [
      "Fill Single Click (Effects menu, Ctrl+Alt+U) smooths over one selected click, and Shift+R repeats it on the next one. In the Waveform Editor the waveform now moves as you drag the level knob.",
      "Track Visibility has Save and Load presets. Suspend Groups (Ctrl+Shift+G) lets you pick one clip from a group. Meter Input Signal (Alt+I) checks every mic before you arm it. Guest Track adds a track with the Remote Guest cleanup ready, and Podcast Mode and Voiceover Mode in the View menu switch layouts.",
    ],
  },
];

/** Raise this with the newest page's rev. */
export const WELCOME_REV = Math.max(...WELCOME_PAGES.map((p) => p.rev));

export interface WelcomePlan {
  kind: "install" | "upgrade";
  pages: WelcomePage[];
}

/**
 * What to show at start-up, from the settings as they were saved (or nothing saved yet).
 * Settings from before the welcome existed count as having seen revision 1.
 */
export function welcomePlan(saved: unknown, pages: readonly WelcomePage[] = WELCOME_PAGES): WelcomePlan | null {
  if (saved === null || saved === undefined) return { kind: "install", pages: [...pages] };
  if (typeof saved !== "object") return null;
  const o = saved as { welcomeRev?: unknown; welcomeSeen?: unknown };
  const seen =
    typeof o.welcomeRev === "number" && Number.isFinite(o.welcomeRev)
      ? Math.max(0, Math.floor(o.welcomeRev))
      : o.welcomeSeen === false
        ? 0
        : 1;
  if (seen === 0) return { kind: "install", pages: [...pages] };
  const news = pages.filter((p) => p.rev > seen);
  return news.length ? { kind: "upgrade", pages: news } : null;
}
