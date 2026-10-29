<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  File > Edit File Tags: the title, artist, album, year, genre, track and
  comment stored inside a WAV, FLAC, MP3 or M4A file. Only filled-in
  fields are written, so chapters and the audio stay exactly as they are.
-->
<script lang="ts">
  import type { FileTags } from "./core";
  import { TAG_FIELDS, cleanTags } from "./tags";

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
  let first = $state<HTMLInputElement | null>(null);
  $effect(() => first?.focus());
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && onClose()} />

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="backdrop" onclick={onClose}>
  <div class="dialog" role="dialog" aria-modal="true" aria-labelledby="tags-title" tabindex="-1" onclick={(e) => e.stopPropagation()}>
    <h1 id="tags-title">File Tags</h1>
    <p class="file" title={name}>{name}</p>
    {#if unsupported}
      <p class="note">Tags can be read from this kind of file but not written to it here. WAV, FLAC, MP3 and M4A files take tags.</p>
    {/if}
    {#each TAG_FIELDS as f, i (f.key)}
      <label class="row">
        <span>{f.label}</span>
        {#if f.key === "comment"}
          <textarea rows="3" bind:value={draft[f.key]} disabled={unsupported}></textarea>
        {:else if i === 0}
          <input type="text" bind:this={first} bind:value={draft[f.key]} disabled={unsupported} />
        {:else}
          <input type="text" bind:value={draft[f.key]} disabled={unsupported} />
        {/if}
      </label>
    {/each}
    <p class="note">Only the fields you fill in are written. Clearing a field here does not remove it from the file yet.</p>
    <div class="buttons">
      <button type="button" onclick={onClose}>Cancel</button>
      <button type="button" class="primary" disabled={unsupported} onclick={() => onSave(cleanTags(draft))}>Save Tags</button>
    </div>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 5100;
    display: grid;
    place-items: center;
    background: rgba(0, 0, 0, 0.5);
  }
  .dialog {
    display: grid;
    gap: 8px;
    width: min(420px, 94vw);
    max-height: 92vh;
    overflow: auto;
    padding: 14px;
    border: 1px solid var(--border-strong);
    background: var(--surface-2);
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.6);
    font-size: 12px;
  }
  h1 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
  }
  .file {
    margin: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text-dim);
  }
  .row {
    display: grid;
    grid-template-columns: 70px minmax(0, 1fr);
    gap: 8px;
    align-items: start;
    color: var(--text-dim);
  }
  input,
  textarea,
  button {
    border: 1px solid var(--border-strong);
    border-radius: 2px;
    background: var(--surface-1);
    color: var(--text);
    font: inherit;
  }
  input,
  button {
    height: 24px;
  }
  input,
  textarea {
    padding: 0 6px;
    min-width: 0;
  }
  textarea {
    padding: 4px 6px;
    resize: vertical;
  }
  .note {
    margin: 0;
    color: var(--text-faint);
  }
  .buttons {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
  }
  .buttons button {
    min-width: 84px;
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
