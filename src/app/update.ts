// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * Help > Check for Updates. The core does the checking (it reads one
 * signed file from the project's GitHub releases); this file only asks it
 * and decides when the optional weekly check is due.
 */
import { invoke } from "@tauri-apps/api/core";

export interface UpdateInfo {
  version: string;
  current: string;
  notes: string | null;
  /** False for the portable zip and non-Windows systems: they get the download page. */
  canInstall: boolean;
}

export const WEEK_MS = 7 * 24 * 60 * 60 * 1000;

/** Looks for a newer version. Null means this one is current. */
export const updateCheck = (): Promise<UpdateInfo | null> => invoke("update_check");

/** Downloads, installs and restarts. Only after a click in the update dialog. */
export const updateInstall = (): Promise<void> => invoke("update_install");

/** The check runs only when turned on. Every start: always. Weekly: when a week has passed since the last one. */
export function updateDue(enabled: boolean, lastCheck: number, now: number, every: "launch" | "week" = "week"): boolean {
  if (!enabled) return false;
  if (every === "launch") return true;
  if (!Number.isFinite(lastCheck) || lastCheck <= 0) return true;
  // A clock set back never blocks the check for good.
  return now < lastCheck || now - lastCheck >= WEEK_MS;
}

/** Release notes written in Markdown, shown as plain text in the update dialog: no #, ** or backticks, bullets as dots. */
export function plainNotes(md: string): string {
  return md
    .replace(/\r\n/g, "\n")
    .replace(/^#{1,6}\s*/gm, "")
    .replace(/\*\*([^*]+)\*\*/g, "$1")
    .replace(/`([^`]+)`/g, "$1")
    .replace(/\[([^\]]+)\]\([^)]+\)/g, "$1")
    .replace(/^\s*[-*]\s+/gm, "\u2022 ")
    .replace(/\n{3,}/g, "\n\n")
    .trim();
}

/**
 * The first few changes from the release notes, one line each, for the update card.
 * Bullets win; with none, the first plain lines are used. Long lines are cut at a word.
 */
export function highlights(md: string | null, max = 4, width = 90): string[] {
  if (!md) return [];
  const lines = plainNotes(md)
    .split("\n")
    .map((l) => l.trim())
    .filter((l) => l !== "");
  const bullets = lines.filter((l) => l.startsWith("•")).map((l) => l.replace(/^•\s*/, ""));
  const pool = bullets.length > 0 ? bullets : lines;
  return pool.slice(0, max).map((l) => {
    if (l.length <= width) return l;
    const cut = l.slice(0, width - 1);
    const sp = cut.lastIndexOf(" ");
    return `${(sp > width / 2 ? cut.slice(0, sp) : cut).trimEnd()}…`;
  });
}
