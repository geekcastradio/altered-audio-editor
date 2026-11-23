// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/** The theme choices, in the order the tool strip shows them, with their hover text. */
export type ThemeChoice = "light" | "system" | "dark";

export const THEME_CHOICES: readonly { id: ThemeChoice; label: string; help: string }[] = [
  { id: "light", label: "Light Mode", help: "Light panels, for bright rooms." },
  { id: "system", label: "Sync with System", help: "Follow your computer's light or dark setting, and change when it does." },
  { id: "dark", label: "Dark Mode", help: "Dark panels, easier on the eyes for long sessions." },
];

/** The theme to draw: light or dark. `systemLight` is whether the computer is set to light. */
export function effectiveTheme(choice: ThemeChoice, systemLight: boolean): "light" | "dark" {
  if (choice === "system") return systemLight ? "light" : "dark";
  return choice;
}

/** Whether the computer is set to a light theme. Read locally; nothing is sent anywhere. */
export function systemIsLight(): boolean {
  try {
    return typeof window !== "undefined" && typeof window.matchMedia === "function" && window.matchMedia("(prefers-color-scheme: light)").matches;
  } catch {
    return false;
  }
}

/** Calls `fn` when the computer's light or dark setting changes. Returns a way to stop. */
export function watchSystemTheme(fn: () => void): () => void {
  try {
    if (typeof window === "undefined" || typeof window.matchMedia !== "function") return () => {};
    const q = window.matchMedia("(prefers-color-scheme: light)");
    q.addEventListener("change", fn);
    return () => q.removeEventListener("change", fn);
  } catch {
    return () => {};
  }
}
