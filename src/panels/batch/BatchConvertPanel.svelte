<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Batch Convert: a list of files and one set of output settings, turned
  into WAV, FLAC or MP3 one after another. Add files with the buttons,
  from the Files panel's selection, or by dropping them on the panel.
  Rows show each file's progress; a file that fails says why and the
  rest carry on. Settings are remembered between runs.
-->
<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { onDestroy } from "svelte";
  import { batchCancel, batchConvert, expandMediaPaths, exportPickFolder, pickMediaFiles, pickMediaFolder, unzipMedia, type BatchProgress } from "../../app/core";
  import type { FileDropRouter } from "../../app/filedrop";
  import { extensionOf } from "../../app/output-format";
  import OutputFormatFields from "../../app/OutputFormatFields.svelte";
  import type { MediaPool } from "../../stores/media-pool";
  import type { Preferences } from "../../stores/preferences";
  import { addRows, batchRows, conflicts, destFor, expandTemplate, fileName, sanitizeBatch, type BatchRow, type BatchSettings } from "./batch";

  const TOKENS = [
    { token: "{name}", hint: "The original file name, without its extension." },
    { token: "{number}", hint: "The file's place in this list: 01, 02, 03 and so on." },
    { token: "{date}", hint: "Today's date, like 2026-10-15." },
    { token: "{format}", hint: "The new format, like mp3." },
  ] as const;

  interface Props {
    pool: MediaPool;
    drop: FileDropRouter;
    prefs: Preferences;
    onNotice: (text: string) => void;
    onError: (e: unknown) => void;
  }

  const { pool, drop, prefs, onNotice, onError }: Props = $props();

  const KEY = "batch.settings";
  let s = $state<BatchSettings>(sanitizeBatch(null));
  let loaded = $state(false);
  // Settings save once the saved ones are in, so defaults never overwrite them.
  // svelte-ignore state_referenced_locally
  void prefs.load().then(() => {
    s = sanitizeBatch(prefs.raw(KEY));
    loaded = true;
  });
  $effect(() => {
    const snap = $state.snapshot(s);
    if (loaded) prefs.set(KEY, snap);
  });

  let rows = $state<BatchRow[]>([]);
  const unsub = batchRows.subscribe((v) => (rows = v));
  onDestroy(unsub);
  const setRows = (next: BatchRow[]) => batchRows.set(next);

  let selected = $state<Set<string>>(new Set());
  let running = $state(false);
  let root = $state<HTMLElement | null>(null);
  let dropping = $state(false);
  let poolCount = $state(0);
  $effect(() =>
    pool.subscribe((p) => {
      poolCount = p.selected.size;
    }),
  );

  $effect(() => {
    if (!root) return;
    return drop.register({ element: root, onDrop: (paths) => void add(paths), onHover: (over) => (dropping = over) });
  });

  const today = new Date();
  const dests = $derived(
    new Map(rows.map((r, i) => [r.id, destFor(r.source, { template: s.template, folder: s.folder, kind: s.format.kind }, { number: i + 1, count: rows.length, date: today })])),
  );
  const problems = $derived(conflicts(rows, dests, s.overwrite));
  const waiting = $derived(rows.filter((r) => r.status !== "done" && !problems.has(r.id)));
  const audioSummary = $derived.by(() => {
    const f = s.format;
    const rate = f.sampleRate ? `${f.sampleRate} Hz` : "each file's rate";
    const ch = f.channels === 1 ? "mono" : f.channels === 2 ? "stereo" : "each file's channels";
    const depth = f.bitDepth === 32 ? "32-bit float" : f.bitDepth ? `${f.bitDepth}-bit` : "each file's depth";
    const q = f.kind === "mp3" ? (f.mp3Mode === "cbr" ? `MP3 ${f.mp3Bitrate} kbps constant` : `MP3 variable, quality ${f.mp3Quality}`) : `${f.kind.toUpperCase()} ${depth}`;
    return `${q}, ${rate}, ${ch}`;
  });
  const sample = $derived(
    rows[0]
      ? fileName(dests.get(rows[0].id) ?? "")
      : `${expandTemplate(s.template, "Episode 12", s.format.kind, { number: 1, count: 1, date: today })}.${extensionOf(s.format.kind)}`,
  );
  let templateInput = $state<HTMLInputElement | null>(null);

  /** Puts a token in the template at the cursor, or at the end. */
  function insertToken(token: string) {
    const el = templateInput;
    const at = el?.selectionStart ?? s.template.length;
    const to = el?.selectionEnd ?? at;
    s.template = s.template.slice(0, at) + token + s.template.slice(to);
    requestAnimationFrame(() => {
      el?.focus();
      el?.setSelectionRange(at + token.length, at + token.length);
    });
  }

  async function add(paths: string[]) {
    if (paths.length === 0) return;
    try {
      const plain: string[] = [];
      for (const p of paths) {
        if (/\.zip$/i.test(p)) plain.push(...(await unzipMedia(p)));
        else plain.push(p);
      }
      const files = await expandMediaPaths(plain);
      const r = addRows(rows, files, () => crypto.randomUUID());
      setRows(r.rows);
      if (r.added < files.length) onNotice(`${files.length - r.added} already in the list.`);
    } catch (e) {
      onError(e);
    }
  }

  const addFiles = async () => add(await pickMediaFiles());
  async function addFolder() {
    const f = await pickMediaFolder();
    if (f) await add([f]);
  }
  const addFromFiles = () => add(pool.selectedItems().map((it) => it.path));

  function remove(ids: Set<string>) {
    setRows(rows.filter((r) => !ids.has(r.id)));
    selected = new Set();
  }

  function pick(e: MouseEvent, id: string) {
    const next = new Set(e.ctrlKey || e.metaKey ? selected : []);
    if (next.has(id) && (e.ctrlKey || e.metaKey)) next.delete(id);
    else next.add(id);
    selected = next;
  }

  async function chooseFolder() {
    const f = await exportPickFolder(s.folder, "Convert Into");
    if (f) s.folder = f;
  }

  function patch(id: string, p: Partial<BatchRow>) {
    setRows(rows.map((r) => (r.id === id ? { ...r, ...p } : r)));
  }

  async function run() {
    if (running || waiting.length === 0) return;
    running = true;
    const jobs = waiting.map((r) => ({ id: r.id, source: r.source, dest: dests.get(r.id)! }));
    setRows(rows.map((r) => (jobs.some((j) => j.id === r.id) ? { ...r, status: "waiting", progress: 0, error: null } : r)));
    const off = await listen<BatchProgress>("batch-progress", (e) => {
      const p = e.payload;
      patch(p.id, { status: p.status, progress: p.progress, error: p.error });
    });
    try {
      const r = await batchConvert(jobs, $state.snapshot(s.format), s.overwrite);
      if (s.removeDone) setRows(rows.filter((x) => x.status !== "done"));
      const parts = [`Converted ${r.done} ${r.done === 1 ? "file" : "files"}`];
      if (r.failed) parts.push(`${r.failed} failed`);
      if (r.stopped) parts.push("stopped early");
      onNotice(`${parts.join(", ")}.`);
    } catch (e) {
      onError(e);
    } finally {
      off();
      running = false;
    }
  }

  function statusText(r: BatchRow): string {
    const problem = problems.get(r.id);
    if (problem && r.status !== "done") return problem;
    switch (r.status) {
      case "running":
        return `${Math.round(r.progress * 100)}%`;
      case "done":
        return "Done";
      case "failed":
        return r.error ?? "Failed";
      case "stopped":
        return "Stopped";
      default:
        return running ? "Waiting" : "Ready";
    }
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<section class="panel" class:dropping aria-label="Batch Convert" bind:this={root}>
  <div class="toolbar">
    <button type="button" class="tool" title="Add Files" aria-label="Add Files" disabled={running} onclick={addFiles}>
      <svg viewBox="0 0 16 16" aria-hidden="true"><path d="M3 1.5h6l4 4v9H3z" fill="none" stroke="currentColor" /><path d="M8 7v5M5.5 9.5h5" stroke="currentColor" stroke-width="1.4" /></svg>
    </button>
    <button type="button" class="tool" title="Add Folder" aria-label="Add Folder" disabled={running} onclick={addFolder}>
      <svg viewBox="0 0 16 16" aria-hidden="true"><path d="M1.5 3.5h5l1.5 1.5h6.5v8h-13z" fill="none" stroke="currentColor" /><path d="M8 7v4.5M5.75 9.25h4.5" stroke="currentColor" stroke-width="1.4" /></svg>
    </button>
    <button
      type="button"
      class="tool"
      title={poolCount ? "Add the files selected in the Files panel" : "Select files in the Files panel first"}
      aria-label="Add from Files panel"
      disabled={running || poolCount === 0}
      onclick={addFromFiles}
    >
      <svg viewBox="0 0 16 16" aria-hidden="true"><path d="M2 3.5h12M2 7h12M2 10.5h7" stroke="currentColor" /><path d="M12.5 9v5M10 11.5h5" stroke="currentColor" stroke-width="1.4" /></svg>
    </button>
    <span class="gap"></span>
    <button type="button" class="tool" title="Remove from list" aria-label="Remove from list" disabled={running || selected.size === 0} onclick={() => remove(selected)}>
      <svg viewBox="0 0 16 16" aria-hidden="true"><path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.4" /></svg>
    </button>
    <button type="button" class="tool" title="Clear finished" aria-label="Clear finished" disabled={running || !rows.some((r) => r.status === "done")} onclick={() => remove(new Set(rows.filter((r) => r.status === "done").map((r) => r.id)))}>
      <svg viewBox="0 0 16 16" aria-hidden="true"><path d="M3 8.5l3 3 7-7" fill="none" stroke="currentColor" stroke-width="1.4" /></svg>
    </button>
  </div>

  <div class="list" role="listbox" aria-label="Files to convert" aria-multiselectable="true">
    {#if rows.length === 0}
      <p class="empty">Drop audio files, folders or zips here, or use the buttons above. A call recorder's per-speaker files convert in one go.</p>
    {:else}
      <div class="head" aria-hidden="true"><span>File</span><span>Becomes</span><span>Status</span></div>
      {#each rows as r (r.id)}
        {@const bad = problems.has(r.id) && r.status !== "done"}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <div
          class="row"
          class:sel={selected.has(r.id)}
          class:bad={bad || r.status === "failed"}
          class:done={r.status === "done"}
          role="option"
          aria-selected={selected.has(r.id)}
          tabindex="-1"
          title={r.source}
          onclick={(e) => pick(e, r.id)}
        >
          <span class="name">{fileName(r.source)}</span>
          <span class="dest" title={dests.get(r.id)}>{fileName(dests.get(r.id) ?? "")}</span>
          <span class="status" title={statusText(r)}>
            {#if r.status === "running"}<span class="bar"><span style:width="{r.progress * 100}%"></span></span>{/if}
            {statusText(r)}
          </span>
        </div>
      {/each}
    {/if}
  </div>

  <div class="settings">
    <h2>Name</h2>
    <label class="template">
      <span>Template</span>
      <input type="text" spellcheck="false" placeholder={"{name}"} bind:value={s.template} bind:this={templateInput} disabled={running} />
    </label>
    <div class="tokens" role="group" aria-label="Insert into the template">
      {#each TOKENS as t (t.token)}
        <button type="button" title={t.hint} disabled={running} onclick={() => insertToken(t.token)}>{t.token}</button>
      {/each}
    </div>
    <p class="hint">Example: <span class="mono">{sample}</span></p>

    <h2>Folder</h2>
    <label class="check"><input type="radio" name="dest" checked={s.folder === null} disabled={running} onchange={() => (s.folder = null)} /> Next to each original</label>
    <div class="folder">
      <label class="check"><input type="radio" name="dest" checked={s.folder !== null} disabled={running} onchange={chooseFolder} /> Into</label>
      <input type="text" readonly value={s.folder ?? ""} placeholder="Choose a folder" title={s.folder ?? ""} />
      <button type="button" disabled={running} onclick={chooseFolder}>Browse…</button>
    </div>
    <label class="check"><input type="checkbox" bind:checked={s.overwrite} disabled={running} /> Replace existing files</label>

    <h2>Audio</h2>
    <OutputFormatFields bind:format={s.format} sourceLabel="Same as each file" />
    <p class="hint">Writes: {audioSummary}</p>

    <label class="check"><input type="checkbox" bind:checked={s.removeDone} disabled={running} /> Remove finished files from the list</label>
    <label class="check off" title="Arrives with metadata export."><input type="checkbox" disabled /> Keep markers and metadata (arrives with metadata export)</label>
  </div>

  <footer>
    <span class="count">{rows.length} {rows.length === 1 ? "file" : "files"}{problems.size ? `, ${problems.size} need attention` : ""}</span>
    {#if running}
      <button type="button" onclick={() => void batchCancel()}>Stop</button>
    {:else}
      <button type="button" class="primary" disabled={waiting.length === 0} onclick={() => void run()}>Convert {waiting.length || ""}</button>
    {/if}
  </footer>
</section>

<style>
  .panel {
    display: grid;
    grid-template-rows: auto minmax(70px, 2fr) minmax(120px, 3fr) auto;
    height: 100%;
    min-height: 0;
    overflow: hidden;
    background: var(--surface-1);
  }

  .panel.dropping {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }

  .toolbar {
    display: flex;
    gap: 2px;
    align-items: center;
    padding: 3px 4px;
    border-bottom: 1px solid var(--border);
  }

  .gap {
    flex: 1;
  }

  .tool {
    display: grid;
    place-items: center;
    width: 22px;
    height: 20px;
    padding: 0;
    border: 1px solid transparent;
    border-radius: 2px;
    background: none;
    color: var(--text-dim);
  }

  .tool svg {
    width: 14px;
    height: 14px;
  }

  .tool:hover:not(:disabled) {
    border-color: var(--border-strong);
    color: var(--text);
  }

  .tool:disabled {
    color: var(--text-faint);
    opacity: 0.6;
  }

  .list {
    min-height: 0;
    overflow: auto;
    background: var(--surface-0);
  }

  .empty {
    margin: 0;
    padding: 14px 12px;
    color: var(--text-faint);
    line-height: 1.5;
  }

  .head,
  .row {
    display: grid;
    grid-template-columns: minmax(0, 1.2fr) minmax(0, 1fr) minmax(64px, 0.9fr);
    gap: 8px;
    align-items: center;
    padding: 0 8px;
  }

  .head {
    position: sticky;
    top: 0;
    height: 20px;
    border-bottom: 1px solid var(--border);
    background: var(--surface-2);
    color: var(--text-faint);
    font-size: var(--font-size-small);
  }

  .row {
    height: 22px;
    border-bottom: 1px solid color-mix(in srgb, var(--border) 50%, transparent);
    cursor: default;
  }

  .row.sel {
    background: color-mix(in srgb, var(--accent) 22%, transparent);
  }

  .row span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .dest {
    color: var(--text-dim);
  }

  .status {
    display: flex;
    gap: 6px;
    align-items: center;
    color: var(--text-dim);
    font-size: var(--font-size-small);
  }

  .row.done .status {
    color: var(--ok, #5fbf8f);
  }

  .row.bad .status {
    color: var(--danger);
  }

  .bar {
    flex: 0 0 36px;
    height: 5px;
    border-radius: 2px;
    background: var(--surface-3);
  }

  .bar span {
    display: block;
    height: 100%;
    border-radius: 2px;
    background: var(--accent);
  }

  .settings {
    display: grid;
    align-content: start;
    gap: 6px;
    min-height: 0;
    overflow: auto;
    padding: 8px 10px 10px;
    border-top: 1px solid var(--border);
  }

  h2 {
    margin: 6px 0 0;
    color: var(--text-faint);
    font-size: var(--font-size-small);
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .template {
    display: grid;
    gap: 3px;
    color: var(--text-dim);
  }

  .tokens {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }

  .tokens button {
    height: 20px;
    padding: 0 7px;
    border: 1px solid var(--border-strong);
    border-radius: 2px;
    background: var(--surface-3);
    color: var(--text-dim);
    font: var(--font-mono, inherit);
    font-size: var(--font-size-small);
  }

  .tokens button:disabled {
    opacity: 0.5;
  }

  input[type="text"] {
    min-width: 0;
    height: 22px;
    padding: 0 6px;
    border: 1px solid var(--border-strong);
    border-radius: 2px;
    background: var(--surface-0);
    color: var(--text);
    font: inherit;
  }

  input[readonly] {
    color: var(--text-dim);
  }

  .folder {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    gap: 6px;
    align-items: center;
  }

  .check {
    display: flex;
    gap: 6px;
    align-items: center;
  }

  .off {
    color: var(--text-faint);
  }

  .hint {
    margin: 0;
    color: var(--text-faint);
    font-size: var(--font-size-small);
  }

  .mono {
    color: var(--text-dim);
    font-family: var(--font-mono);
  }

  footer {
    display: flex;
    gap: 8px;
    align-items: center;
    justify-content: space-between;
    padding: 6px 10px;
    border-top: 1px solid var(--border);
    background: var(--surface-2);
  }

  .count {
    color: var(--text-dim);
    font-size: var(--font-size-small);
  }

  button:not(.tool) {
    height: 24px;
    padding: 0 12px;
    border: 1px solid var(--border-strong);
    border-radius: 2px;
    background: var(--surface-3);
    color: var(--text);
    font: inherit;
  }

  button.primary {
    border-color: var(--accent);
    background: var(--accent);
    color: var(--accent-text, #fff);
  }

  button:disabled {
    opacity: 0.5;
  }
</style>
