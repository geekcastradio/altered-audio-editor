<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Voice Chain: up to 16 effects in order on the selected clip or track.
  Pick a preset or add effects one by one, bypass or reorder them, and
  set each effect's controls below the list. Every change is one
  undoable step, and playback picks it up from where it is: sliders
  apply as they move (a drag is one step), and the meters show the
  track chain's level going in and coming out.

  In the Waveform editor the chain is a preview on the open file: it
  plays but changes nothing until Apply writes it into the selection
  (or the whole file) as one undoable edit.
-->
<script lang="ts">
  import { effectsCatalog, learnNoisePrint, type EffectInfo, type EffectParamInfo, type LearnSource } from "../../app/core";
  import { editor, type EditorState } from "../../stores/editor";
  import type { CommandRunner } from "../../stores/media-pool";
  import type { Preferences } from "../../stores/preferences";
  import { meterFraction, type MeterLevel, type TransportState, type TransportStore } from "../../stores/transport";
  import type { EffectChainDoc, EffectSlotDoc, EffectTarget, SessionDoc, SessionStore } from "../../stores/session";
  import {
    addsOne,
    addSlot,
    applyPreset,
    BUILTIN_PRESETS,
    CATEGORY_COLORS,
    chainOf,
    formatParam,
    fromSlider,
    MAX_SLOTS,
    moveSlot,
    newSlot,
    paramValue,
    presetFrom,
    removeSlot,
    sanitizePresets,
    setParam,
    setSlot,
    toSlider,
    type ChainPreset,
  } from "./chain";
  import ResponseGraph from "./ResponseGraph.svelte";
  import { GRAPHED } from "./graph";

  interface Props {
    session: SessionStore;
    runner: CommandRunner;
    prefs: Preferences;
    ask: (title: string, label: string, value: string) => Promise<string | null>;
    /** Playback, for the chain meters. */
    transport?: TransportStore;
    /** The Waveform editor's file and playback, for the preview chain. */
    wave?: SessionStore;
    waveTransport?: TransportStore;
    /** Writes the preview chain into the file (selection or whole file). */
    onApplyWave?: (chain: EffectChainDoc) => Promise<void>;
    onNotice?: (m: string) => void;
    onError?: (e: unknown) => void;
  }

  const { session, runner, prefs, ask, transport, wave, waveTransport, onApplyWave, onNotice = () => {}, onError = console.error }: Props = $props();
  const KEY_PRESETS = "voiceChain.presets";
  const KEY_TAB = "voiceChain.tab";

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

  let catalog = $state<EffectInfo[]>([]);
  let loadError = $state<string | null>(null);
  void effectsCatalog()
    .then((c) => (catalog = c))
    .catch((e) => (loadError = String(e)));
  const byId = $derived(new Map(catalog.map((e) => [e.id, e])));
  const known = $derived(new Set(catalog.map((e) => e.id)));
  const categories = $derived([...new Set(catalog.map((e) => e.category))]);

  let userPresets = $state<ChainPreset[]>([]);
  let tab = $state<"clip" | "track">("track");
  // svelte-ignore state_referenced_locally
  void prefs.load().then(() => {
    userPresets = sanitizePresets(prefs.raw(KEY_PRESETS));
    if (prefs.raw(KEY_TAB) === "clip") tab = "clip";
  });
  function setTab(t: "clip" | "track") {
    tab = t;
    prefs.set(KEY_TAB, t);
  }

  // What the chain belongs to.
  const oneClip = $derived(ed.selectedClips.size === 1 ? [...ed.selectedClips][0]! : null);
  const clipTrack = $derived(oneClip ? doc?.tracks.find((t) => t.clips.some((c) => c.id === oneClip)) : undefined);
  // The track picked in the panel wins; otherwise the selected track,
  // the selected clip's track, or the first track, so the chain is never
  // hidden just because nothing is selected yet.
  let pickedTrack = $state<string | null>(null);
  // Derived values only signal when they change, so this runs on a new
  // selection, not on every editor update.
  const selTrack = $derived(ed.selectedTrack);
  $effect(() => {
    // A new selection in the editor takes over from the panel's pick.
    void selTrack;
    void oneClip;
    pickedTrack = null;
  });
  const track = $derived(
    doc?.tracks.find((t) => t.id === pickedTrack) ?? doc?.tracks.find((t) => t.id === ed.selectedTrack) ?? clipTrack ?? doc?.tracks[0],
  );
  const clip = $derived(oneClip ? clipTrack?.clips.find((c) => c.id === oneClip) : undefined);
  const inWave = $derived(ed.mode === "waveform");
  const waveTrack = $derived(waveDoc?.tracks[0]);
  const target = $derived.by((): EffectTarget | null => {
    if (inWave) return waveTrack ? { kind: "track", id: waveTrack.id } : null;
    if (tab === "clip") return clip ? { kind: "clip", id: clip.id } : null;
    return track ? { kind: "track", id: track.id } : null;
  });
  const why = $derived(
    inWave
      ? "Open a file in the Waveform editor to try a Voice Chain on it."
      : tab === "clip"
        ? ed.selectedClips.size > 1
          ? "Select one clip to see its effects."
          : "Select a clip to give it effects."
        : "Add a track to the session to give it effects.",
  );
  const label = $derived(inWave ? "Preview on the open file" : tab === "clip" ? (clip ? `Clip: ${clip.name}` : "") : track ? `Track: ${track.name}` : "");
  const chain = $derived(chainOf(inWave ? waveTrack?.effects : tab === "clip" ? clip?.effects : track?.effects));

  let selectedSlot = $state<string | null>(null);
  const slot = $derived(chain.slots.find((s) => s.id === selectedSlot) ?? chain.slots[0] ?? null);
  const slotInfo = $derived(slot ? byId.get(slot.effect) : undefined);

  function commit(what: string, next: EffectChainDoc | null) {
    if (!target) return;
    if (!next) {
      onNotice(`A Voice Chain holds ${MAX_SLOTS} effects.`);
      return;
    }
    const op = { op: "set_effects", target, chain: next } as const;
    // The Waveform preview is not an edit: it never enters undo, and Apply is the edit.
    if (inWave) void wave?.apply(op).catch(onError);
    else void session.edit(what, op, runner).catch(onError);
  }

  const waveSel = $derived(waveDoc?.view.selection && waveDoc.view.selection.end > waveDoc.view.selection.start ? waveDoc.view.selection : null);
  let applying = $state(false);
  async function applyWave() {
    if (!onApplyWave || applying) return;
    applying = true;
    try {
      await onApplyWave($state.snapshot(chain) as EffectChainDoc);
    } catch (e) {
      onError(e);
    } finally {
      applying = false;
    }
  }

  function add(id: string) {
    const info = byId.get(id);
    if (!info) return;
    const s = newSlot(info, crypto.randomUUID());
    selectedSlot = s.id;
    commit(`Add ${info.name}`, addSlot(chain, s));
  }

  function choosePreset(name: string) {
    const p = [...userPresets, ...BUILTIN_PRESETS].find((x) => x.name === name);
    if (!p) return;
    const next = applyPreset(chain, p, known, () => crypto.randomUUID());
    selectedSlot = next?.slots[addsOne(p) ? next.slots.length - 1 : 0]?.id ?? null;
    commit(`Preset ${p.name}`, next);
  }

  async function savePreset() {
    const name = (await ask("Save Voice Chain Preset", "Preset name", ""))?.trim();
    if (!name) return;
    if (BUILTIN_PRESETS.some((p) => p.name === name)) {
      onNotice(`"${name}" is a built-in preset. Pick another name.`);
      return;
    }
    userPresets = [...userPresets.filter((p) => p.name !== name), presetFrom(name, chain)].sort((a, b) => a.name.localeCompare(b.name));
    prefs.set(KEY_PRESETS, $state.snapshot(userPresets));
    onNotice(`Saved preset ${name}.`);
  }

  function deletePreset(name: string) {
    userPresets = userPresets.filter((p) => p.name !== name);
    prefs.set(KEY_PRESETS, $state.snapshot(userPresets));
    onNotice(`Deleted preset ${name}.`);
  }

  // Sliders show the value while dragging and apply as they move, a
  // few times a second, so playback follows. Consecutive changes to one
  // control merge into one undo step. Controls that change an effect's
  // look-ahead would restart playback each time, so they apply on release.
  let draft = $state<{ key: string; value: number } | null>(null);
  const LIVE_MS = 90;
  let liveTimer: ReturnType<typeof setTimeout> | undefined;
  let liveNext: (() => void) | null = null;
  function live(fn: () => void) {
    liveNext = fn;
    if (liveTimer) return;
    liveTimer = setTimeout(() => {
      liveTimer = undefined;
      const f = liveNext;
      liveNext = null;
      f?.();
    }, LIVE_MS);
  }
  function settle() {
    clearTimeout(liveTimer);
    liveTimer = undefined;
    liveNext = null;
  }
  $effect(() => () => clearTimeout(liveTimer));
  const paramLabel = (s: EffectSlotDoc, p: EffectParamInfo) => `${byId.get(s.effect)?.name ?? "Effect"} ${p.name}`;
  function shown(s: EffectSlotDoc, p: EffectParamInfo): number {
    return draft && draft.key === `${s.id}.${p.key}` ? draft.value : paramValue(s, p);
  }
  function drag(s: EffectSlotDoc, p: EffectParamInfo, v: number) {
    draft = { key: `${s.id}.${p.key}`, value: v };
    if (p.retimes) return;
    const id = s.id;
    live(() => {
      const cur = chain.slots.find((x) => x.id === id);
      if (cur && paramValue(cur, p) !== v) commit(paramLabel(cur, p), setParam(chain, id, p, v));
    });
  }
  function setValue(s: EffectSlotDoc, p: EffectParamInfo, v: number) {
    settle();
    draft = null;
    const cur = chain.slots.find((x) => x.id === s.id) ?? s;
    if (v === paramValue(cur, p)) return;
    commit(paramLabel(cur, p), setParam(chain, s.id, p, v));
  }
  /** A Curve EQ point dragged on the graph: frequency and gain together. */
  function dragPoint(s: EffectSlotDoc, pf: EffectParamInfo, f: number, pg: EffectParamInfo, g: number, done: boolean) {
    const id = s.id;
    const apply = () => {
      const next = setParam(setParam(chain, id, pf, f), id, pg, g);
      commit(`${paramLabel(s, pf).replace(pf.name, "").trim() || "Curve"} Point`, next);
    };
    if (done) {
      settle();
      apply();
    } else live(apply);
  }

  /** Input, output and mix: the same, on the chain itself. */
  function dragChain(key: "input_gain_db" | "output_gain_db" | "mix", label: string, shownValue: number, value: number) {
    draft = { key: `chain.${key}`, value: shownValue };
    live(() => {
      if (chain[key] !== value) commit(label, { ...chain, [key]: value });
    });
  }
  function setChain(key: "input_gain_db" | "output_gain_db" | "mix", label: string, value: number) {
    settle();
    draft = null;
    if (chain[key] !== value) commit(label, { ...chain, [key]: value });
  }

  // Noise Print Reducer: learn the print from the time selection.
  const LEARNS = new Set(["altaudio.noise_print", "altaudio.sound_eraser"]);
  const learnSel = $derived.by(() => {
    const v = inWave ? waveDoc?.view.selection : doc?.view.selection;
    return v && v.end > v.start ? v : null;
  });
  let learning = $state(false);
  async function learnPrint(s: EffectSlotDoc) {
    if (!learnSel || !target || learning) return;
    const source: LearnSource =
      target.kind === "master" || inWave
        ? { kind: "wave", start: learnSel.start, end: learnSel.end }
        : { kind: target.kind, id: target.id, start: learnSel.start, end: learnSel.end };
    learning = true;
    try {
      const r = await learnNoisePrint(source);
      const cur = chain.slots.find((x) => x.id === s.id);
      if (!cur) return;
      commit("Learn Noise Print", { ...chain, slots: chain.slots.map((x) => (x.id === s.id ? { ...x, params: { ...x.params, ...r.params }, preset: null } : x)) });
      onNotice(`Learned ${s.effect === "altaudio.sound_eraser" ? "the sound" : "the noise"} from ${r.seconds.toFixed(2)} s.`);
    } catch (e) {
      onError(e);
    } finally {
      learning = false;
    }
  }

  // Meters: the track chain's level in and out, as playback hears it.
  let ts = $state<TransportState | null>(null);
  const tp = $derived(inWave ? waveTransport : transport);
  $effect(() =>
    tp?.subscribe((x) => {
      ts = x;
    }),
  );
  const meterTrack = $derived(inWave ? waveTrack : tab === "track" ? track : clipTrack);
  const chainMeter = $derived(meterTrack ? ts?.chains[meterTrack.id] : undefined);
  const louder = (a?: MeterLevel, b?: MeterLevel) => (a && b ? (a.peak >= b.peak ? a : b) : (a ?? b));
  const dbText = (m: MeterLevel | undefined) => (m && m.peak > 0.00001 ? (20 * Math.log10(m.peak)).toFixed(1) : "-inf");

  const GAIN = (key: string, name: string): EffectParamInfo => ({ key, name, unit: "dB", min: -24, max: 24, default: 0, step: 0.1, log: false, toggle: false });
  const mixShown = $derived(draft?.key === "chain.mix" ? draft.value : Math.round(chain.mix * 100));
  const MIX: EffectParamInfo = { key: "mix", name: "Mix", unit: "%", min: 0, max: 100, default: 100, step: 1, log: false, toggle: false };

  const groups = (info: EffectInfo) => {
    const out: { name: string; params: EffectParamInfo[] }[] = [];
    for (const p of info.params) {
      if (p.hidden) continue;
      const g = p.group ?? "";
      const last = out[out.length - 1];
      if (last && last.name === g) last.params.push(p);
      else out.push({ name: g, params: [p] });
    }
    return out;
  };
</script>

<section class="vc" aria-label="Voice Chain">
  <div class="tabs" role="tablist" hidden={inWave}>
    <button type="button" role="tab" aria-selected={tab === "clip"} class:on={tab === "clip"} onclick={() => setTab("clip")}>Clip Effects</button>
    <button type="button" role="tab" aria-selected={tab === "track"} class:on={tab === "track"} onclick={() => setTab("track")}>Track Effects</button>
  </div>

  {#if loadError}
    <p class="empty">Effects could not load: {loadError}</p>
  {:else if !target}
    <p class="empty">{why}</p>
  {:else}
    <div class="top">
      <div class="line">
        <button
          type="button"
          class="power"
          class:on={chain.enabled}
          title={chain.enabled ? "Chain on: click to bypass every effect" : "Chain bypassed: click to turn it on"}
          aria-pressed={chain.enabled}
          onclick={() => commit(chain.enabled ? "Bypass Voice Chain" : "Turn On Voice Chain", { ...chain, enabled: !chain.enabled })}
        >
          <svg viewBox="0 0 16 16" aria-hidden="true"><path d="M8 2v6" stroke="currentColor" stroke-width="1.6" /><path d="M4.6 4.4a5 5 0 1 0 6.8 0" fill="none" stroke="currentColor" stroke-width="1.4" /></svg>
        </button>
        <select
          class="grow"
          aria-label="Presets"
          value=""
          onchange={(e) => {
            choosePreset(e.currentTarget.value);
            e.currentTarget.value = "";
          }}
        >
          <option value="" disabled>Presets…</option>
          {#if userPresets.length}
            <optgroup label="Yours">
              {#each userPresets as p (p.name)}<option value={p.name}>{p.name}</option>{/each}
            </optgroup>
          {/if}
          {#each ["Voice chains", "Breath gates", "Processing", "Repair", "EQ"] as g (g)}
            <optgroup label={g === "Voice chains" ? "Voice chains (replace the chain)" : `${g} (add one effect)`}>
              {#each BUILTIN_PRESETS.filter((p) => p.group === g) as p (p.name)}<option value={p.name}>{p.name}</option>{/each}
            </optgroup>
          {/each}
        </select>
        <button type="button" title="Save this chain as a preset" disabled={chain.slots.length === 0} onclick={() => void savePreset()}>Save…</button>
        {#if userPresets.length}
          <select
            class="del"
            aria-label="Delete a saved preset"
            title="Delete a saved preset"
            value=""
            onchange={(e) => {
              deletePreset(e.currentTarget.value);
              e.currentTarget.value = "";
            }}
          >
            <option value="" disabled>Delete…</option>
            {#each userPresets as p (p.name)}<option value={p.name}>{p.name}</option>{/each}
          </select>
        {/if}
      </div>
      {#if !inWave && tab === "track" && doc && doc.tracks.length > 1}
        <label class="line target">
          <span>Track</span>
          <select class="grow" aria-label="Track" value={track?.id ?? ""} onchange={(e) => (pickedTrack = e.currentTarget.value)}>
            {#each doc.tracks as t (t.id)}<option value={t.id}>{t.name}{(t.effects?.slots.length ?? 0) > 0 ? ` (${t.effects!.slots.length})` : ""}</option>{/each}
          </select>
        </label>
      {:else}
        <div class="target" title={label}>{label}</div>
      {/if}
    </div>

    <ol class="slots" class:off={!chain.enabled}>
      {#each Array.from({ length: MAX_SLOTS }, (_, i) => i) as i (i)}
        {@const s = chain.slots[i]}
        {@const info = s ? byId.get(s.effect) : undefined}
        <li class:sel={!!s && s.id === slot?.id} class:bypassed={!!s && !s.enabled}>
          <span class="n">{i + 1}</span>
          {#if s}
            <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
            <span class="name" onclick={() => (selectedSlot = s.id)} title={info?.description ?? "Not a built-in effect: kept in the file, skipped on playback."}>
              <span class="badge" style:background={info ? (CATEGORY_COLORS[info.category] ?? "#888") : "#666"}></span>
              {info?.name ?? s.effect}
            </span>
            <button type="button" class="mini" class:on={s.enabled} title={s.enabled ? "Bypass" : "Turn on"} aria-pressed={s.enabled} onclick={() => commit(s.enabled ? "Bypass Effect" : "Turn On Effect", setSlot(chain, s.id, { enabled: !s.enabled }))}>
              {s.enabled ? "On" : "Off"}
            </button>
            <button type="button" class="mini" title="Move up" aria-label="Move up" disabled={i === 0} onclick={() => commit("Move Effect", moveSlot(chain, s.id, -1))}>▲</button>
            <button type="button" class="mini" title="Move down" aria-label="Move down" disabled={i === chain.slots.length - 1} onclick={() => commit("Move Effect", moveSlot(chain, s.id, 1))}>▼</button>
            <button type="button" class="mini" title="Remove" aria-label="Remove" onclick={() => commit(`Remove ${info?.name ?? "Effect"}`, removeSlot(chain, s.id))}>✕</button>
          {:else if i === chain.slots.length}
            <select
              class="add"
              aria-label="Add an effect"
              value=""
              onchange={(e) => {
                add(e.currentTarget.value);
                e.currentTarget.value = "";
              }}
            >
              <option value="" disabled>+ Add effect</option>
              {#each categories as c (c)}
                <optgroup label={c}>
                  {#each catalog.filter((e) => e.category === c) as e (e.id)}<option value={e.id}>{e.name}</option>{/each}
                </optgroup>
              {/each}
            </select>
          {:else}
            <span class="name blank"></span>
          {/if}
        </li>
      {/each}
    </ol>

    <div class="editor">
      {#if slot && slotInfo}
        <h2>{slotInfo.name}{slot.preset ? ` · ${slot.preset}` : ""}</h2>
        {#if GRAPHED.has(slot.effect)}
          <ResponseGraph {slot} info={slotInfo} onPoint={(pf, f, pg, g, done) => dragPoint(slot, pf, f, pg, g, done)} />
        {/if}
        {#if LEARNS.has(slot.effect)}
          {@const len = slot.params.printlen ?? 0}
          {@const what = slot.effect === "altaudio.sound_eraser" ? "sound" : "noise"}
          <div class="learn">
            <button type="button" disabled={!learnSel || learning} title={learnSel ? `Learn the ${what} in the time selection` : `Select a stretch with only the ${what} in it first`} onclick={() => void learnPrint(slot)}>
              {learning ? "Learning…" : "Learn from Selection"}
            </button>
            <span class="hint">
              {#if len > 0}{what === "sound" ? "Sound" : "Noise"} learned from {len.toFixed(2)} s.{:else if what === "sound"}Nothing learned yet: select the sound to remove, as clean as you can find it, then Learn. Until then it changes nothing.{:else}Nothing learned yet: select a pause with only the noise, then Learn. Until then it changes nothing.{/if}
            </span>
          </div>
        {/if}
        {#each groups(slotInfo) as g (g.name)}
          {#if g.name}<h3>{g.name}</h3>{/if}
          {#each g.params as p (p.key)}
            {@const v = shown(slot, p)}
            <label class="param">
              <span class="pname">{p.name}</span>
              {#if p.toggle}
                <input type="checkbox" checked={v >= 0.5} onchange={(e) => setValue(slot, p, e.currentTarget.checked ? 1 : 0)} />
              {:else if p.choices?.length}
                <select value={String(Math.round(v))} onchange={(e) => setValue(slot, p, Number(e.currentTarget.value))}>
                  {#each p.choices as c, ci (c)}<option value={String(p.min + ci)}>{c}</option>{/each}
                </select>
              {:else}
                <input
                  type="range"
                  min="0"
                  max="1"
                  step="0.001"
                  value={toSlider(p, v)}
                  oninput={(e) => drag(slot, p, fromSlider(p, Number(e.currentTarget.value)))}
                  onchange={(e) => setValue(slot, p, fromSlider(p, Number(e.currentTarget.value)))}
                  ondblclick={() => setValue(slot, p, p.default)}
                  title="Double-click for the default"
                />
              {/if}
              <span class="val">{formatParam(p, v)}</span>
            </label>
          {/each}
        {/each}
      {:else if slot}
        <p class="hint">This effect is not built in. It stays in the file and is skipped on playback.</p>
      {:else}
        <p class="hint">Add an effect or pick a preset. Effects run top to bottom.</p>
      {/if}
    </div>

    <div class="bottom">
      {#each [GAIN("input_gain_db", "Input"), GAIN("output_gain_db", "Output")] as p (p.key)}
        {@const cur = draft?.key === `chain.${p.key}` ? draft.value : chain[p.key as "input_gain_db" | "output_gain_db"]}
        <label class="param">
          <span class="pname">{p.name}</span>
          <input
            type="range"
            min="0"
            max="1"
            step="0.001"
            value={toSlider(p, cur)}
            oninput={(e) => {
              const v = fromSlider(p, Number(e.currentTarget.value));
              dragChain(p.key as "input_gain_db", `Voice Chain ${p.name} Gain`, v, v);
            }}
            onchange={(e) => setChain(p.key as "input_gain_db", `Voice Chain ${p.name} Gain`, fromSlider(p, Number(e.currentTarget.value)))}
            ondblclick={() => setChain(p.key as "input_gain_db", `Voice Chain ${p.name} Gain`, 0)}
            title="Double-click for 0 dB"
          />
          <span class="val">{formatParam(p, cur)}</span>
        </label>
      {/each}
      <label class="param">
          <span class="pname" title="Dry/Wet: how much of the effected sound you hear">Mix</span>
          <input
            type="range"
            min="0"
            max="100"
            step="1"
            value={mixShown}
            oninput={(e) => dragChain("mix", "Voice Chain Mix", Number(e.currentTarget.value), Number(e.currentTarget.value) / 100)}
            onchange={(e) => setChain("mix", "Voice Chain Mix", Number(e.currentTarget.value) / 100)}
          />
          <span class="val">{formatParam(MIX, mixShown)}</span>
        </label>
      <div class="meters" aria-label="Track chain meters">
        {#each [["In", 0], ["Out", 2]] as const as [name, k] (name)}
          <div class="mrow">
            <span class="pname">{name}</span>
            <div class="mbars">
              {#each [k, k + 1] as ch (ch)}
                {@const m = chainMeter?.[ch]}
                <div class="mbar" role="meter" aria-label="{name} {ch % 2 === 0 ? 'left' : 'right'}" aria-valuemin={-60} aria-valuemax={0} aria-valuenow={m && m.peak > 0 ? Math.max(-60, Math.round(20 * Math.log10(m.peak))) : -60}>
                  <div class="mcover" style:width="{(1 - meterFraction(m?.level ?? 0)) * 100}%"></div>
                  {#if m && m.peak > 0}<div class="mpeak" style:left="{meterFraction(m.peak) * 100}%"></div>{/if}
                </div>
              {/each}
            </div>
            <span class="val">{dbText(louder(chainMeter?.[k], chainMeter?.[k + 1]))}</span>
          </div>
        {/each}
        <p class="hint">
          {#if !meterTrack}Meters show a track's chain while playing.
          {:else if tab === "clip" && !inWave}Meters show the track chain of {meterTrack.name}. Clip chains are not metered yet.
          {:else if !ts?.playing}Meters move while playing.
          {:else if !chainMeter}This track's chain does nothing yet, so there is nothing to meter.
          {:else}Track chain level going in and coming out.{/if}
        </p>
      </div>
      {#if inWave}
        <div class="apply">
          <p class="hint">You hear this chain when you play the file. The file changes only when you apply it, and Undo takes it back.</p>
          <div class="line">
            <button type="button" class="primary" disabled={!onApplyWave || applying || !chain.enabled || chain.slots.length === 0} onclick={() => void applyWave()}>
              {applying ? "Applying…" : waveSel ? "Apply to Selection" : "Apply to Whole File"}
            </button>
            <button type="button" disabled={applying || chain.slots.length === 0} onclick={() => commit("Clear", chainOf(null))}>Clear Preview</button>
          </div>
        </div>
      {/if}
    </div>
  {/if}
</section>

<style>
  .vc {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    overflow: auto;
    background: var(--surface-1);
    font-size: 12px;
  }
  .tabs {
    display: flex;
    border-bottom: 1px solid var(--border);
  }
  .tabs button {
    flex: 1;
    padding: 4px 6px;
    background: none;
    border: 0;
    border-bottom: 2px solid transparent;
    color: var(--text-dim);
    font: inherit;
  }
  .tabs button.on {
    color: var(--text);
    border-bottom-color: var(--accent);
  }
  .top {
    display: grid;
    gap: 4px;
    padding: 6px 8px;
    border-bottom: 1px solid var(--border);
  }
  .line {
    display: flex;
    gap: 4px;
    align-items: center;
  }
  .grow {
    flex: 1;
    min-width: 0;
  }
  .del {
    width: 70px;
  }
  .target {
    overflow: hidden;
    color: var(--text-dim);
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  select,
  button {
    background: var(--surface-0);
    border: 1px solid var(--border-strong);
    border-radius: 2px;
    color: var(--text);
    font: inherit;
    padding: 1px 4px;
  }
  button:disabled {
    opacity: 0.45;
  }
  .power {
    display: grid;
    width: 24px;
    height: 22px;
    padding: 0;
    place-items: center;
    color: var(--text-faint);
  }
  .power.on {
    color: var(--accent);
  }
  .power svg {
    width: 14px;
    height: 14px;
  }
  .slots {
    margin: 0;
    padding: 4px 0;
    list-style: none;
    border-bottom: 1px solid var(--border);
  }
  .slots.off {
    opacity: 0.55;
  }
  .slots li {
    display: flex;
    gap: 3px;
    align-items: center;
    height: 22px;
    padding: 0 8px;
  }
  .slots li.sel {
    background: var(--surface-2);
  }
  .slots li.bypassed .name {
    color: var(--text-faint);
    text-decoration: line-through;
  }
  .n {
    width: 16px;
    color: var(--text-faint);
    font-variant-numeric: tabular-nums;
    text-align: right;
  }
  .name {
    display: flex;
    flex: 1;
    gap: 6px;
    min-width: 0;
    overflow: hidden;
    align-items: center;
    cursor: pointer;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .name.blank {
    height: 1px;
    border-bottom: 1px dotted var(--border);
    cursor: default;
  }
  .badge {
    flex: none;
    width: 8px;
    height: 8px;
    border-radius: 2px;
  }
  .add {
    flex: 1;
    color: var(--text-dim);
  }
  .mini {
    min-width: 20px;
    padding: 0 3px;
    font-size: 10px;
    line-height: 16px;
  }
  .mini.on {
    color: var(--accent);
  }
  .editor,
  .bottom {
    display: grid;
    gap: 3px;
    padding: 6px 8px;
  }
  .bottom {
    border-top: 1px solid var(--border);
  }
  h2,
  h3 {
    margin: 4px 0 0;
    font-size: 12px;
  }
  h3 {
    color: var(--text-dim);
    font-weight: normal;
  }
  .param {
    display: grid;
    grid-template-columns: 88px minmax(0, 1fr) 72px;
    gap: 6px;
    align-items: center;
  }
  .pname {
    overflow: hidden;
    color: var(--text-dim);
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .val {
    font-variant-numeric: tabular-nums;
    text-align: right;
  }
  input[type="range"] {
    width: 100%;
  }
  .empty,
  .hint {
    margin: 0;
    color: var(--text-faint);
  }
  .meters {
    display: grid;
    gap: 3px;
  }
  .learn {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    align-items: center;
  }
  .tabs[hidden] {
    display: none;
  }
  .apply {
    display: grid;
    gap: 4px;
    padding-top: 4px;
    border-top: 1px solid var(--border);
  }
  .apply .primary {
    border-color: var(--accent);
    background: var(--accent);
    color: var(--on-accent, #fff);
  }
  .mrow {
    display: grid;
    grid-template-columns: 88px minmax(0, 1fr) 72px;
    gap: 6px;
    align-items: center;
  }
  .mbars {
    display: grid;
    gap: 2px;
  }
  .mbar {
    position: relative;
    height: 5px;
    overflow: hidden;
    background: linear-gradient(to right, #3d8f6a 0 var(--meter-yellow, 80%), #c9b33f var(--meter-yellow, 80%) var(--meter-red, 95%), #e5534b var(--meter-red, 95%) 100%);
  }
  .mcover {
    position: absolute;
    top: 0;
    right: 0;
    height: 100%;
    background: var(--surface-0);
  }
  .mpeak {
    position: absolute;
    top: 0;
    width: 1px;
    height: 100%;
    background: var(--text);
  }
  .empty {
    padding: 12px 8px;
  }
</style>
