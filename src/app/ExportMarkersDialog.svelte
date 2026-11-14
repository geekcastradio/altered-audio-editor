<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  File > Export > Markers and Chapters: writes the session's markers as
  a timestamp list, a CSV spreadsheet or podcast chapters JSON, with a
  preview and warnings about lists a site or app would reject.
-->
<script lang="ts">
  import Dialog from "./Dialog.svelte";
  import { CHAPTER_FORMATS, chaptersOf, render, warnings, type ChapterFormat } from "../panels/markers/chapters";
  import type { MarkerDoc } from "../stores/session";

  interface Props {
    markers: readonly MarkerDoc[];
    rate: number;
    format: ChapterFormat;
    onFormat: (f: ChapterFormat) => void;
    onSave: (content: string, format: ChapterFormat) => void;
    onClose: () => void;
  }

  const { markers, rate, format, onFormat, onSave, onClose }: Props = $props();

  const hasChapters = $derived(markers.some((m) => m.kind === "chapter"));
  // svelte-ignore state_referenced_locally
  let which = $state<"chapters" | "all">(markers.some((m) => m.kind === "chapter") ? "chapters" : "all");
  const list = $derived(chaptersOf(markers, rate, which));
  const text = $derived(render(list, format));
  const issues = $derived(warnings(list, format));
</script>

<Dialog title="Export Markers and Chapters" width={520} labelWidth={60} {onClose}>
  <label class="row">
    <span>Format</span>
    <select value={format} onchange={(e) => onFormat(e.currentTarget.value as ChapterFormat)}>
      {#each CHAPTER_FORMATS as f (f.id)}<option value={f.id}>{f.label}</option>{/each}
    </select>
  </label>
  <fieldset>
    <label><input type="radio" name="which" value="chapters" checked={which === "chapters"} disabled={!hasChapters} onchange={() => (which = "chapters")} /> Chapter markers only{hasChapters ? "" : " (none yet: set a marker's type to Chapter in the Markers panel)"}</label>
    <label><input type="radio" name="which" value="all" checked={which === "all"} onchange={() => (which = "all")} /> Every marker</label>
  </fieldset>
  <textarea class="preview" readonly rows="10" aria-label="Preview">{text}</textarea>
  {#each issues as w (w)}<p class="warn">{w}</p>{/each}
  {#snippet footer()}
    <button type="button" onclick={onClose}>Close</button>
    <button type="button" class="primary" disabled={list.length === 0} onclick={() => onSave(text, format)}>Save...</button>
  {/snippet}
</Dialog>

<style>
  .preview {
    padding: 6px;
    border-color: var(--border);
    background: var(--surface-0);
    font-family: var(--font-mono, monospace);
    font-size: 11px;
  }
</style>
