// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * The User Guide: one Markdown file (`guide.md`) kept in the program, so
 * it reads offline and the same text can go on a website later. A `##`
 * line starts a section. Inside: paragraphs, `-` lists, `1.` lists, and
 * `**bold**` and `` `code` `` in the text. Nothing else is needed.
 */

export type Span = { text: string; bold?: boolean; code?: boolean };

/** `img` is a picture on a line of its own: `![what it shows](file-name)`, a file in `guide/images`. */
export type Block = { kind: "p"; spans: Span[] } | { kind: "ul"; items: Span[][] } | { kind: "ol"; items: Span[][] } | { kind: "img"; name: string; alt: string };

export interface GuideSection {
  id: string;
  title: string;
  blocks: Block[];
  /** Plain text, lower case, for searching. */
  text: string;
}

/** Splits a line into plain, bold and code runs. */
export function spans(line: string): Span[] {
  const out: Span[] = [];
  const re = /\*\*(.+?)\*\*|`(.+?)`/g;
  let last = 0;
  for (let m = re.exec(line); m; m = re.exec(line)) {
    if (m.index > last) out.push({ text: line.slice(last, m.index) });
    if (m[1] !== undefined) out.push({ text: m[1], bold: true });
    else out.push({ text: m[2]!, code: true });
    last = m.index + m[0].length;
  }
  if (last < line.length) out.push({ text: line.slice(last) });
  return out;
}

const plain = (s: Span[]) => s.map((x) => x.text).join("");

const slug = (t: string) =>
  t
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "");

export function parseGuide(md: string): GuideSection[] {
  const sections: GuideSection[] = [];
  let cur: GuideSection | null = null;
  let para: string[] = [];
  let list: { kind: "ul" | "ol"; items: Span[][] } | null = null;
  const flushPara = () => {
    if (cur && para.length > 0) cur.blocks.push({ kind: "p", spans: spans(para.join(" ")) });
    para = [];
  };
  const flushList = () => {
    if (cur && list) cur.blocks.push(list);
    list = null;
  };
  for (const raw of md.split(/\r?\n/)) {
    const line = raw.trimEnd();
    const head = /^##\s+(.+)$/.exec(line);
    if (head) {
      flushPara();
      flushList();
      cur = { id: slug(head[1]!), title: head[1]!.trim(), blocks: [], text: "" };
      sections.push(cur);
      continue;
    }
    if (!cur) continue;
    const image = /^!\[(.*?)\]\((.+?)\)$/.exec(line.trim());
    if (image) {
      flushPara();
      flushList();
      cur.blocks.push({ kind: "img", alt: image[1]!, name: image[2]! });
      continue;
    }
    const bullet = /^-\s+(.+)$/.exec(line);
    const number = /^\d+\.\s+(.+)$/.exec(line);
    if (bullet || number) {
      flushPara();
      const kind = bullet ? "ul" : "ol";
      if (list && list.kind !== kind) flushList();
      list ??= { kind, items: [] };
      list.items.push(spans((bullet ?? number)![1]!));
      continue;
    }
    if (line.trim() === "") {
      flushPara();
      flushList();
      continue;
    }
    // A wrapped line continues the paragraph or the last list item.
    if (list) {
      const last = list.items[list.items.length - 1]!;
      last.push(...spans(" " + line.trim()));
    } else para.push(line.trim());
  }
  flushPara();
  flushList();
  for (const s of sections) {
    s.text = (s.title + " " + s.blocks.map((b) => (b.kind === "p" ? plain(b.spans) : b.kind === "img" ? b.alt : b.items.map(plain).join(" "))).join(" ")).toLowerCase();
  }
  return sections;
}

/** Sections that contain every word of `query` (all of them for an empty query). */
export function searchGuide(sections: readonly GuideSection[], query: string): GuideSection[] {
  const words = query.toLowerCase().split(/\s+/).filter(Boolean);
  if (words.length === 0) return [...sections];
  return sections.filter((s) => words.every((w) => s.text.includes(w)));
}
