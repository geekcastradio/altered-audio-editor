// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * Checks that a release tag matches the version in every file that carries it.
 *
 *   node scripts/check-release-version.mjs v0.5.5
 *
 * The release workflow runs this first, so a tag pushed before the version was
 * bumped stops there instead of shipping an installer with the old version inside.
 * With no argument it only checks that the files agree with each other.
 */

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
/** @param {string} p */
const read = (p) => readFileSync(join(root, p), "utf8").replace(/^\uFEFF/, "").replace(/\r\n/g, "\n");

/** @type {[string, (s: string) => string | undefined][]} */
const places = [
  ["package.json", (s) => JSON.parse(s).version],
  ["package-lock.json", (s) => JSON.parse(s).version],
  ["src-tauri/tauri.conf.json", (s) => JSON.parse(s).version],
  ["src-tauri/Cargo.toml", (s) => /\[package\][\s\S]*?\nversion = "([^"]+)"/.exec(s)?.[1]],
  ["src-tauri/Cargo.lock", (s) => /name = "altered-audio-editor"\nversion = "([^"]+)"/.exec(s)?.[1]],
];

const tag = process.argv[2];
if (tag !== undefined && !/^v\d+\.\d+\.\d+$/.test(tag)) {
  console.error(`The tag "${tag}" is not like v1.2.3.`);
  process.exit(1);
}

const found = places.map(([file, get]) => {
  try {
    return [file, get(read(file)) ?? "(not found)"];
  } catch (e) {
    return [file, `(unreadable: ${e instanceof Error ? e.message : e})`];
  }
});
const want = tag === undefined ? found[0][1] : tag.slice(1);
const wrong = found.filter(([, v]) => v !== want);
if (wrong.length > 0) {
  console.error(tag === undefined ? "The version files disagree:" : `The tag ${tag} does not match the files:`);
  for (const [file, v] of found) console.error(`  ${v === want ? "ok " : "BAD"} ${file}: ${v}`);
  console.error(tag === undefined ? "" : "Run npm run bump, commit and merge, then tag the new commit.");
  process.exit(1);
}
console.log(`All ${found.length} version files say ${want}${tag === undefined ? "" : ", matching the tag"}.`);
