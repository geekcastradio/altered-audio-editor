<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  File > Export > Mixdown: file name and folder, the format, and three
  summary boxes (sample type, format settings, mixdown options) that each
  open their own small window with Change. Shows what the file will be and
  roughly how big before anything is written. Enter exports, Escape
  cancels (or closes the small window that is open).
-->
<script lang="ts" module>
  import type { FormatMemory, OutputFormat } from "./output-format";

  export type ExportScope = "all" | "selection" | "clips";

  export interface ExportSettings {
    scope: ExportScope;
    name: string;
    folder: string;
    format: OutputFormat;
    addToFiles: boolean;
    /** Write chapter markers into the file (WAV, FLAC, MP3 and M4A). */
    chapters: boolean;
    /** Also write every other marker: WAV cue points, or a FLAC cue sheet. */
    markers?: boolean;
    /** Read the file back after saving and check it. */
    verify: boolean;
    /** A loudness target to bring the file to (an id from Check Loudness); "" leaves it as mixed. */
    normalize: string;
    /** 3D Space sessions: a speaker layout ("5.1", "7.1.4"...) or "binaural". */
    speakers?: string;
    /** Each format's own remembered settings, so an MP3 and a WAV master keep theirs. */
    formats?: FormatMemory;
  }

  export interface ExportScopeInfo {
    scope: ExportScope;
    label: string;
    /** Length in seconds, or null when unavailable. */
    seconds: number | null;
    /** Why it's unavailable. */
    reason: string;
  }
</script>

<script lang="ts">
  import OutputFormatFields from "./OutputFormatFields.svelte";
  import InfoTip from "./InfoTip.svelte";
  import { PLATFORM } from "../lib/platform";
  import { describeFormat, describeSampleType, describeSettings, estimateBytes, extensionOf, formatAvailable, FORMAT_INFO, FORMAT_KINDS, switchFormat, type FormatKind } from "./output-format";
  import { SPACE_LAYOUTS, type SpaceLayout } from "../stores/session";
  import { targetsFor, type LoudnessReport } from "./loudness";
  import LoudnessScan from "./LoudnessScan.svelte";
  import { LAYOUT_NAMES } from "../panels/space/space";

  interface Props {
    initial: ExportSettings;
    scopes: ExportScopeInfo[];
    source: { sampleRate: number; channels: number; bitDepth: number };
    onBrowse: (from: string) => Promise<string | null>;
    /** Measures the chosen part of the mix without writing a file. */
    onScan?: (scope: ExportScope) => Promise<LoudnessReport | null>;
    /** Chapter markers in the session. */
    chapterCount?: number;
    /** Markers of every type in the session. */
    markerCount?: number;
    /** 3D Space sessions: the session's speaker layout, to offer every layout and binaural. */
    spaceLayout?: SpaceLayout | null;
    onDone: (value: ExportSettings | null) => void;
  }

  const { initial, scopes, source, onBrowse, onScan, chapterCount = 0, markerCount = 0, spaceLayout = null, onDone }: Props = $props();

  // svelte-ignore state_referenced_locally
  let s = $state<ExportSettings>({ ...initial, format: { ...initial.format }, formats: { ...(initial.formats ?? {}) } });
  let input = $state<HTMLInputElement | null>(null);
  /** The small window that is open, if any. */
  let sub = $state<null | "type" | "settings" | "mix">(null);

  $effect(() => {
    input?.focus();
    input?.select();
  });

  const current = $derived(scopes.find((x) => x.scope === s.scope));
  /** A layout with more than two speakers is written as a WAV with one channel per speaker. */
  const multi = $derived(!!spaceLayout && s.speakers !== undefined && s.speakers !== "binaural" && s.speakers !== "2.0");
  $effect(() => {
    if (multi && s.format.kind !== "wav") changeKind("wav");
  });
  const targets = targetsFor();
  const ext = $derived(extensionOf(s.format.kind));
  const canChapter = $derived(["wav", "flac", "mp3", "m4a"].includes(s.format.kind));
  const summary = $derived(describeFormat(s.format, source));
  const sampleType = $derived(describeSampleType(s.format, source));
  const settingsText = $derived(describeSettings(s.format));
  const mixText = $derived.by(() => {
    const parts = [current?.label ?? ""];
    if (current?.seconds !== null && current?.seconds !== undefined) parts.push(clock(current.seconds));
    if (spaceLayout && s.speakers) parts.push(s.speakers === "binaural" ? "headphones" : (LAYOUT_NAMES[s.speakers as SpaceLayout] ?? s.speakers));
    return parts.filter(Boolean).join(", ");
  });
  const size = $derived.by(() => {
    const secs = current?.seconds;
    if (secs === null || secs === undefined) return "";
    const b = estimateBytes(s.format, secs, source);
    return b >= 1e9 ? `${(b / 1e9).toFixed(2)} GB` : b >= 1e6 ? `${(b / 1e6).toFixed(1)} MB` : `${Math.max(1, Math.round(b / 1e3))} KB`;
  });
  const ok = $derived(s.name.trim() !== "" && s.folder !== "" && (current?.seconds ?? null) !== null);
  const loudOn = $derived(s.normalize !== "");
  const available = (k: FormatKind) => formatAvailable(k, PLATFORM);

  function clock(sec: number): string {
    const t = Math.round(sec);
    const h = Math.floor(t / 3600);
    const m = Math.floor((t % 3600) / 60);
    const r = t % 60;
    return h > 0 ? `${h}:${String(m).padStart(2, "0")}:${String(r).padStart(2, "0")}` : `${m}:${String(r).padStart(2, "0")}`;
  }

  /** Switching format keeps each format's own settings. */
  function changeKind(kind: FormatKind) {
    if (kind === s.format.kind) return;
    const next = switchFormat(s.formats ?? {}, s.format, kind);
    s.formats = next.memory;
    s.format = next.format;
  }

  function toggleLoudness(on: boolean) {
    s.normalize = on ? (targets[0]?.id ?? "") : "";
  }

  function done() {
    if (!ok) return;
    const formats = { ...(s.formats ?? {}), [s.format.kind]: { ...s.format } };
    onDone({ ...s, formats, name: s.name.trim().replace(/\.(wav|wave|flac|mp3|m4a|aac)$/i, "") });
  }

  async function browse() {
    const f = await onBrowse(s.folder);
    if (f) s.folder = f;
  }

  function key(e: KeyboardEvent) {
    e.stopPropagation();
    if (e.key === "Enter" && !(e.target instanceof HTMLSelectElement) && !(e.target instanceof HTMLButtonElement)) {
      e.preventDefault();
      if (sub) sub = null;
      else done();
    } else if (e.key === "Escape") {
      e.preventDefault();
      if (sub) sub = null;
      else onDone(null);
    }
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="backdrop" onclick={() => onDone(null)}>
  <div class="dialog" role="dialog" aria-modal="true" aria-labelledby="export-title" tabindex="-1" onclick={(e) => e.stopPropagation()} onkeydown={key}>
    <h1 id="export-title">Export Mixdown</h1>

    <label class="line">
      <span>File name</span>
      <span class="name">
        <input type="text" spellcheck="false" bind:value={s.name} bind:this={input} />
        <span class="ext">.{ext}</span>
      </span>
    </label>
    <div class="line">
      <span>Location</span>
      <span class="folder">
        <input type="text" readonly value={s.folder} title={s.folder} placeholder="Choose a folder" />
        <button type="button" onclick={browse}>Browse…</button>
      </span>
    </div>
    <div class="line">
      <label for="export-format">Format</label>
      <span class="name">
        <select id="export-format" value={s.format.kind} onchange={(e) => changeKind(e.currentTarget.value as FormatKind)} disabled={multi}>
          {#each FORMAT_KINDS.filter((k) => k.group === "main") as k (k.value)}
            <option value={k.value} disabled={!available(k.value)}>{k.label}{available(k.value) ? "" : " (Windows only for now)"}</option>
          {/each}
          <optgroup label="Other formats">
            {#each FORMAT_KINDS.filter((k) => k.group === "other") as k (k.value)}
              <option value={k.value} disabled={!available(k.value)}>{k.label}{available(k.value) ? "" : " (Windows only for now)"}</option>
            {/each}
          </optgroup>
        </select>
        <InfoTip text={FORMAT_INFO[s.format.kind]} label="About this format" />
      </span>
    </div>
    {#if multi}<p class="note">Surround layouts are written as a WAV with one channel per speaker.</p>{/if}

    <div class="boxes">
      <div class="box">
        <div class="head"><h2>Sample type</h2><button type="button" onclick={() => (sub = "type")}>Change…</button></div>
        <p>{sampleType}</p>
      </div>
      <div class="box">
        <div class="head">
          <h2>Format settings</h2>
          <button type="button" disabled={settingsText === null} title={settingsText === null ? "This format has no extra options." : ""} onclick={() => (sub = "settings")}>Change…</button>
        </div>
        <p>{settingsText ?? "No extra options"}</p>
      </div>
      <div class="box">
        <div class="head"><h2>Mixdown options</h2><button type="button" onclick={() => (sub = "mix")}>Change…</button></div>
        <p>{mixText}</p>
      </div>
    </div>

    <p class="summary">
      <strong>{summary}</strong>{#if size}<span>&nbsp;· about {size}</span>{/if}
    </p>

    <section class="options">
      <label class="check"><input type="checkbox" bind:checked={s.addToFiles} /> Add the file to the Files panel</label>
      {#if chapterCount > 0}
        <label class="check" class:off={!canChapter} title={canChapter ? "" : "Chapters go into WAV, FLAC, MP3 and M4A files. For other formats, use File > Export > Markers and Chapters."}>
          <input type="checkbox" bind:checked={s.chapters} disabled={!canChapter} /> Add the {chapterCount} chapter {chapterCount === 1 ? "marker" : "markers"} to the file
        </label>
      {:else}
        <label class="check off" title="Set a marker's type to Chapter, or use Edit > Markers > Add Chapter Marker.">
          <input type="checkbox" disabled /> Add chapters to the file (no chapter markers yet)
        </label>
      {/if}
      {#if markerCount > 0 && (s.format.kind === "wav" || s.format.kind === "flac")}
        <label
          class="check"
          title={s.format.kind === "wav"
            ? "Every marker, not only chapters, goes in as a cue point that other editors can read. Ranges keep their length. This replaces the chapter cue points, since chapters are included."
            : "Writes a FLAC cue sheet with a track at each marker start. A cue sheet has no names, and players built on ffmpeg show it as unnamed chapters instead of the named ones, so leave this off unless a program asks for a cue sheet."}
        >
          <input type="checkbox" bind:checked={s.markers} />
          {s.format.kind === "wav" ? `Add all ${markerCount} ${markerCount === 1 ? "marker" : "markers"} as cue points` : `Add all ${markerCount} ${markerCount === 1 ? "marker" : "markers"} as a cue sheet`}
        </label>
      {/if}
      <label class="check" title="Reads the finished file back from start to end: it opens, every sample decodes, it is the right length and the chapters are in it."><input type="checkbox" bind:checked={s.verify} /> Check the file after saving</label>
      <label class="check" class:off={multi} title="Measures the mix first, then raises or lowers the whole file by one fixed amount to reach the target. Peaks stay under the target's limit, so a mix with high peaks can end up a little quieter than the target. The session itself is not changed. Mono targets are measured when the session mixes to mono.">
        <input type="checkbox" checked={loudOn} disabled={multi} onchange={(e) => toggleLoudness(e.currentTarget.checked)} /> Bring the file to a loudness target
      </label>
      {#if loudOn && !multi}
        <div class="indent">
          <select bind:value={s.normalize} aria-label="Loudness target">
            {#each targets as t (t.id)}<option value={t.id}>{t.name}</option>{/each}
          </select>
          {#if onScan}
            <LoudnessScan target={s.normalize} mono={s.format.channels === 1 || (s.format.channels === null && source.channels === 1)} stamp={`${s.scope}|${s.format.channels}`} scan={() => onScan(s.scope)} />
          {/if}
        </div>
      {/if}
    </section>

    <footer>
      <button type="button" onclick={() => onDone(null)}>Cancel</button>
      <button type="button" class="primary" disabled={!ok} onclick={done}>Export</button>
    </footer>

    {#if sub}
      <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
      <div class="sub-backdrop" onclick={() => (sub = null)}>
        <div class="dialog sub" role="dialog" aria-modal="true" aria-labelledby="export-sub-title" tabindex="-1" onclick={(e) => e.stopPropagation()}>
          {#if sub === "type"}
            <h1 id="export-sub-title">Sample type</h1>
            <OutputFormatFields bind:format={s.format} sourceLabel="Same as the session" parts="type" />
            <p class="note">"Same as the session" keeps what the mix already is. {FORMAT_KINDS.find((k) => k.value === s.format.kind)?.lossy ? "This format is lossy, so it has no bit depth to choose." : ""}</p>
          {:else if sub === "settings"}
            <h1 id="export-sub-title">Format settings</h1>
            <OutputFormatFields bind:format={s.format} sourceLabel="Same as the session" parts="settings" />
            {#if s.format.kind === "m4a" || s.format.kind === "aac"}
              <p class="note">AAC is written at 44.1 or 48 kHz, mono or stereo, using the encoder built into Windows.</p>
            {/if}
          {:else}
            <h1 id="export-sub-title">Mixdown options</h1>
            <div class="scopes" role="radiogroup" aria-label="What to mix">
              {#each scopes as x (x.scope)}
                <label class="radio" class:off={x.seconds === null} title={x.seconds === null ? x.reason : ""}>
                  <input type="radio" name="scope" value={x.scope} bind:group={s.scope} disabled={x.seconds === null} />
                  {x.label}{x.seconds !== null ? ` (${clock(x.seconds)})` : ""}
                </label>
              {/each}
            </div>
            {#if spaceLayout}
              <label class="line wide" title="The same session plays in any layout: pick one for this file. Layouts with more than two speakers are written as WAV with one channel per speaker, labeled for other software. Headphones writes a stereo binaural mix.">
                <span>Speaker layout</span>
                <select bind:value={s.speakers}>
                  {#each SPACE_LAYOUTS as l (l)}<option value={l}>{LAYOUT_NAMES[l]}{l === spaceLayout ? " (the session's)" : ""}</option>{/each}
                  <option value="binaural">Headphones (binaural stereo)</option>
                </select>
              </label>
            {/if}
          {/if}
          <footer><button type="button" class="primary" onclick={() => (sub = null)}>OK</button></footer>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 5000;
    display: grid;
    place-items: center;
    background: rgba(0, 0, 0, 0.5);
  }

  .dialog {
    display: grid;
    gap: 12px;
    width: min(520px, 92vw);
    max-height: 92vh;
    overflow: auto;
    padding: 14px;
    border: 1px solid var(--border-strong);
    background: var(--surface-2);
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.6);
  }

  h1 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
  }

  h2 {
    margin: 0;
    padding: 0;
    color: var(--text-faint);
    font-size: var(--font-size-small);
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .box button {
    height: 22px;
    padding: 0 10px;
  }

  section {
    display: grid;
    gap: 8px;
    margin: 0;
    padding: 10px;
    border: 1px solid var(--border);
    border-radius: 3px;
    background: var(--surface-1);
  }

  .scopes {
    display: flex;
    flex-wrap: wrap;
    gap: 6px 18px;
  }

  .line {
    display: grid;
    grid-template-columns: 56px 1fr;
    gap: 8px;
    align-items: center;
    color: var(--text-dim);
  }

  .name,
  .folder {
    display: flex;
    gap: 6px;
    align-items: center;
    min-width: 0;
  }

  .ext {
    color: var(--text-faint);
    font-family: var(--font-mono);
  }

  input[type="text"] {
    flex: 1;
    min-width: 0;
    height: 24px;
    padding: 0 8px;
    border: 1px solid var(--border-strong);
    border-radius: 2px;
    background: var(--surface-0);
    color: var(--text);
    font: inherit;
  }

  input[readonly] {
    color: var(--text-dim);
  }

  .summary {
    margin: 2px 0 0;
    color: var(--text-dim);
  }

  .summary strong {
    color: var(--value);
    font-weight: 600;
  }

  .radio,
  .check {
    display: flex;
    gap: 6px;
    align-items: center;
    color: var(--text);
  }

  .off {
    color: var(--text-faint);
  }

  .options {
    gap: 6px;
  }

  footer {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
  }

  button {
    height: 26px;
    padding: 0 14px;
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

  .dialog {
    position: relative;
  }

  .line > label,
  .line > span:first-child {
    color: var(--text-dim);
  }

  .line {
    grid-template-columns: 76px 1fr;
  }

  .line.wide {
    grid-template-columns: 110px 1fr;
  }

  select {
    flex: 1;
    min-width: 0;
    height: 24px;
    padding: 0 6px;
    border: 1px solid var(--border-strong);
    border-radius: 2px;
    background: var(--surface-0);
    color: var(--text);
    font: inherit;
  }

  .boxes {
    display: grid;
    gap: 6px;
  }

  .box {
    padding: 6px 10px;
    border: 1px solid var(--border);
    border-radius: 3px;
    background: var(--surface-1);
  }

  .box .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .box p {
    margin: 2px 0 0;
    color: var(--value);
    font-family: var(--font-mono);
    font-size: var(--font-size-small);
  }

  .note {
    margin: 0;
    color: var(--text-faint);
    font-size: var(--font-size-small);
  }

  .indent {
    display: grid;
    gap: 6px;
    padding-left: 22px;
  }

  .sub-backdrop {
    position: fixed;
    inset: 0;
    z-index: 5100;
    display: grid;
    place-items: center;
    background: rgba(0, 0, 0, 0.45);
  }

  .dialog.sub {
    width: min(440px, 90vw);
    max-height: 80vh;
  }
</style>
