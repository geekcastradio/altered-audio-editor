// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import type { Library, LibraryItem } from "../app/core";
import { UndoStack } from "../commands";
import { folderName, itemPath, joinPath, LibraryStore, parseTags, retag, type LibraryBackend } from "./libraries";
import type { KnownLibrary } from "./library-list";

function emptyLib(name: string): Library {
  return {
    format: "altaudio.library",
    schema_version: 1,
    id: `lib-${name}`,
    name,
    scope: "show",
    created: "2026-10-13T00:00:00Z",
    items: [],
    collections: [],
  };
}

function item(id: string, file = `audio/${id}.wav`): LibraryItem {
  return {
    id,
    name: id,
    kind: "audio",
    file,
    copied: true,
    original_path: null,
    sample_rate: 48000,
    channels: 2,
    length: 48000,
    tags: [],
    notes: "",
    color: null,
    added: "2026-10-13T00:00:00Z",
  };
}

function setup(
  saved: { open: string[]; active: string | null } = { open: [], active: null },
  disk = new Map<string, Library>(),
  savedKnown: KnownLibrary[] = [],
) {
  let n = 0;
  const backend: LibraryBackend = {
    defaultRoot: async () => "C:\\Users\\host\\Documents\\Altered Audio\\Libraries",
    create: async (dir, name) => {
      const lib = emptyLib(name);
      disk.set(dir, lib);
      return lib;
    },
    open: async (dir) => {
      const lib = disk.get(dir);
      if (!lib) throw new Error("not a library");
      return structuredClone(lib);
    },
    save: async (dir, lib) => void disk.set(dir, structuredClone(lib)),
    import: async (_dir, paths) => ({ items: paths.map(() => item(`i${++n}`)), skipped: [] }),
  };
  let prefs = saved;
  let known = savedKnown;
  const store = new LibraryStore(backend, {
    get: () => prefs,
    set: (open, active) => (prefs = { open, active }),
    getKnown: () => known,
    setKnown: (list) => (known = structuredClone(list)),
  });
  return { store, disk, stack: new UndoStack(), prefs: () => prefs, known: () => known };
}

describe("LibraryStore", () => {
  it("creates a library under the default folder and remembers it", async () => {
    const { store, prefs } = setup();
    const lib = await store.create("Network: Promos?");
    expect(lib.dir).toBe("C:\\Users\\host\\Documents\\Altered Audio\\Libraries\\Network Promos");
    expect(store.current.active?.lib.name).toBe("Network: Promos?");
    expect(prefs().open).toEqual([lib.dir]);
  });

  it("imports into the active library and the shown collection, undoably", async () => {
    const { store, stack, disk } = setup();
    const { dir } = await store.create("Beds");
    const cid = await store.addCollection("Intros", stack);
    store.showCollection(cid);
    await store.importPaths(["a.wav", "b.wav"], stack);
    expect(store.current.active?.lib.items.map((i) => i.id)).toEqual(["i1", "i2"]);
    expect(store.shownItems().map((i) => i.id)).toEqual(["i1", "i2"]);
    expect(stack.current.undoLabel).toBe("Add 2 to Library");
    expect([...store.current.selected]).toEqual(["i1", "i2"]);
    await stack.undo();
    expect(disk.get(dir)?.items).toEqual([]);
    expect(store.current.selected.size).toBe(0);
    await stack.redo();
    expect(disk.get(dir)?.items).toHaveLength(2);
  });

  it("removes items from the library and every collection", async () => {
    const { store, stack } = setup();
    await store.create("X");
    await store.importPaths(["a", "b", "c"], stack);
    const cid = await store.addCollection("C", stack, ["i1", "i2"]);
    await store.removeItems(["i2"], stack);
    const lib = store.current.active!.lib;
    expect(lib.items.map((i) => i.id)).toEqual(["i1", "i3"]);
    expect(lib.collections.find((c) => c.id === cid)?.items).toEqual(["i1"]);
    await stack.undo();
    expect(store.current.active!.lib.collections[0]?.items).toEqual(["i1", "i2"]);
  });

  it("edits item metadata and collections", async () => {
    const { store, stack } = setup();
    await store.create("X");
    await store.importPaths(["a"], stack);
    await store.updateItem("i1", { name: "Cold Open", tags: ["intro"] }, stack);
    expect(stack.current.undoLabel).toBe("Rename Library Item");
    const cid = await store.addCollection("Openers", stack);
    await store.addToCollection(cid, ["i1", "i1"], stack);
    await store.renameCollection(cid, "Cold Opens", stack);
    store.showCollection(cid);
    expect(store.shownItems().map((i) => i.name)).toEqual(["Cold Open"]);
    await store.removeFromCollection(cid, ["i1"], stack);
    expect(store.shownItems()).toEqual([]);
    await store.deleteCollection(cid, stack);
    expect(store.current.collection).toBeNull();
    await stack.undo();
    expect(store.current.active!.lib.collections[0]?.name).toBe("Cold Opens");
  });

  it("sets a cover from a picture only, clears it on request, and drops it when the picture is removed", async () => {
    const { store, stack, disk } = setup();
    const o = await store.create("Show");
    await store.edit("add", (lib) => void lib.items.push(item("song"), { ...item("art", "art.png"), kind: "image" }), stack);
    await store.setLibraryCover("song", stack);
    expect(store.current.active!.lib.cover ?? null).toBeNull();
    await store.setLibraryCover("art", stack);
    expect(store.current.active!.lib.cover).toBe("art");
    expect(disk.get(o.dir)!.cover).toBe("art");
    await stack.undo();
    expect(store.current.active!.lib.cover ?? null).toBeNull();
    await stack.redo();
    await store.setLibraryCover(null, stack);
    expect(store.current.active!.lib.cover).toBeNull();
    await store.setLibraryCover("art", stack);
    await store.removeItems(["art"], stack);
    expect(store.current.active!.lib.cover).toBeNull();
  });

  it("restores open libraries and skips missing ones", async () => {
    const first = setup();
    const a = await first.store.create("A");
    const b = await first.store.create("B");
    const second = setup({ open: [a.dir, "D:\\gone", b.dir, a.dir], active: a.dir }, first.disk);
    const failed = await second.store.restore();
    expect(failed).toEqual(["D:\\gone"]);
    expect(second.store.current.libs.map((l) => l.lib.name)).toEqual(["A", "B"]);
    expect(second.store.current.active?.dir).toBe(a.dir);
    // A second restore (the window reloading) adds nothing.
    expect(await second.store.restore()).toEqual([]);
    expect(second.store.current.libs).toHaveLength(2);
  });
});

describe("known libraries", () => {
  it("keeps a closed library on the list and reopens it in one step", async () => {
    const { store, known } = setup();
    const a = await store.create("Beds");
    expect(known().map((k) => k.name)).toEqual(["Beds"]);
    store.close(a.dir);
    expect(store.current.libs).toHaveLength(0);
    expect(store.current.known.map((k) => k.dir)).toEqual([a.dir]);
    expect(store.isOpen(a.dir)).toBe(false);
    await store.openDir(a.dir);
    expect(store.isOpen(a.dir)).toBe(true);
    expect(store.current.known).toHaveLength(1);
  });

  it("tracks a rename and a color change on the list", async () => {
    const { store, stack, known } = setup();
    await store.create("Beds");
    await store.renameLibrary("Music Beds", stack);
    await store.setLibraryColor("--clip-4", stack);
    expect(known()).toEqual([{ dir: store.current.active!.dir, name: "Music Beds", color: "--clip-4" }]);
  });

  it("forgets a library without touching its folder", async () => {
    const { store, disk, known } = setup();
    const a = await store.create("Beds");
    store.forget(a.dir);
    expect(store.current.libs).toHaveLength(0);
    expect(known()).toEqual([]);
    expect(disk.has(a.dir)).toBe(true);
  });

  it("adds new entries from a list and ignores the ones it has", async () => {
    const { store, known } = setup();
    const a = await store.create("Beds");
    const added = store.addKnown([
      { dir: a.dir.toLowerCase(), name: "Beds", color: null },
      { dir: "D:\\Shows\\Promos", name: "Promos", color: "--clip-2" },
    ]);
    expect(added).toBe(1);
    expect(known().map((k) => k.name)).toEqual(["Beds", "Promos"]);
  });

  it("locates a moved library: opens the new folder and drops the old entry", async () => {
    const first = setup();
    const a = await first.store.create("Beds");
    const moved = "E:\\Moved\\Beds";
    first.disk.set(moved, first.disk.get(a.dir)!);
    first.disk.delete(a.dir);
    const second = setup({ open: [], active: null }, first.disk, [{ dir: a.dir, name: "Beds", color: null }]);
    await second.store.restore();
    await expect(second.store.openDir(a.dir)).rejects.toThrow();
    await second.store.locate(a.dir, moved);
    expect(second.known().map((k) => k.dir)).toEqual([moved]);
    expect(second.store.isOpen(moved)).toBe(true);
  });

  it("keeps a library that was open but is gone on the list under its folder name", async () => {
    const { store, known } = setup({ open: ["D:\\Gone\\Old Show"], active: null });
    expect(await store.restore()).toEqual(["D:\\Gone\\Old Show"]);
    expect(known()).toEqual([{ dir: "D:\\Gone\\Old Show", name: "Old Show", color: null }]);
  });

  it("puts libraries that were already open on the list the first time it runs", async () => {
    const first = setup();
    const a = await first.store.create("A");
    const second = setup({ open: [a.dir], active: a.dir }, first.disk, []);
    await second.store.restore();
    expect(second.known().map((k) => k.name)).toEqual(["A"]);
  });
});

describe("library paths", () => {
  it("joins and resolves item paths in either slash style", () => {
    expect(joinPath("C:\\Libs\\Beds", "audio/bed.wav")).toBe("C:\\Libs\\Beds\\audio\\bed.wav");
    expect(joinPath("/home/s/Beds/", "audio/bed.wav")).toBe("/home/s/Beds/audio/bed.wav");
    expect(itemPath("C:\\Libs", item("x", "D:\\Raw\\x.wav"))).toBe("D:\\Raw\\x.wav");
    expect(itemPath("/libs", item("x", "audio/x.wav"))).toBe("/libs/audio/x.wav");
  });

  it("makes folder-safe names", () => {
    expect(folderName("Show: Promos / Ads")).toBe("Show Promos Ads");
    expect(folderName("  ...  ")).toBe("Library");
  });
});

describe("bulk tags", () => {
  it("reads a comma separated box without blanks or repeats", () => {
    expect(parseTags(" intro, Ads ,, intro,ADS ")).toEqual(["intro", "Ads"]);
    expect(parseTags("  ")).toEqual([]);
  });

  it("adds tags once and removes them ignoring case", () => {
    expect(retag(["intro", "music"], ["Music", "bed"], [])).toEqual(["intro", "music", "bed"]);
    expect(retag(["intro", "music"], [], ["MUSIC"])).toEqual(["intro"]);
    expect(retag([], ["a"], ["a"])).toEqual([]);
  });
});
