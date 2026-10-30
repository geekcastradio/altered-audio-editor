<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Metadata panel: the tags inside audio files (WAV, FLAC, MP3, M4A). Add
  files, search the list, edit one file's fields, set MP3 cover art, and
  keep named tag sets (a show's artist, album and genre) to put on many
  files at once. Only filled-in fields are written, so chapters and the
  audio stay as they are. A file open in the Waveform editor is refused.
-->
<script lang="ts">
  import { coverChoose, coverClear, coverRead, coverSet, pickMediaFiles, tagsRead, tagsWrite, type FileTags } from "../../app/core";
  import { TAGGABLE, TAG_FIELDS, cleanTags } from "../../app/tags";
  import type { FileDropRouter } from "../../app/filedrop";
  import type { Preferences } from "../../stores/preferences";
  import { addPaths, baseName, filterEntries, overlay, problems, removeSet, sanitizeSets, upsertSet, type Entry, type TagKey, type TagSet } from "./metadata";

  interface Props {
    prefs: Preferences;
    /** Lets a picture dragged from the Libraries panel land on the cover art slot. */
    router?: FileDropRouter | null;
    /** The path of the file open in the Waveform editor, or null. */
    openPath?: () => string | null;
    ask: (title: string, label: string, initial: string) => Promise<string | null>;
    onNotice?: (m: string) => void;
    onError?: (e: unknown) => void;
  }

  const { prefs, router = null, openPath = () => null, ask, onNotice = () => {}, onError = console.error }: Props = $props();

  const KEY_SETS = "metadata.sets";
  let entries = $state<Entry[]>([]);
  let query = $state("");
  let selected = $state<string | null>(null);
  let draft = $state<Record<string, string>>({});
  let cover = $state<string | null>(null);
  let sets = $state<TagSet[]>([]);
  let setName = $state("");
  let busy = $state(false);

  $effect(() => {
    void prefs.load().then(() => {
      sets = sanitizeSets(prefs.raw(KEY_SETS));
    });
  });
  const saveSets = (next: TagSet[]) => {
    sets = next;
    prefs.set(KEY_SETS, next);
  };

  const shown = $derived(filterEntries(entries, query));
  const current = $derived(entries.find((e) => e.path === selected) ?? null);
  const unsupported = $derived(current ? !TAGGABLE.test(current.path) : false);
  const isMp3 = $derived(current ? /\.mp3$/i.test(current.path) : false);
  const issues = $derived(problems(draft as Partial<Record<TagKey, string>>));
  const chosenSet = $derived(sets.find((s) => s.name === setName) ?? null);

  const guarded = (path: string): boolean => {
    const open = openPath();
    if (open && open.replace(/\\/g, "/").toLowerCase() === path.replace(/\\/g, "/").toLowerCase()) {
      onNotice("That file is open in the Waveform editor. Save and close it first, then edit its tags.");
      return true;
    }
    return false;
  };

  async function add() {
    try {
      const paths = await pickMediaFiles();
      if (!paths.length) return;
      const fresh = addPaths(entries.map((e) => e.path), paths).slice(entries.length);
      const read = await Promise.all(fresh.map(async (path) => ({ path, tags: await tagsRead(path).catch(() => ({}) as FileTags) })));
      entries = [...entries, ...read];
      if (!selected && read[0]) await select(read[0].path);
    } catch (e) {
      onError(e);
    }
  }

  async function select(path: string) {
    selected = path;
    const e = entries.find((x) => x.path === path);
    draft = Object.fromEntries(TAG_FIELDS.map((f) => [f.key, e?.tags[f.key] ?? ""]));
    cover = null;
    if (/\.mp3$/i.test(path)) cover = await coverRead(path).catch(() => null);
  }

  function clear() {
    entries = [];
    selected = null;
    draft = {};
    cover = null;
  }

  async function reload(path: string) {
    const tags = await tagsRead(path).catch(() => ({}) as FileTags);
    entries = entries.map((e) => (e.path === path ? { path, tags } : e));
  }

  async function save() {
    if (!current || issues.length || guarded(current.path)) return;
    busy = true;
    try {
      const ok = await tagsWrite(current.path, cleanTags(draft));
      if (!ok) onNotice("Nothing was written: no field is filled in, or this format can't take tags here.");
      else onNotice(`Tags saved to ${baseName(current.path)}.`);
      await reload(current.path);
    } catch (e) {
      onError(e);
    } finally {
      busy = false;
    }
  }

  let coverBox = $state<HTMLElement | null>(null);
  let coverOver = $state(false);
  $effect(() => {
    if (!router || !coverBox) return;
    return router.register({
      element: coverBox,
      onDrop: (paths) => void dropCover(paths),
      onHover: (over) => (coverOver = over),
    });
  });

  async function dropCover(paths: string[]) {
    const pic = paths.find((p) => /\.(jpe?g|png)$/i.test(p));
    if (!current || guarded(current.path)) return;
    if (!pic) {
      onNotice("Drop a JPEG or PNG picture to use it as cover art.");
      return;
    }
    try {
      cover = await coverSet(current.path, pic);
      onNotice("Cover art set.");
    } catch (e) {
      onError(e);
    }
  }

  async function chooseCover() {
    if (!current || guarded(current.path)) return;
    try {
      const url = await coverChoose(current.path);
      if (url) {
        cover = url;
        onNotice("Cover art set.");
      }
    } catch (e) {
      onError(e);
    }
  }

  async function removeCover() {
    if (!current || guarded(current.path)) return;
    try {
      await coverClear(current.path);
      cover = null;
      onNotice("Cover art removed.");
    } catch (e) {
      onError(e);
    }
  }

  async function saveSet() {
    const name = await ask("Save Tag Set", "Name for this set", setName || "");
    if (name === null || !name.trim()) return;
    const next = upsertSet(sets, name, draft as Partial<Record<TagKey, string>>);
    if (next === sets) return onNotice("Fill in at least one field to save a set.");
    saveSets(next);
    setName = name.trim();
    onNotice(`Saved the set "${name.trim()}".`);
  }

  function fillFromSet() {
    if (!chosenSet || !current) return;
    const merged = overlay(cleanTags(draft), chosenSet);
    draft = Object.fromEntries(TAG_FIELDS.map((f) => [f.key, merged[f.key] ?? ""]));
  }

  async function applyToAll() {
    const set = chosenSet;
    if (!set) return;
    busy = true;
    let done = 0;
    const skipped: string[] = [];
    try {
      for (const e of entries) {
        if (!TAGGABLE.test(e.path) || guarded(e.path)) {
          skipped.push(baseName(e.path));
          continue;
        }
        if (await tagsWrite(e.path, { ...set.tags })) done += 1;
        else skipped.push(baseName(e.path));
        await reload(e.path);
      }
      if (selected) await select(selected);
      onNotice(`Set "${set.name}" written to ${done} ${done === 1 ? "file" : "files"}${skipped.length ? `; skipped ${skipped.length}` : ""}.`);
    } catch (e) {
      onError(e);
    } finally {
      busy = false;
    }
  }
</script>

<section class="meta" aria-label="Metadata">
  <div class="bar">
    <button type="button" onclick={() => void add()} disabled={busy}>Add Files...</button>
    <button type="button" onclick={clear} disabled={!entries.length || busy}>Clear</button>
    <input type="search" placeholder="Search files and tags" bind:value={query} aria-label="Search files and tags" />
  </div>

  {#if !entries.length}
    <p class="empty">Add audio files to see and edit their tags. WAV, FLAC, MP3 and M4A files take tags; MP3 files can also hold cover art.</p>
  {:else}
    <div class="body">
      <ul class="files" aria-label="Files">
        {#each shown as e (e.path)}
          <li>
            <button type="button" class:on={e.path === selected} onclick={() => void select(e.path)} title={e.path}>
              <span class="name">{baseName(e.path)}</span>
              <span class="sub">{e.tags.title ?? ""}{e.tags.title && e.tags.artist ? " · " : ""}{e.tags.artist ?? ""}</span>
            </button>
          </li>
        {:else}
          <li class="none">No file matches.</li>
        {/each}
      </ul>

      {#if current}
        <div class="edit">
          {#if unsupported}
            <p class="note">This kind of file can't take tags here.</p>
          {/if}
          {#each TAG_FIELDS as f (f.key)}
            <label class="row">
              <span>{f.label}</span>
              {#if f.key === "comment"}
                <textarea rows="2" bind:value={draft[f.key]} disabled={unsupported || busy}></textarea>
              {:else}
                <input type="text" bind:value={draft[f.key]} disabled={unsupported || busy} />
              {/if}
            </label>
          {/each}
          {#each issues as m (m)}
            <p class="issue" role="alert">{m}</p>
          {/each}
          <p class="note">Only filled-in fields are written. Clearing a field does not remove it from the file yet.</p>

          {#if isMp3}
            <div class="cover" class:over={coverOver} bind:this={coverBox}>
              {#if cover}
                <img src={cover} alt="Cover art" />
              {:else}
                <div class="blank">No cover</div>
              {/if}
              <div class="coverbtns">
                <button type="button" onclick={() => void chooseCover()} disabled={busy}>Choose Cover...</button>
                <button type="button" onclick={() => void removeCover()} disabled={!cover || busy}>Remove</button>
              </div>
            </div>
          {/if}

          <div class="actions">
            <button type="button" class="primary" onclick={() => void save()} disabled={unsupported || busy || issues.length > 0}>Save Tags</button>
          </div>
        </div>
      {/if}
    </div>

    <div class="sets">
      <span class="label">Tag sets</span>
      <select bind:value={setName} aria-label="Saved tag sets">
        <option value="">(none)</option>
        {#each sets as s (s.name)}
          <option value={s.name}>{s.name}</option>
        {/each}
      </select>
      <button type="button" onclick={fillFromSet} disabled={!chosenSet || !current || unsupported || busy} title="Fill this file's fields from the set. Nothing is written until you save.">Fill Fields</button>
      <button type="button" onclick={() => void applyToAll()} disabled={!chosenSet || busy} title="Write the set's fields into every file in the list.">Apply to All</button>
      <button type="button" onclick={() => void saveSet()} disabled={!current || busy}>Save as Set...</button>
      <button type="button" onclick={() => saveSets(removeSet(sets, setName))} disabled={!chosenSet || busy}>Delete Set</button>
    </div>
  {/if}
</section>

<style>
  .meta {
    display: grid;
    grid-template-rows: auto minmax(0, 1fr) auto;
    height: 100%;
    background: var(--surface-1);
    font-size: var(--font-size-small);
  }
  .bar,
  .sets {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    align-items: center;
    padding: 4px 6px;
    border-bottom: 1px solid var(--border);
  }
  .sets {
    border-top: 1px solid var(--border);
    border-bottom: none;
  }
  .label {
    color: var(--text-dim);
  }
  button,
  input,
  select,
  textarea {
    border: 1px solid var(--border-strong);
    border-radius: 2px;
    background: var(--surface-3);
    color: var(--text);
    font: inherit;
  }
  button,
  input,
  select {
    height: 20px;
  }
  button {
    padding: 0 8px;
  }
  button:disabled {
    color: var(--text-faint);
  }
  input[type="search"] {
    flex: 1 1 120px;
    min-width: 0;
    padding: 0 6px;
  }
  .empty {
    margin: 0;
    padding: 16px;
    color: var(--text-faint);
    text-align: center;
  }
  .body {
    display: grid;
    grid-template-columns: minmax(120px, 38%) minmax(0, 1fr);
    min-height: 0;
  }
  .files {
    margin: 0;
    padding: 0;
    list-style: none;
    overflow: auto;
    border-right: 1px solid var(--border);
  }
  .files button {
    display: grid;
    width: 100%;
    height: auto;
    padding: 3px 6px;
    border: none;
    border-radius: 0;
    background: transparent;
    text-align: left;
  }
  .files button.on {
    background: var(--accent);
    color: var(--on-accent, #fff);
  }
  .name,
  .sub {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .sub {
    color: var(--text-faint);
  }
  .files button.on .sub {
    color: inherit;
    opacity: 0.8;
  }
  .none {
    padding: 8px;
    color: var(--text-faint);
  }
  .edit {
    display: grid;
    align-content: start;
    gap: 4px;
    padding: 6px;
    overflow: auto;
  }
  .row {
    display: grid;
    grid-template-columns: 58px minmax(0, 1fr);
    gap: 6px;
    align-items: start;
    color: var(--text-dim);
  }
  .row input {
    padding: 0 6px;
    min-width: 0;
  }
  textarea {
    padding: 3px 6px;
    resize: vertical;
  }
  .note {
    margin: 0;
    color: var(--text-faint);
  }
  .issue {
    margin: 0;
    color: var(--danger, #e5534b);
  }
  .cover {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  .cover img,
  .blank {
    width: 72px;
    height: 72px;
    object-fit: cover;
    border: 1px solid var(--border-strong);
    background: var(--surface-2);
  }
  .blank {
    display: grid;
    place-items: center;
    color: var(--text-faint);
  }
  .cover.over {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .coverbtns {
    display: grid;
    gap: 4px;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
  }
  .primary {
    border-color: var(--accent);
    background: var(--accent);
    color: var(--on-accent, #fff);
  }
  .primary:disabled {
    opacity: 0.5;
  }
</style>
