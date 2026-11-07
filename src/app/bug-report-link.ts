// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * Help > Report a Problem opens the project's bug form in the browser with
 * the app version and operating system already filled in. That is all it
 * puts in the link: no errors, no file names, no paths. The person sees the
 * whole form on GitHub and decides what to send. Details such as recent
 * errors come from Help > Copy Bug Report Details, pasted by hand.
 */

const NEW_ISSUE = "https://github.com/geekcastradio/altered-audio-editor/issues/new";

export type ReportOs = "Windows" | "macOS" | "Linux";

/** The operating system named in the browser engine's user agent, or null when it is not one of the three. */
export function osFrom(userAgent: string): ReportOs | null {
  if (/Windows/i.test(userAgent)) return "Windows";
  if (/Macintosh|Mac OS X/i.test(userAgent)) return "macOS";
  if (/Linux|X11/i.test(userAgent)) return "Linux";
  return null;
}

/** The bug form's link, with the app version and operating system filled in where known. */
export function reportUrl(appVersion: string | null, userAgent: string): string {
  const q = new URLSearchParams({ template: "bug_report.yml" });
  if (appVersion && /^[0-9A-Za-z.+-]{1,40}$/.test(appVersion)) q.set("app_version", appVersion);
  const os = osFrom(userAgent);
  if (os) q.set("os", os);
  return `${NEW_ISSUE}?${q.toString()}`;
}
