/*
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  The look choice: Light, Sync with System or Dark. The header switcher and
  the View menu both use these entries. Plain moon, sun and monitor shapes
  are drawn in ThemeSwitcher.svelte.
*/

export type ThemeChoice = "light" | "system" | "dark";

export interface ThemeEntry {
  id: ThemeChoice;
  label: string;
  hint: string;
}

export const THEMES: readonly ThemeEntry[] = [
  { id: "light", label: "Light Mode", hint: "Light Mode: a light surface with dark text. Easier to read in a bright room." },
  { id: "system", label: "Sync with System", hint: "Sync with System: follow the light or dark setting of your operating system, and change when it changes." },
  { id: "dark", label: "Dark Mode", hint: "Dark Mode: a dark surface tuned for long sessions in a dim room." },
];

/** The theme to paint for a choice. Anything unknown falls back to dark. */
export function resolveTheme(choice: string, systemDark: boolean): "light" | "dark" {
  if (choice === "light") return "light";
  if (choice === "system") return systemDark ? "dark" : "light";
  return "dark";
}
