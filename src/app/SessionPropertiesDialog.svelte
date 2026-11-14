<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Session name, sample rate and bit depth, plus a read-only summary.
  Enter confirms, Escape cancels.
-->
<script lang="ts">
  import Dialog from "./Dialog.svelte";
  import { formatClock } from "../panels/editor/timeline";
  import { formatRate, formatSize } from "../panels/files/format";
  import { sessionEnd, type SessionDoc } from "../stores/session";
  import {
    BIT_DEPTHS,
    currentProperties,
    mixdownBytes,
    LOUDNESS_CHOICES,
    PAN_LAWS,
    SAMPLE_RATES,
    type SessionProperties,
  } from "./session-properties";

  interface Props {
    doc: SessionDoc;
    path: string | null;
    onDone: (value: SessionProperties | null) => void;
  }

  const { doc, path, onDone }: Props = $props();

  // svelte-ignore state_referenced_locally
  const start = currentProperties(doc);
  let name = $state(start.name);
  let sampleRate = $state(start.sampleRate);
  let bitDepth = $state(start.bitDepth);
  let panLaw = $state(start.panLaw ?? "balance");
  let loudnessTarget = $state(start.loudnessTarget ?? "");

  const rates = $derived(
    (SAMPLE_RATES as readonly number[]).includes(start.sampleRate)
      ? [...SAMPLE_RATES]
      : [...SAMPLE_RATES, start.sampleRate].sort((a, b) => a - b),
  );
  const clips = $derived(doc.tracks.reduce((n, t) => n + t.clips.length, 0));
  const length = $derived(sessionEnd(doc));
  const lengthAtNewRate = $derived(Math.round((length * sampleRate) / (doc.sample_rate || sampleRate)));
  const rateChanged = $derived(sampleRate !== doc.sample_rate);

  function done() {
    if (name.trim()) onDone({ name: name.trim(), sampleRate, bitDepth, panLaw, loudnessTarget });
  }

</script>

<Dialog title="Session Properties" width={420} onEnter={done} onClose={() => onDone(null)}>
  <label class="stack">
    <span>Name</span>
    <input type="text" spellcheck="false" bind:value={name} data-autofocus />
  </label>
  <div class="cols">
    <label class="stack">
      <span>Sample rate</span>
      <select bind:value={sampleRate}>
        {#each rates as r (r)}
          <option value={r}>{formatRate(r)}</option>
        {/each}
      </select>
    </label>
    <label class="stack">
      <span>Bit depth</span>
      <select bind:value={bitDepth}>
        {#each BIT_DEPTHS as d (d.value)}
          <option value={d.value}>{d.label}</option>
        {/each}
      </select>
    </label>
  </div>
  <label class="stack">
    <span>Pan law</span>
    <select bind:value={panLaw}>
      {#each PAN_LAWS as l (l.value)}
        <option value={l.value}>{l.label}</option>
      {/each}
    </select>
  </label>
  <p class="note">{PAN_LAWS.find((l) => l.value === panLaw)?.hint}{panLaw !== start.panLaw ? " Changing it changes how every panned track sounds in the mix." : ""}</p>
  <label class="stack">
    <span>Loudness target</span>
    <select bind:value={loudnessTarget}>
      {#each LOUDNESS_CHOICES as c (c.id)}
        <option value={c.id}>{c.label}</option>
      {/each}
    </select>
  </label>
  <p class="note">What File &gt; Check Loudness measures this session against. It is saved with the session and with a template made from it.</p>
  {#if rateChanged}
    <p class="note">
      Clips keep their place in time. Files at a different rate are converted as they play, so nothing on disk
      changes.
    </p>
  {/if}
  <dl>
    <dt>Tracks</dt>
    <dd>{doc.tracks.length}</dd>
    <dt>Clips</dt>
    <dd>{clips}</dd>
    <dt>Files used</dt>
    <dd>{doc.media.length}</dd>
    <dt>Length</dt>
    <dd>{formatClock(length, doc.sample_rate)}</dd>
    <dt>Stereo mixdown</dt>
    <dd>{formatSize(mixdownBytes(lengthAtNewRate, bitDepth))}</dd>
    <dt>Saved at</dt>
    <dd class="path" title={path ?? ""}>{path ?? "Not saved yet"}</dd>
  </dl>
  {#snippet footer()}
    <button type="button" onclick={() => onDone(null)}>Cancel</button>
    <button type="button" class="primary" disabled={!name.trim()} onclick={done}>OK</button>
  {/snippet}
</Dialog>

<style>
  .cols {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
  }
  .note {
    font-size: var(--font-size-small);
  }
  dl {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    gap: 2px 12px;
    margin: 0;
    padding-top: 8px;
    border-top: 1px solid var(--border);
    font-size: var(--font-size-small);
  }
  dt {
    color: var(--text-dim);
  }
  dd {
    margin: 0;
    color: var(--value);
    font-family: var(--font-mono);
    text-align: right;
  }
  dd.path {
    overflow: hidden;
    font-family: inherit;
    text-overflow: ellipsis;
    white-space: nowrap;
    direction: rtl;
  }
</style>
