<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Sound Types: say what a clip or track is (Dialogue, Music, SFX, Ambience),
  tick the jobs it needs, pick a starting preset and set one Amount. Changes
  apply on their own as one undo step. It writes ordinary Voice Chain effects
  at the front of the chain; edit them there and this panel shows Custom.
-->
<script lang="ts">
  import { onDestroy } from "svelte";
  import { editor, type EditorState } from "../../stores/editor";
  import { prefs } from "../../stores/prefs";
  import type { CommandRunner } from "../../stores/media-pool";
  import type { EffectChainDoc, EffectTarget, SessionDoc, SessionStore } from "../../stores/session";
  import { chainOf } from "../voice-chain/chain";
  import { isCustom, presetSetup, readSetup, TYPES, typeInfo, withoutSetup, withSetup, type Setup, type SoundType } from "./soundtypes";

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

  const aim = $derived.by((): { target: EffectTarget; chain: EffectChainDoc | undefined; name: string; type: SoundType | null } | null => {
    if (inWave) {
      const t = waveDoc?.tracks[0];
      return t ? { target: { kind: "track", id: t.id }, chain: t.effects, name: "the open file (preview)", type: null } : null;
    }
    if (clip) return { target: { kind: "clip", id: clip.id }, chain: clip.effects, name: clip.name, type: (clip.audio_type as SoundType | null | undefined) ?? null };
    if (track) return { target: { kind: "track", id: track.id }, chain: track.effects, name: track.name, type: null };
    return null;
  });

  let setup = $state<Setup>(presetSetup("dialogue", "clean"));
  /** Whether the panel has been told to make anything yet for this aim. */
  let touched = $state(false);
  let loadedFrom = "";
  $effect(() => {
    const key = aim ? JSON.stringify([aim.target, (aim.chain?.slots ?? []).map((s) => s.preset)]) : "";
    if (key !== loadedFrom) {
      loadedFrom = key;
      const found = readSetup(aim?.chain);
      touched = false;
      setup = found ?? presetSetup(aim?.type ?? "dialogue", typeInfo(aim?.type ?? "dialogue").presets[0]!.id);
    }
  });

  const saved = $derived(readSetup(aim?.chain));
  const custom = $derived(isCustom(aim?.chain, saved));
  const info = $derived(typeInfo(setup.type));
  const presetValue = $derived(info.presets.some((p) => p.id === setup.preset) ? setup.preset : "custom");

  const AUTO_KEY = "soundtypes.auto";
  let auto = $state(true);
  void prefs.load().then(() => {
    auto = prefs.raw(AUTO_KEY) !== false;
  });
  function setAuto(on: boolean) {
    auto = on;
    prefs.set(AUTO_KEY, on);
    if (on && touched) queue();
  }

  let timer: ReturnType<typeof setTimeout> | undefined;
  function queue() {
    clearTimeout(timer);
    timer = setTimeout(() => void apply(), 350);
  }
  onDestroy(() => clearTimeout(timer));

  function change(next: Setup) {
    setup = next;
    touched = true;
    if (auto) queue();
  }

  const pickType = (t: SoundType) => change(presetSetup(t, typeInfo(t).presets[0]!.id));
  function pickPreset(id: string) {
    if (id === "custom") return;
    change(presetSetup(setup.type, id));
  }
  const toggle = (id: string, on: boolean) =>
    change({ ...setup, preset: "custom", sections: on ? [...setup.sections, id] : setup.sections.filter((s) => s !== id) });
  const setAmount = (n: number) => change({ ...setup, preset: info.presets.some((p) => p.id === setup.preset) ? setup.preset : "custom", amount: n });

  async function write(label: string, next: EffectChainDoc | null) {
    if (!aim) return;
    if (!next) {
      onNotice("That Voice Chain is full: it holds 16 effects.");
      return;
    }
    const op = { op: "set_effects", target: aim.target, chain: next } as const;
    try {
      if (inWave) await wave?.apply(op);
      else await session.edit(label, op, runner);
    } catch (e) {
      onError(e);
    }
  }

  async function apply() {
    clearTimeout(timer);
    if (!aim) return;
    const s = setup.sections.length === 0 ? null : setup;
    // Same label each time, so a run of tweaks merges into one undo step.
    await write(`Sound Types: ${info.label}`, s ? withSetup(chainOf(aim.chain), s, () => crypto.randomUUID()) : withoutSetup(chainOf(aim.chain)));
    touched = false;
  }
  async function clear() {
    if (!aim) return;
    clearTimeout(timer);
    await write(`Sound Types: ${info.label}`, withoutSetup(chainOf(aim.chain)));
    touched = false;
  }
</script>

<div class="panel">
  {#if !aim}
    <p class="empty">Select a track or a clip, or open a file in the Waveform editor, to shape its sound.</p>
  {:else}
    <header>
      <p class="aim" title={aim.name}>{aim.name}</p>
    </header>

    <div class="seg" role="tablist" aria-label="Sound type">
      {#each TYPES as t (t.id)}
        <button type="button" role="tab" aria-selected={setup.type === t.id} class:on={setup.type === t.id} title={t.about} onclick={() => pickType(t.id)}>{t.label}</button>
      {/each}
    </div>

    <label class="row">
      <span>Preset</span>
      <select value={presetValue} onchange={(e) => pickPreset(e.currentTarget.value)}>
        {#each info.presets as p (p.id)}<option value={p.id}>{p.label}</option>{/each}
        {#if presetValue === "custom" || custom}<option value="custom">(Custom)</option>{/if}
      </select>
    </label>
    {#if custom && !touched}
      <p class="note">These effects were changed in the Voice Chain. Changing anything here rebuilds them from the settings below.</p>
    {/if}

    <div class="checks">
      {#each info.sections as sec (sec.id)}
        <label title={sec.about}>
          <input type="checkbox" checked={setup.sections.includes(sec.id)} onchange={(e) => toggle(sec.id, e.currentTarget.checked)} />
          {sec.label}
        </label>
      {/each}
    </div>

    <label class="row">
      <span>Amount</span>
      <input type="range" min="0" max="100" step="1" value={setup.amount} oninput={(e) => setAmount(Number(e.currentTarget.value))} />
      <output>{Math.round(setup.amount)}%</output>
    </label>

    <footer>
      <label class="auto" title="Apply a moment after each change. Turn off to apply with the button."
        ><input type="checkbox" checked={auto} onchange={(e) => setAuto(e.currentTarget.checked)} /> Auto</label
      >
      {#if !auto}
        <button type="button" class="primary" disabled={!touched} onclick={() => void apply()}>{inWave ? "Preview" : "Apply"}</button>
      {/if}
      <button type="button" disabled={!saved} onclick={() => void clear()}>Remove</button>
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
  .aim {
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
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
  .row {
    display: flex;
    gap: var(--space-2);
    align-items: center;
  }
  .row span {
    flex: none;
    width: 3.5em;
    color: var(--text-dim);
  }
  .row select,
  .row input[type="range"] {
    flex: 1;
    min-width: 0;
  }
  output {
    flex: none;
    min-width: 3em;
    text-align: right;
    font-variant-numeric: tabular-nums;
  }
  .checks {
    display: grid;
    gap: var(--space-1);
  }
  .checks label {
    display: flex;
    gap: var(--space-2);
    align-items: center;
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
