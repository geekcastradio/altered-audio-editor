// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * The last few errors the app showed, kept in memory so Help > Copy
 * Bug Report Details can put them (with their stack traces) on the clipboard for
 * a bug report. Nothing is stored or sent anywhere.
 */

export interface RecordedError {
  at: string;
  message: string;
  stack: string;
}

const KEEP = 20;
const STACK_LINES = 14;
const errors: RecordedError[] = [];

/** Home folders in text become `<user>`, so a report does not carry the user's name. */
export function scrub(text: string): string {
  return text
    .replace(/[A-Za-z]:[\\/]+Users[\\/]+[^\\/\s:)]+/gi, "<user>")
    .replace(/\/(?:home|Users)\/[^/\s:)]+/g, "<user>");
}

export function recordError(e: unknown, now: Date = new Date()): void {
  const err = e instanceof Error ? e : null;
  errors.push({
    at: now.toISOString(),
    message: scrub(err ? err.message : String(e)),
    stack: scrub(err?.stack?.split("\n").slice(0, STACK_LINES).join("\n") ?? ""),
  });
  if (errors.length > KEEP) errors.shift();
}

export function recordedErrors(): readonly RecordedError[] {
  return errors;
}

export interface BugInfoContext {
  appVersion: string;
  system: string;
  /** Which editor is showing, and whether audio plays. */
  state: string[];
  /** Plug-in effect ids in the open documents. */
  plugins: string[];
}

export function bugInfoText(ctx: BugInfoContext, list: readonly RecordedError[] = errors): string {
  const out = [`Altered Audio Editor ${ctx.appVersion}`, scrub(ctx.system), ...ctx.state];
  out.push(ctx.plugins.length ? `Plug-ins in use: ${ctx.plugins.join(", ")}` : "Plug-ins in use: none");
  if (list.length === 0) {
    out.push("", "No errors recorded since the app started.");
  } else {
    out.push("", `Errors since the app started (newest last, ${list.length}):`);
    for (const e of list) out.push("", `[${e.at}] ${e.message}`, ...(e.stack ? [e.stack] : []));
  }
  return out.join("\n");
}
