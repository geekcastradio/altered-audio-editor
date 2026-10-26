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

/** The weekly check runs only when turned on and a week has passed since the last one. */
export function updateDue(enabled: boolean, lastCheck: number, now: number): boolean {
  if (!enabled) return false;
  if (!Number.isFinite(lastCheck) || lastCheck <= 0) return true;
  // A clock set back never blocks the check for good.
  return now < lastCheck || now - lastCheck >= WEEK_MS;
}
