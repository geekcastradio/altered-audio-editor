<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  File > Publish Package: the episode's title, description and numbers,
  where to put the folder, the loudness target and the audio format.
  Makes one folder holding the audio (chapters inside, checked after
  saving) and the files to copy into a podcast host. Nothing is sent.
-->
<script lang="ts" module>
  import type { OutputFormat } from "./output-format";
  import type { EpisodeInfo } from "./publish";

  export interface PublishSettings {
    episode: EpisodeInfo;
    folder: string;
    format: OutputFormat;
    /** A loudness target id, or "" to leave the level as mixed. */
    normalize: string;
  }
</script>

<script lang="ts">
  import Dialog from "./Dialog.svelte";
  import OutputFormatFields from "./OutputFormatFields.svelte";
  import { describeFormat } from "./output-format";
  import { targetsFor, type LoudnessReport } from "./loudness";
  import LoudnessScan from "./LoudnessScan.svelte";
  import { DESCRIPTION_MAX, TITLE_MAX, episodeProblems, slugOf } from "./publish";

  interface Props {
    initial: PublishSettings;
    source: { sampleRate: number; channels: number; bitDepth: number };
    chapterCount: number;
    adCount: number;
    onBrowse: (from: string) => Promise<string | null>;
    onScan: () => Promise<LoudnessReport | null>;
    onDone: (value: PublishSettings | null) => void;
  }

  const { initial, source, chapterCount, adCount, onBrowse, onScan, onDone }: Props = $props();

  // svelte-ignore state_referenced_locally
  let s = $state<PublishSettings>({ ...initial, episode: { ...initial.episode }, format: { ...initial.format } });
  const targets = targetsFor();
  const problems = $derived(episodeProblems(s.episode));
  const ok = $derived(problems.length === 0 && s.folder !== "");
  const where = $derived(`${s.folder.replace(/[\\/]+$/, "")}${s.folder.includes("\\") ? "\\" : "/"}${slugOf(s.episode.title)}`);
  const summary = $derived(describeFormat(s.format, source));

  /** An empty box means "not given"; anything else is read as a number. */
  function num(text: string): number | null {
    const t = text.trim();
    return t === "" ? null : Number(t);
  }

  async function browse() {
    const f = await onBrowse(s.folder);
    if (f) s.folder = f;
  }

  function done() {
    if (ok) onDone({ ...s, episode: { ...s.episode, title: s.episode.title.trim() } });
  }

</script>

<Dialog title="Publish Package" width={560} onEnter={done} onClose={() => onDone(null)}>
    <section>
      <h2>Episode</h2>
      <label class="line">
        <span>Title</span>
        <input type="text" spellcheck="true" maxlength={TITLE_MAX + 50} bind:value={s.episode.title} data-autofocus />
      </label>
      <label class="top">
        <span>Description</span>
        <textarea rows="4" spellcheck="true" bind:value={s.episode.description}></textarea>
        <span class="count" class:over={s.episode.description.length > DESCRIPTION_MAX}>{s.episode.description.length} of {DESCRIPTION_MAX}</span>
      </label>
      <div class="nums">
        <label>Season <input type="text" inputmode="numeric" size="3" value={s.episode.season ?? ""} oninput={(e) => (s.episode.season = num(e.currentTarget.value))} /></label>
        <label>Episode <input type="text" inputmode="numeric" size="3" value={s.episode.episode ?? ""} oninput={(e) => (s.episode.episode = num(e.currentTarget.value))} /></label>
        <label>
          Type
          <select bind:value={s.episode.type}>
            <option value="full">Full episode</option>
            <option value="trailer">Trailer</option>
            <option value="bonus">Bonus</option>
          </select>
        </label>
        <label class="check"><input type="checkbox" bind:checked={s.episode.explicit} /> Explicit</label>
      </div>
      {#each problems as p (p)}<p class="problem">{p}</p>{/each}
    </section>
  
    <section>
      <h2>Save to</h2>
      <div class="line">
        <span>Folder</span>
        <span class="folder">
          <input type="text" readonly value={s.folder} title={s.folder} placeholder="Choose a folder" />
          <button type="button" onclick={browse}>Browse…</button>
        </span>
      </div>
      {#if s.folder}<p class="summary">Everything goes into <strong>{where}</strong></p>{/if}
    </section>
  
    <section>
      <h2>Audio</h2>
      <OutputFormatFields bind:format={s.format} sourceLabel="Session" />
      <p class="summary"><strong>{summary}</strong></p>
      <label class="line" title="Measures the mix, then raises or lowers the whole file by one fixed amount to reach the target. Peaks stay under the target's limit.">
        <span>Loudness</span>
        <select bind:value={s.normalize}>
          <option value="">Leave as mixed</option>
          {#each targets as t (t.id)}<option value={t.id}>Bring to {t.name}</option>{/each}
        </select>
      </label>
      <LoudnessScan target={s.normalize} mono={s.format.channels === 1 || (s.format.channels === null && source.channels === 1)} stamp={`${s.format.channels}`} scan={onScan} />
      <p class="summary">
        {chapterCount === 0 ? "No chapter markers." : `${chapterCount} ${chapterCount === 1 ? "chapter goes" : "chapters go"} inside the file and into chapters.json.`}
        {adCount === 0 ? "" : `${adCount} ad ${adCount === 1 ? "position is" : "positions are"} listed in episode.json.`}
        The finished file is read back and checked.
      </p>
    </section>
  {#snippet footer()}
    <button type="button" onclick={() => onDone(null)}>Cancel</button>
    <button type="button" class="primary" disabled={!ok} onclick={done}>Make Package</button>
  {/snippet}
</Dialog>

<style>

  h2 {
    margin: 0;
    color: var(--text-faint);
    font-size: var(--font-size-small);
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  section {
    display: grid;
    gap: 8px;
    margin: 0;
    padding: 10px;
    border: 1px solid var(--border);
    border-radius: 3px;
    background: transparent;
  }

  .line {
    display: grid;
    grid-template-columns: 72px 1fr;
    gap: 8px;
    align-items: center;
    color: var(--text-dim);
  }

  .top {
    display: grid;
    gap: 4px;
    color: var(--text-dim);
  }

  .count {
    justify-self: end;
    color: var(--text-faint);
    font-size: var(--font-size-small);
  }

  .count.over,
  .problem {
    color: var(--danger);
  }

  .problem {
    margin: 0;
  }

  .nums {
    display: flex;
    flex-wrap: wrap;
    gap: 8px 16px;
    align-items: center;
    color: var(--text-dim);
  }

  .nums label {
    display: flex;
    gap: 6px;
    align-items: center;
  }

  .check {
    color: var(--text);
  }

  .folder {
    display: flex;
    gap: 6px;
    align-items: center;
    min-width: 0;
  }

  .folder input {
    flex: 1;
  }

  input[readonly] {
    color: var(--text-dim);
  }

  .summary {
    margin: 0;
    color: var(--text-dim);
  }

  .summary strong {
    color: var(--value);
    font-weight: 600;
    word-break: break-all;
  }

</style>
