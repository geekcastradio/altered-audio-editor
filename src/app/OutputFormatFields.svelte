<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  The format controls shared by Export Mixdown and Batch Convert:
  format, sample rate, channels, then bit depth for WAV and FLAC or the
  bit rate for MP3. "Same as source" keeps what the audio already is.
-->
<script lang="ts">
  import { formatRate } from "../panels/files/format";
  import { depthsFor, FORMAT_KINDS, MP3_BITRATES, MP3_MAX_RATE, OUTPUT_RATES, type OutputFormat } from "./output-format";

  interface Props {
    format: OutputFormat;
    /** What "Same as source" means here, for example "Session" or "Same as each file". */
    sourceLabel: string;
  }

  let { format = $bindable(), sourceLabel }: Props = $props();

  const depths = $derived(depthsFor(format.kind));
  const lossy = $derived(FORMAT_KINDS.find((k) => k.value === format.kind)?.lossy ?? false);

  // A depth the new format can't write goes back to the source's.
  $effect(() => {
    if (format.bitDepth !== null && !depths.some((d) => d.value === format.bitDepth)) format = { ...format, bitDepth: null };
  });

  const rateLabel = (r: number) => `${formatRate(r)}${format.kind === "mp3" && r > MP3_MAX_RATE ? " (MP3 uses 48 kHz)" : ""}`;
</script>

<div class="fields">
  <label>
    <span>Format</span>
    <select bind:value={format.kind}>
      {#each FORMAT_KINDS as k (k.value)}
        <option value={k.value}>{k.label}</option>
      {/each}
    </select>
  </label>
  <label>
    <span>Sample rate</span>
    <select bind:value={format.sampleRate}>
      <option value={null}>{sourceLabel}</option>
      {#each OUTPUT_RATES as r (r)}
        <option value={r}>{rateLabel(r)}</option>
      {/each}
    </select>
  </label>
  <label>
    <span>Channels</span>
    <select bind:value={format.channels}>
      <option value={null}>{sourceLabel}</option>
      <option value={1}>Mono</option>
      <option value={2}>Stereo</option>
    </select>
  </label>
  {#if format.kind === "mp3"}
    <label>
      <span>Bit rate</span>
      <select bind:value={format.mp3Mode}>
        <option value="cbr">Constant</option>
        <option value="vbr">Variable</option>
      </select>
    </label>
    {#if format.mp3Mode === "cbr"}
      <label>
        <span>Kbps</span>
        <select bind:value={format.mp3Bitrate}>
          {#each MP3_BITRATES as b (b)}
            <option value={b}>{b} kbps{b === 128 ? " (spoken word)" : b === 192 ? " (common)" : ""}</option>
          {/each}
        </select>
      </label>
    {:else}
      <label>
        <span>Quality</span>
        <select bind:value={format.mp3Quality}>
          {#each [0, 1, 2, 3, 4, 5, 6, 7, 8, 9] as q (q)}
            <option value={q}>{q}{q === 0 ? " (best, largest)" : q === 9 ? " (smallest)" : ""}</option>
          {/each}
        </select>
      </label>
    {/if}
  {:else}
    <label>
      <span>Bit depth</span>
      <select bind:value={format.bitDepth}>
        <option value={null}>{sourceLabel}</option>
        {#each depths as d (d.value)}
          <option value={d.value}>{d.label}</option>
        {/each}
      </select>
    </label>
  {/if}
</div>
{#if lossy}
  <p class="note">MP3 is lossy: fine for publishing, but keep a WAV or FLAC master for editing.</p>
{/if}

<style>
  .fields {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(118px, 1fr));
    gap: 8px 10px;
  }

  label {
    display: grid;
    gap: 4px;
    color: var(--text-dim);
  }

  select {
    width: 100%;
    min-width: 0;
    height: 24px;
    padding: 0 6px;
    border: 1px solid var(--border-strong);
    border-radius: 2px;
    background: var(--surface-1);
    color: var(--text);
    font: inherit;
  }

  .note {
    margin: 0;
    color: var(--text-faint);
    font-size: var(--font-size-small);
  }
</style>
