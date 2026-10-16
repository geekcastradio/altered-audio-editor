// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * Batch Convert's list and naming rules. The list lives here rather than
 * in the panel so it survives the panel being closed and reopened.
 */

import { writable } from "svelte/store";
import { DEFAULT_FORMAT, extensionOf, sanitizeFormat, type FormatKind, type OutputFormat } from "../../app/output-format";

export type RowStatus = "waiting" | "running" | "done" | "failed" | "stopped";

export interface BatchRow {
  id: string;
  source: string;
  status: RowStatus;
  /** 0 to 1 while running. */
  progress: number;
  error: string | null;
}

export interface BatchSettings {
  format: OutputFormat;
  /** How output files are named: text plus {name}, {number}, {date} and {format}. */
  template: string;
  /** Null writes next to each source file. */
  folder: string | null;
  overwrite: boolean;
  removeDone: boolean;
}

export const DEFAULT_TEMPLATE = "{name}";

export const DEFAULT_BATCH: BatchSettings = {
  format: { ...DEFAULT_FORMAT },
  template: DEFAULT_TEMPLATE,
  folder: null,
  overwrite: false,
  removeDone: false,
};

export const batchRows = writable<BatchRow[]>([]);

export function sanitizeBatch(raw: unknown): BatchSettings {
  const v = (raw && typeof raw === "object" ? raw : {}) as Partial<Record<keyof BatchSettings | "prefix" | "suffix", unknown>>;
  const text = (x: unknown) => (typeof x === "string" ? x : "");
  // Settings saved before templates had a Before and After box.
  const old = `${text(v.prefix)}{name}${text(v.suffix)}`;
  return {
    format: sanitizeFormat(v.format),
    template: text(v.template).trim() !== "" ? text(v.template) : old,
    folder: typeof v.folder === "string" && v.folder ? v.folder : null,
    overwrite: v.overwrite === true,
    removeDone: v.removeDone === true,
  };
}

const lastSep = (p: string) => Math.max(p.lastIndexOf("/"), p.lastIndexOf("\\"));

export function fileName(path: string): string {
  return path.slice(lastSep(path) + 1);
}

/** Characters Windows won't take in a file name. */
const cleanPart = (s: string) => s.replace(/[<>:"/\\|?*\u0000-\u001f]/g, "_");

/** What the tokens in a name template can use. */
export interface NameContext {
  /** Position in the list, starting at 1. */
  number: number;
  /** How many files are in the list; sets how many digits {number} has. */
  count: number;
  date: Date;
}

const pad = (n: number, w: number) => String(n).padStart(w, "0");

/** Fills a name template: {name}, {number}, {date} (2026-10-15) and {format} (the output extension). */
export function expandTemplate(template: string, stem: string, kind: FormatKind, ctx: NameContext): string {
  const date = `${ctx.date.getFullYear()}-${pad(ctx.date.getMonth() + 1, 2)}-${pad(ctx.date.getDate(), 2)}`;
  const out = (template.trim() === "" ? DEFAULT_TEMPLATE : template)
    .replace(/\{name\}/gi, stem)
    .replace(/\{number\}/gi, pad(ctx.number, Math.max(2, String(ctx.count).length)))
    .replace(/\{date\}/gi, date)
    .replace(/\{format\}/gi, extensionOf(kind));
  return cleanPart(out).trim() || cleanPart(stem);
}

/** Where a source file converts to. */
export function destFor(
  source: string,
  s: { template: string; folder: string | null; kind: FormatKind },
  ctx: NameContext = { number: 1, count: 1, date: new Date() },
): string {
  const cut = lastSep(source);
  const dir = (s.folder ?? (cut >= 0 ? source.slice(0, cut) : ".")).replace(/[\\/]+$/, "");
  const name = source.slice(cut + 1);
  const dot = name.lastIndexOf(".");
  const stem = dot > 0 ? name.slice(0, dot) : name;
  const sep = dir.includes("\\") || (s.folder === null && source.includes("\\")) ? "\\" : "/";
  return `${dir}${sep}${expandTemplate(s.template, stem, s.kind, ctx)}.${extensionOf(s.kind)}`;
}

const same = (a: string, b: string) => a.replace(/\\/g, "/").toLowerCase() === b.replace(/\\/g, "/").toLowerCase();

/**
 * Problems that stop a row from converting: two rows writing the same
 * file, or a row writing over its own source without Replace on.
 * Keyed by row id.
 */
export function conflicts(rows: readonly BatchRow[], dests: ReadonlyMap<string, string>, overwrite: boolean): Map<string, string> {
  const out = new Map<string, string>();
  const seen = new Map<string, string>();
  for (const r of rows) {
    const d = dests.get(r.id);
    if (!d) continue;
    const key = d.replace(/\\/g, "/").toLowerCase();
    const first = seen.get(key);
    if (first) out.set(r.id, `Writes the same file as ${fileName(first)}. Add a prefix or suffix.`);
    else seen.set(key, r.source);
    if (!out.has(r.id) && same(d, r.source) && !overwrite) out.set(r.id, "Would replace the original. Turn on Replace existing files, or add a prefix or suffix.");
  }
  return out;
}

/** Adds files not already listed; returns how many were new. */
export function addRows(rows: readonly BatchRow[], paths: readonly string[], newId: () => string): { rows: BatchRow[]; added: number } {
  const have = new Set(rows.map((r) => r.source.replace(/\\/g, "/").toLowerCase()));
  const next = [...rows];
  for (const p of paths) {
    const k = p.replace(/\\/g, "/").toLowerCase();
    if (have.has(k)) continue;
    have.add(k);
    next.push({ id: newId(), source: p, status: "waiting", progress: 0, error: null });
  }
  return { rows: next, added: next.length - rows.length };
}
