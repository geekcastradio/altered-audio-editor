<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  The Speech Leveler window, laid out like Audition's Speech Volume
  Leveler: sliders for the target level, how much leveling to apply and
  the dynamic range, a Boost Low Signals switch, and an Advanced section
  with a Compressor and a Noise Gate. Our own Most boost, Most cut,
  Response and Ignore below settings sit under Advanced too.
-->
<script lang="ts">
  import type { EffectInfo, EffectParamInfo } from "../../app/core";
  import type { EffectSlotDoc } from "../../stores/session";
  import { paramValue } from "./chain";

  interface Props {
    slot: EffectSlotDoc;
    info: EffectInfo;
    onParams: (changes: [EffectParamInfo, number][], label: string, done: boolean) => void;
  }

  const { slot, info, onParams }: Props = $props();

  const byKey = $derived(new Map(info.params.map((p) => [p.key, p])));
  const P = (k: string) => byKey.get(k)!;
  const val = (k: string) => paramValue(slot, P(k));
  const set = (k: string, v: number, done: boolean) => {
    const p = P(k);
    onParams([[p, Math.max(p.min, Math.min(p.max, v))]], `${info.name} ${p.name}`, done);
  };
  const fmt = (v: number, step = 1) => (step < 1 ? v.toFixed(1) : String(Math.round(v)));
  let advanced = $state(true);
</script>

{#snippet slider(k: string, label: string, unit: string, ticks: number[], extra = "", disabled = false)}
  {@const p = P(k)}
  {@const v = val(k)}
  <div class="srow" class:off={disabled}>
    <span class="lab">{label}</span>
    <div class="track">
      <input
        type="range"
        min={p.min}
        max={p.max}
        step={p.step}
        value={v}
        {disabled}
        aria-label={label}
        oninput={(e) => set(k, Number(e.currentTarget.value), false)}
        onchange={(e) => set(k, Number(e.currentTarget.value), true)}
        ondblclick={() => set(k, p.default, true)}
      />
      <div class="ticks" aria-hidden="true">{#each ticks as t (t)}<span style:left="{((t - p.min) / (p.max - p.min)) * 100}%">{t}</span>{/each}</div>
    </div>
    <span class="num">{fmt(v, p.step)}{unit}{extra}</span>
  </div>
{/snippet}

<div class="lev">
  {@render slider("target", "Target Volume Level", " dB", [-40, -30, -20, -10], " (RMS)")}
  {@render slider("amount", "Leveling Amount", " %", [0, 50, 100])}
  {@render slider("range", "Target Dynamic Range", " dB", [0, 20, 40, 60])}
  <label class="check">
    <input type="checkbox" checked={val("boost_low") >= 0.5} onchange={(e) => set("boost_low", e.currentTarget.checked ? 1 : 0, true)} />
    Boost Low Signals
  </label>

  <hr />
  <button type="button" class="adv" aria-expanded={advanced} onclick={() => (advanced = !advanced)}>{advanced ? "▾" : "▸"} Advanced</button>
  {#if advanced}
    <fieldset class:off={val("comp_on") < 0.5}>
      <legend>
        <label><input type="checkbox" checked={val("comp_on") >= 0.5} onchange={(e) => set("comp_on", e.currentTarget.checked ? 1 : 0, true)} /> Compressor</label>
      </legend>
      {@render slider("comp_thr", "Threshold", " dB", [-9, -5, -1], "", val("comp_on") < 0.5)}
    </fieldset>
    <fieldset class:off={val("gate_on") < 0.5}>
      <legend>
        <label><input type="checkbox" checked={val("gate_on") >= 0.5} onchange={(e) => set("gate_on", e.currentTarget.checked ? 1 : 0, true)} /> Noise Gate</label>
      </legend>
      {@render slider("gate_off", "Noise Offset", " dB", [0, 10, 25], "", val("gate_on") < 0.5)}
    </fieldset>
    <fieldset class="own">
      <legend>Limits and timing</legend>
      {@render slider("max_gain", "Most boost", " dB", [0, 15, 30])}
      {@render slider("max_cut", "Most cut", " dB", [0, 15, 30])}
      {@render slider("speed", "Response", " ms", [100, 1000, 10000])}
      {@render slider("floor", "Ignore below", " dB", [-80, -50, -20])}
    </fieldset>
  {/if}
</div>

<style>
  .lev {
    display: grid;
    gap: 6px;
    padding: 8px 10px;
  }
  .srow {
    display: grid;
    grid-template-columns: 8.5em minmax(0, 1fr) 7.2em;
    align-items: center;
    gap: 8px;
  }
  .srow.off {
    opacity: 0.45;
  }
  .lab {
    color: var(--text-dim);
    text-align: right;
  }
  .num {
    color: var(--accent);
    font-family: var(--font-mono);
    text-align: right;
    white-space: nowrap;
  }
  .track {
    position: relative;
    padding-bottom: 12px;
  }
  .track input {
    width: 100%;
    margin: 0;
    accent-color: var(--accent);
  }
  .ticks {
    position: absolute;
    left: 0;
    right: 0;
    bottom: 0;
    height: 11px;
    font-size: 9px;
    color: var(--text-faint);
  }
  .ticks span {
    position: absolute;
    transform: translateX(-50%);
  }
  .check {
    display: flex;
    gap: 6px;
    align-items: center;
    justify-content: center;
  }
  hr {
    width: 100%;
    margin: 2px 0;
    border: 0;
    border-top: 1px solid var(--border);
  }
  .adv {
    justify-self: start;
    border: 0;
    background: none;
    color: var(--accent);
    font: inherit;
  }
  fieldset {
    display: grid;
    gap: 4px;
    margin: 0;
    padding: 6px 8px 8px;
    border: 1px solid var(--border);
    border-radius: 3px;
  }
  fieldset.off legend {
    color: var(--text-dim);
  }
  legend label {
    display: flex;
    gap: 5px;
    align-items: center;
  }
</style>
