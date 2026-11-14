<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  File > Edit File Tags: the title, artist, album, year, genre, track and
  comment stored inside a WAV, FLAC, MP3 or M4A file. Only filled-in
  fields are written, so chapters and the audio stay exactly as they are.
-->
<script lang="ts">
  import Dialog from "./Dialog.svelte";
  import type { FileTags } from "./core";
  import { TAG_FIELDS, saveTags } from "./tags";

  interface Props {
    /** The file's name, shown in the title. */
    name: string;
    tags: FileTags;
    /** The format can't hold tags here (for example Ogg). */
    unsupported?: boolean;
    onSave: (tags: FileTags) => void;
    onClose: () => void;
  }

  const { name, tags, unsupported = false, onSave, onClose }: Props = $props();
  // svelte-ignore state_referenced_locally
  let draft = $state<Record<string, string>>(Object.fromEntries(TAG_FIELDS.map((f) => [f.key, tags[f.key] ?? ""])));
</script>

<Dialog title="File Tags" width={420} raised {onClose}>
  <p class="file" title={name}>{name}</p>
  {#if unsupported}
    <p class="note">Tags can be read from this kind of file but not written to it here. WAV, FLAC, MP3 and M4A files take tags.</p>
  {/if}
  {#each TAG_FIELDS as f (f.key)}
    <label class="row" class:top={f.key === "comment"}>
      <span>{f.label}</span>
      {#if f.key === "comment"}
        <textarea rows="3" bind:value={draft[f.key]} disabled={unsupported}></textarea>
      {:else}
        <input type="text" bind:value={draft[f.key]} disabled={unsupported} />
      {/if}
    </label>
  {/each}
  <p class="note">Fields you fill in are written; fields you empty are removed from the file.</p>
  {#snippet footer()}
    <button type="button" onclick={onClose}>Cancel</button>
    <button type="button" class="primary" disabled={unsupported} onclick={() => onSave(saveTags(draft))}>Save Tags</button>
  {/snippet}
</Dialog>

<style>
  .file {
    margin: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text-dim);
  }
  .row.top {
    align-items: start;
  }
</style>
