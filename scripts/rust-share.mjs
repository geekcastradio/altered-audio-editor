// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Prints how much of the source is Rust, by size, for the release notes.
// Counts code only: Rust against TypeScript, JavaScript, Svelte, CSS and HTML.
// Markdown, JSON, lock files, build output and node_modules are left out.
// Run with: node scripts/rust-share.mjs

import { readdirSync, statSync } from "node:fs";
import { extname, join } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = fileURLToPath(new URL("..", import.meta.url));
const SKIP = new Set(["node_modules", "target", "dist", ".git", "build", "gen"]);
const RUST = new Set([".rs"]);
const OTHER = new Set([".ts", ".js", ".mjs", ".svelte", ".css", ".html"]);

const totals = new Map();

/** @param {string} dir */
function walk(dir) {
  for (const name of readdirSync(dir)) {
    if (SKIP.has(name)) continue;
    const full = join(dir, name);
    const st = statSync(full);
    if (st.isDirectory()) walk(full);
    else {
      const ext = extname(name);
      if (RUST.has(ext) || OTHER.has(ext)) totals.set(ext, (totals.get(ext) ?? 0) + st.size);
    }
  }
}

walk(ROOT);

let rust = 0;
let other = 0;
for (const [ext, n] of totals) {
  if (RUST.has(ext)) rust += n;
  else other += n;
}
const all = rust + other;
/** @param {number} n */
const mb = (n) => (n / 1e6).toFixed(2);
for (const [ext, n] of [...totals].sort((a, b) => b[1] - a[1])) console.log(`${ext.padEnd(8)}${mb(n)} MB`);
console.log(`Rust ${((rust / all) * 100).toFixed(1)}% (${mb(rust)} MB of ${mb(all)} MB)`);
