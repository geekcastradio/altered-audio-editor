<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Audio FX (dialogue): pick how much Restoration, Level, Clarity and Polish
  a voice gets. Changes apply on their own a moment after you stop clicking,
  as one undo step. It writes ordinary Voice Chain effects at the front of
  the chain, so everything can be opened and changed there afterward.
-->
<script lang="ts">
  import { onDestroy } from "svelte";
  import { editor, type EditorState } from "../../stores/editor";
  import { prefs } from "../../stores/prefs";
  import type { CommandRunner } from "../../stores/media-pool";
  import type { EffectChainDoc, EffectTarget, SessionDoc, SessionStore } from "../../stores/session";
  import { chainOf } from "../voice-chain/chain";
  import { LOOKS, NO_LOOKS, readLooks, sameLooks, withLooks, withoutLooks, type LookId, type Looks, type Strength } from "./audiofx";

  interface Props {
    session: SessionStore;
    runner: CommandRunner;
    /** The Waveform editor's file: there the chain is a preview and Apply in the Voice Chain writes it. */
    wave?: SessionStore;
    onNotice?: (m: string) => void;
    onError?: (e: unknown) => void;
  }

  const { session, runner, wave, onNotice = () => {}, onError = console.error }: Props = $props();

  let doc = $state<SessionDoc | null>(null);
  $effect(() =>
    session.subscribe((d) => {
      doc = d;
    }),
  );
  let waveDoc = $state<SessionDoc | null>(null);
  $effect(() =>
    wave?.subscribe((d) => {
      waveDoc = d;
    }),
  );
  let ed = $state<EditorState>(editor.current());
  $effect(() =>
    editor.subscribe((s) => {
      ed = s;
    }),
  );

  const inWave = $derived(ed.mode === "waveform");
  const oneClip = $derived(ed.selectedClips.size === 1 ? [...ed.selectedClips][0]! : null);
  const clipTrack = $derived(oneClip ? doc?.tracks.find((t) => t.clips.some((c) => c.id === oneClip)) : undefined);
  const clip = $derived(oneClip ? clipTrack?.clips.find((c) => c.id === oneClip) : undefined);
  const track = $derived(doc?.tracks.find((t) => t.id === ed.selectedTrack));

  /** What the looks go on: the open file's preview, one selected clip, or the selected track. */
  const aim = $derived.by((): { target: EffectTarget; chain: EffectChainDoc | undefined; name: string; type: string | null } | null => {
    if (inWave) {
      const t = waveDoc?.tracks[0];
      return t ? { target: { kind: "track", id: t.id }, chain: t.effects, name: "the open file (preview)", type: null } : null;
    }
    if (clip) return { target: { kind: "clip", id: clip.id }, chain: clip.effects, name: clip.name, type: clip.audio_type ?? null };
    if (track) return { target: { kind: "track", id: track.id }, chain: track.effects, name: track.name, type: null };
    return null;
  });
  /** A clip marked as music, effects or ambience is not dialogue: the looks are made for voices. */
  const notDialogue = $derived(!!aim?.type && aim.type !== "dialogue");

  let looks = $state<Looks>({ ...NO_LOOKS });
  /** Loads what the chain already carries whenever the aim or its chain changes from outside. */
  let loadedFrom = "";
  $effect(() => {
    const key = aim ? JSON.stringify([aim.target, (aim.chain?.slots ?? []).map((s) => s.preset)]) : "";
    if (key !== loadedFrom) {
      loadedFrom = key;
      looks = readLooks(aim?.chain);
    }
  });
  const saved = $derived(readLooks(aim?.chain));
  const changed = $derived(!sameLooks(looks, saved));
  const anyOn = $derived(LOOKS.some((l) => looks[l.id] !== 0));

  const AUTO_KEY = "audiofx.auto";
  let auto = $state(true);
  void prefs.load().then(() => {
    auto = prefs.raw(AUTO_KEY) !== false;
  });
  function setAuto(on: boolean) {
    auto = on;
    prefs.set(AUTO_KEY, on);
    if (on && changed) queue();
  }

  let timer: ReturnType<typeof setTimeout> | undefined;
  /** Applies a moment after the last click, so a few quick changes are one undo step. */
  function queue() {
    clearTimeout(timer);
    timer = setTimeout(() => void apply(), 350);
  }
  onDestroy(() => clearTimeout(timer));

  function set(id: LookId, s: Strength) {
    looks = { ...looks, [id]: s };
    if (auto) queue();
  }

  async function write(label: string, next: EffectChainDoc | null) {
    if (!aim) return;
    if (!next) {
      onNotice("That Voice Chain is full: it holds 16 effects.");
      return;
    }
    const op = { op: "set_effects", target: aim.target, chain: next } as const;
    try {
      // In the Waveform editor the chain is a preview, not an edit.
      if (inWave) await wave?.apply(op);
      else await session.edit(label, op, runner);
    } catch (e) {
      onError(e);
    }
  }

  // One label for every change, so the history merges a run of tweaks into one step.
  const apply = () => {
    clearTimeout(timer);
    if (!aim || sameLooks(looks, readLooks(aim.chain))) return;
    return write("Audio FX", withLooks(chainOf(aim.chain), looks, () => crypto.randomUUID()));
  };
  async function clear() {
    if (!aim) return;
    clearTimeout(timer);
    looks = { ...NO_LOOKS };
    await write("Audio FX", withoutLooks(chainOf(aim.chain)));
  }

  const labels = ["Off", "Light", "Medium", "Strong"] as const;
</script>

<div class="panel">
  {#if !aim}
    <p class="empty">Select a track or a clip, or open a file in the Waveform editor, to shape a voice.</p>
  {:else}
    <header>
      <p class="kicker">Dialogue</p>
      <p class="aim" title={aim.name}>{aim.name}</p>
    </header>

    {#if notDialogue}
      <p class="note">This clip is marked as {aim.type}. These looks are made for voices. Change it with Clip &gt; Audio Type, or use Loudness Match to set its level.</p>
    {/if}

    <div class="looks">
      {#each LOOKS as l (l.id)}
        <section>
          <div class="head">
            <h3 title={l.about}>{l.label}</h3>
            <span class="hint">{looks[l.id] === 0 ? "" : l.strengths[looks[l.id] - 1]}</span>
          </div>
          <div class="seg" role="radiogroup" aria-label={l.label} title={l.about}>
            {#each labels as name, i (name)}
              <button type="button" role="radio" aria-checked={looks[l.id] === i} class:on={looks[l.id] === i} onclick={() => set(l.id, i as Strength)}>{name}</button>
            {/each}
          </div>
        </section>
      {/each}
    </div>

    <footer>
      <label class="auto" title="Apply a moment after each change. Turn off to apply with the button."
        ><input type="checkbox" checked={auto} onchange={(e) => setAuto(e.currentTarget.checked)} /> Auto</label
      >
      {#if !auto}
        <button type="button" class="primary" disabled={!changed || (!anyOn && !LOOKS.some((l) => saved[l.id] !== 0))} onclick={() => void apply()}>
          {inWave ? "Preview" : "Apply"}
        </button>
      {/if}
      <button type="button" disabled={!LOOKS.some((l) => saved[l.id] !== 0)} onclick={() => void clear()}>Remove</button>
    </footer>
  {/if}
</div>

<style>
  .panel {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    height: 100%;
    padding: var(--space-3);
    overflow: auto;
    background: var(--surface-1);
    color: var(--text);
  }
  .empty,
  .note {
    margin: 0;
    color: var(--text-dim);
  }
  .note {
    padding: var(--space-2);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm, 4px);
    background: var(--surface-2);
  }
  header p {
    margin: 0;
  }
  .kicker {
    color: var(--text-dim);
    font-size: 0.8em;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
  .aim {
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .looks {
    display: grid;
    gap: var(--space-3);
  }
  section {
    display: grid;
    gap: var(--space-1);
  }
  .head {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
  }
  h3 {
    margin: 0;
    font-size: 1em;
  }
  .hint {
    color: var(--value, var(--text-dim));
    font-variant-numeric: tabular-nums;
  }
  .seg {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm, 4px);
    overflow: hidden;
  }
  .seg button {
    padding: var(--space-1) var(--space-2);
    border: 0;
    border-right: 1px solid var(--border);
    background: var(--surface-2);
    color: var(--text);
    cursor: pointer;
  }
  .seg button:last-child {
    border-right: 0;
  }
  .seg button.on {
    background: var(--accent);
    color: var(--on-accent, #fff);
  }
  .seg button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }
  footer {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
    margin-top: auto;
  }
  footer button {
    padding: var(--space-1) var(--space-3);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm, 4px);
    background: var(--surface-2);
    color: var(--text);
    cursor: pointer;
  }
  footer button.primary {
    background: var(--accent);
    border-color: var(--accent);
    color: var(--on-accent, #fff);
  }
  footer button:disabled {
    opacity: 0.45;
    cursor: default;
  }
  .auto {
    display: inline-flex;
    gap: var(--space-1);
    align-items: center;
    color: var(--text-dim);
  }
</style>
