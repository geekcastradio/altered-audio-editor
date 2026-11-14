<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  File > Export > Edit Lists: writes the cut as EDLs (one per track), a marker EDL for DaVinci
  Resolve, or an OpenTimelineIO file, so it can move to another editor.
-->
<script lang="ts">
  import Dialog from "./Dialog.svelte";
  import type { InterchangeKind, InterchangeRequest, InterchangeResult } from "./core";

  interface Props {
    hasClips: boolean;
    hasMarkers: boolean;
    initial: { kind: InterchangeKind; rate: string; start: string; color: string };
    /** Asks for a folder, writes the files and gives back what was written. Null when cancelled. */
    run: (request: InterchangeRequest) => Promise<InterchangeResult | null>;
    onRemember: (v: { kind: InterchangeKind; rate: string; start: string; color: string }) => void;
    onClose: () => void;
  }

  const { hasClips, hasMarkers, initial, run, onRemember, onClose }: Props = $props();

  const KINDS: { id: InterchangeKind; label: string; hint: string }[] = [
    { id: "edl", label: "EDL, one file per track (CMX3600)", hint: "Opens in most editors. An EDL holds one clip at a time, so each track gets its own file." },
    { id: "marker-edl", label: "Markers as an EDL (DaVinci Resolve)", hint: "Imports as timeline markers. Check the first import in Resolve before relying on it." },
    { id: "otio", label: "OpenTimelineIO (.otio)", hint: "Keeps tracks, clip names, source files and markers in one file." },
  ];
  const RATES = [
    { id: "23.976", label: "23.976" },
    { id: "24", label: "24" },
    { id: "25", label: "25" },
    { id: "29.97", label: "29.97" },
    { id: "29.97df", label: "29.97 drop frame" },
    { id: "30", label: "30" },
  ];
  const COLORS = ["Blue", "Cyan", "Green", "Yellow", "Red", "Pink", "Purple", "Fuchsia"];

  // svelte-ignore state_referenced_locally
  let kind = $state<InterchangeKind>(initial.kind);
  // svelte-ignore state_referenced_locally
  let rate = $state(initial.rate);
  // svelte-ignore state_referenced_locally
  let start = $state(initial.start);
  // svelte-ignore state_referenced_locally
  let color = $state(initial.color);
  let busy = $state(false);
  let error = $state("");
  let done = $state<InterchangeResult | null>(null);

  const startOk = $derived(/^\d{1,2}[:;]\d{2}[:;]\d{2}[:;]\d{2}$/.test(start.trim()));
  const blocked = $derived(
    kind === "marker-edl" ? (hasMarkers ? "" : "The session has no markers.") : hasClips ? "" : "The session has no clips.",
  );
  const hint = $derived(KINDS.find((k) => k.id === kind)?.hint ?? "");

  async function go() {
    if (busy || blocked || !startOk) return;
    busy = true;
    error = "";
    done = null;
    const request: InterchangeRequest = { kind, rate, start: start.trim().replace(/;/g, ":"), color };
    try {
      const r = await run(request);
      if (r) {
        done = r;
        onRemember({ kind, rate, start: start.trim(), color });
      }
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }

  const name = (p: string) => p.split(/[\\/]/).pop() ?? p;
</script>

<Dialog title="Export Edit Lists" width={500} labelWidth={96} canClose={!busy} onEnter={() => void go()} {onClose}>
  <label class="row">
    <span>Format</span>
    <select bind:value={kind} data-autofocus>
      {#each KINDS as k (k.id)}<option value={k.id}>{k.label}</option>{/each}
    </select>
  </label>
  <p class="hint">{hint}</p>
  <label class="row">
    <span>Frame rate</span>
    <select bind:value={rate}>
      {#each RATES as r (r.id)}<option value={r.id}>{r.label}</option>{/each}
    </select>
  </label>
  <label class="row">
    <span>Starts at</span>
    <input type="text" bind:value={start} spellcheck="false" aria-invalid={!startOk} />
  </label>
  {#if kind === "marker-edl"}
    <label class="row">
      <span>Marker color</span>
      <select bind:value={color}>
        {#each COLORS as c (c)}<option value={c}>{c}</option>{/each}
      </select>
    </label>
  {/if}
  <p class="hint">Fades, volume automation and effects stay behind: these formats do not carry them.</p>
  {#if !startOk}<p class="warn">Start time looks like 01:00:00:00.</p>{/if}
  {#if blocked}<p class="warn">{blocked}</p>{/if}
  {#if error}<p class="warn" role="alert">{error}</p>{/if}
  {#if done}
    <div class="result" role="status">
      <p>Wrote {done.files.length} {done.files.length === 1 ? "file" : "files"}:</p>
      <ul>{#each done.files as f (f)}<li>{name(f)}</li>{/each}</ul>
      {#each done.warnings as w (w)}<p class="warn">{w}</p>{/each}
    </div>
  {/if}
  {#snippet footer()}
    <button type="button" onclick={onClose} disabled={busy}>{done ? "Close" : "Cancel"}</button>
    <button type="button" class="primary" disabled={busy || !!blocked || !startOk} onclick={() => void go()}>
      {busy ? "Writing..." : "Choose Folder and Export..."}
    </button>
  {/snippet}
</Dialog>

<style>
  .hint {
    margin: 0;
    color: var(--text-dim);
    font-size: 11px;
  }
  .result ul {
    margin: 2px 0 6px;
    padding-left: 18px;
    font-size: 12px;
  }
  .result p {
    margin: 0;
    font-size: 12px;
  }
</style>
