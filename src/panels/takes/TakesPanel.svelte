<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Takes panel. Lists the sets of takes in the session: several clips over
  the same passage of a track, where only the chosen pieces play. Pick a
  take to make it play over the time selection (or over the whole set when
  nothing is selected); the editor cuts the takes at the edges of the
  range, so every piece stays an ordinary clip and every step can be undone.
-->
<script lang="ts">
  import { editor, type EditorState } from "../../stores/editor";
  import type { SessionDoc, SessionStore } from "../../stores/session";
  import { settings, type AppSettings } from "../../stores/settings";
  import { clipEnd, mediaOf } from "../editor/clip-edit";
  import { makeTakesOps, takeSets, type TakeLane, type TakeSet } from "../editor/takes";

  interface Props {
    session: SessionStore;
    onMake: () => void;
    onPick: (set: string, number: number) => void;
    onFlatten: (set: string) => void;
    onDelete: (set: string, number: number) => void;
  }

  const { session, onMake, onPick, onFlatten, onDelete }: Props = $props();

  let doc = $state<SessionDoc | null>(null);
  $effect(() =>
    session.subscribe((d) => {
      doc = d;
    }),
  );
  let ed = $state<EditorState>(editor.current());
  $effect(() =>
    editor.subscribe((s) => {
      ed = s;
    }),
  );
  let prefs = $state<AppSettings>(settings.current);
  $effect(() =>
    settings.subscribe((s) => {
      prefs = s;
    }),
  );

  const rate = $derived(doc?.sample_rate ?? 48_000);
  const sets = $derived(doc ? takeSets(doc) : []);
  const canMake = $derived(!!doc && makeTakesOps(doc, ed.selectedClips, () => "x") !== null);
  const range = $derived(doc?.view.selection ?? null);

  const seconds = (n: number) => `${(n / rate).toFixed(1)} s`;
  const trackName = (id: string) => doc?.tracks.find((t) => t.id === id)?.name ?? "Track";

  /** How much of the set's stretch this lane plays, 0 to 1. */
  function share(s: TakeSet, lane: TakeLane): number {
    if (!doc) return 0;
    const media = mediaOf(doc);
    const span = s.end - s.start;
    if (span <= 0) return 0;
    let n = 0;
    for (const c of lane.clips) if (c.take?.active) n += clipEnd(c, media, rate) - c.start;
    return Math.min(1, n / span);
  }
</script>

<section class="takes" aria-label="Takes">
  {#if !prefs.takeLanes}
    <p class="empty">Take lanes are off. Turn them on in Settings &gt; Multitrack. Sessions with takes still play the chosen takes.</p>
  {:else}
    <div class="toolbar">
      <button type="button" disabled={!canMake} title="Select two or more clips on one track first. They become takes of one passage." onclick={onMake}>Make Takes from Selected Clips</button>
    </div>
    {#if sets.length === 0}
      <p class="empty">No takes yet. Stack two or more clips over the same passage on one track, select them, then choose Make Takes from Selected Clips.</p>
    {:else}
      <div class="list">
        {#each sets as s (s.set)}
          <div class="set">
            <div class="head">
              <strong>{trackName(s.trackId)}</strong>
              <span class="when">{seconds(s.start)} to {seconds(s.end)}</span>
              <button type="button" title="Delete the takes that are not playing and keep the rest as ordinary clips." onclick={() => onFlatten(s.set)}>Flatten</button>
            </div>
            {#each s.lanes as lane (lane.number)}
              <div class="lane">
                <span class="name">Take {lane.number}</span>
                <span class="bar" title="How much of this stretch plays from this take"><span class="fill" style:width="{Math.round(share(s, lane) * 100)}%"></span></span>
                <button type="button" title={range ? "Play this take over the time selection." : "Play this take over the whole stretch."} onclick={() => onPick(s.set, lane.number)}>{range ? "Use over selection" : "Use"}</button>
                <button type="button" title="Delete this take." onclick={() => onDelete(s.set, lane.number)}>Delete</button>
              </div>
            {/each}
          </div>
        {/each}
      </div>
    {/if}
  {/if}
</section>

<style>
  .takes {
    container-type: inline-size;
    display: grid;
    grid-template-rows: auto minmax(0, 1fr);
    height: 100%;
    background: var(--surface-1);
    color: var(--text);
    overflow: auto;
  }

  .toolbar {
    display: flex;
    gap: 4px;
    padding: 4px 6px;
    border-bottom: 1px solid var(--border);
  }

  button {
    height: 20px;
    padding: 0 8px;
    border: 1px solid var(--border-strong);
    border-radius: 2px;
    background: var(--surface-3);
    color: var(--text);
    font: inherit;
    font-size: var(--font-size-small);
  }

  button:disabled {
    color: var(--text-faint);
  }

  .empty {
    margin: 0;
    padding: 16px;
    color: var(--text-faint);
    text-align: center;
  }

  .list {
    overflow: auto;
  }

  .set {
    padding: 6px;
    border-bottom: 1px solid var(--border);
  }

  .head {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 4px;
  }

  .head .when {
    flex: 1;
    color: var(--text-faint);
    font-size: var(--font-size-small);
  }

  .lane {
    display: grid;
    grid-template-columns: 60px minmax(40px, 1fr) auto auto;
    align-items: center;
    gap: 6px;
    padding: 2px 0;
  }

  .bar {
    height: 8px;
    border: 1px solid var(--border-strong);
    border-radius: 2px;
    background: var(--surface-2);
    overflow: hidden;
  }

  .fill {
    display: block;
    height: 100%;
    background: var(--accent, #4a90d9);
  }
</style>
