<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The Audiform contributors.
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor, adapted from Audiform.

  The Vox Strip window: four modules side by side in signal order
  (Dynamics, Tone, Effects, Space), each with a power light, a style
  stepper and its one or two main controls, over an Input and Output
  bar. Flat, in the app's own colors, like the rest of the Voice Chain.
-->
<script lang="ts">
  import { effectMeter, voxToneCurve, type EffectInfo, type EffectParamInfo } from "../../app/core";
  import type { EffectSlotDoc } from "../../stores/session";
  import { formatParam, paramValue } from "./chain";
  import BigKnob from "./BigKnob.svelte";

  interface Props {
    slot: EffectSlotDoc;
    info: EffectInfo;
    /** The session tempo, for the Space module's delay. */
    tempo?: number | undefined;
    onParams: (changes: [EffectParamInfo, number][], label: string, done: boolean) => void;
  }

  const { slot, info, tempo, onParams }: Props = $props();

  const byKey = $derived(new Map(info.params.map((p) => [p.key, p])));
  const P = (k: string) => byKey.get(k)!;
  const val = (k: string) => paramValue(slot, P(k));
  const set = (k: string, v: number, done: boolean) => {
    const p = P(k);
    onParams([[p, Math.max(p.min, Math.min(p.max, v))]], `${info.name} ${p.name}`, done);
  };
  const on = (k: string) => val(k) >= 0.5;
  const choicesOf = (k: string): string[] => P(k).choices ?? [];

  interface Module {
    key: string;
    name: string;
    power: string;
    style: string;
    trim: string;
  }
  const MODULES: Module[] = [
    { key: "dyn", name: "Dynamics", power: "dyn_on", style: "dyn_style", trim: "dyn_trim" },
    { key: "tone", name: "Tone", power: "tone_on", style: "tone_style", trim: "tone_trim" },
    { key: "fx", name: "Effects", power: "fx_on", style: "fx_style", trim: "fx_trim" },
    { key: "space", name: "Space", power: "space_on", style: "space_style", trim: "space_trim" },
  ];

  function step(m: Module, d: number) {
    const n = choicesOf(m.style).length;
    set(m.style, (((Math.round(val(m.style)) + d) % n) + n) % n, true);
  }

  // Meters: gain reduction, output and de-essing, read about 30 times a second.
  let gr = $state(0);
  let ds = $state(0);
  let out = $state(-120);
  $effect(() => {
    const id = slot.id;
    let alive = true;
    let busy = false;
    const timer = setInterval(() => {
      if (busy || !alive) return;
      busy = true;
      effectMeter(id)
        .then((m) => {
          if (!m || !alive) return;
          gr = m[0];
          out = m[1];
          ds = m[2] ?? 0;
        })
        .catch(() => {})
        .finally(() => (busy = false));
    }, 33);
    return () => {
      alive = false;
      clearInterval(timer);
    };
  });

  // The Tone curve, from the core so it matches what is heard.
  let curve = $state<[number, number][]>([]);
  $effect(() => {
    const s = $state.snapshot(slot) as EffectSlotDoc;
    let alive = true;
    voxToneCurve(s, 96)
      .then((c) => alive && (curve = c))
      .catch(() => {});
    return () => {
      alive = false;
    };
  });
  const CW = 200;
  const CH = 64;
  const curvePath = $derived.by(() => {
    if (curve.length < 2) return "";
    const x = (f: number) => (Math.log10(f / 20) / 3) * CW;
    const y = (db: number) => CH / 2 - (Math.max(-15, Math.min(15, db)) / 15) * (CH / 2 - 4);
    return curve.map(([f, db], i) => `${i ? "L" : "M"}${x(f).toFixed(1)},${y(db).toFixed(1)}`).join(" ");
  });

  /** A bar meter's fill, 0 to 1, for 0 to 24 dB of reduction. */
  const fill = (db: number) => Math.max(0, Math.min(1, db / 24));
  const outFill = (db: number) => Math.max(0, Math.min(1, (db + 60) / 60));

  const TONE_SLIDERS = ["low", "mid", "high"];
</script>

{#snippet knob(k: string, size: number, label?: string)}
  {@const p = P(k)}
  <BigKnob param={p} value={val(k)} {size} {label} oninput={(v) => set(k, v, false)} oncommit={(v) => set(k, v, true)} />
{/snippet}

{#snippet slider(k: string, air = false)}
  {@const p = P(k)}
  <label class="vs" class:air title="{p.name}: {formatParam(p, val(k))}. Double-click for the default.">
    <input
      type="range"
      min={p.min}
      max={p.max}
      step={p.step}
      value={val(k)}
      aria-label={p.name}
      oninput={(e) => set(k, Number(e.currentTarget.value), false)}
      onchange={(e) => set(k, Number(e.currentTarget.value), true)}
      ondblclick={() => set(k, p.default, true)}
    />
    <span class="vname">{p.name}</span>
    <span class="vval">{formatParam(p, val(k))}</span>
  </label>
{/snippet}

<div class="vox">
  <div class="tools">
    <span class="tag">Dynamics, Tone, Effects and Space, in signal order</span>
    <button type="button" class="lowcut" class:lit={on("low_cut")} aria-pressed={on("low_cut")} title="Low Cut: removes rumble below 80 Hz before everything else" onclick={() => set("low_cut", on("low_cut") ? 0 : 1, true)}>Low Cut</button>
  </div>

  <div class="modules">
    {#each MODULES as m (m.key)}
      {@const style = choicesOf(m.style)[Math.round(val(m.style))] ?? ""}
      {@const live = on(m.power)}
      <section class="mod" class:off={!live}>
        <header>
          <button type="button" class="led" class:lit={live} aria-pressed={live} title="{m.name}: {live ? 'on (click to bypass)' : 'bypassed (click to turn on)'}" onclick={() => set(m.power, live ? 0 : 1, true)}>
            <span></span>
          </button>
          <span class="mname">{m.name}</span>
        </header>
        <div class="stepper">
          <button type="button" aria-label="Previous {m.name} style" onclick={() => step(m, -1)}>‹</button>
          <div class="stylebox">
            <span>{style}</span>
            <span class="dots">
              {#each choicesOf(m.style) as c, i (c)}<i class:here={i === Math.round(val(m.style))}></i>{/each}
            </span>
          </div>
          <button type="button" aria-label="Next {m.name} style" onclick={() => step(m, 1)}>›</button>
        </div>

        <div class="body">
          {#if m.key === "dyn"}
            <div class="dyn">
              <div class="bars">
                <div class="bar" title="Gain reduction: {gr.toFixed(1)} dB"><i style:height="{fill(live ? gr : 0) * 100}%"></i><span>GR</span></div>
                <div class="bar ds" title="De-essing: {ds.toFixed(1)} dB"><i style:height="{fill(live ? ds : 0) * 100}%"></i><span>DS</span></div>
              </div>
              <div class="knobs">
                {@render knob("comp", 74)}
                {@render knob("deess", 74)}
              </div>
            </div>
          {:else if m.key === "tone"}
            <svg class="curve" viewBox="0 0 {CW} {CH}" preserveAspectRatio="none" aria-hidden="true">
              <line x1="0" x2={CW} y1={CH / 2} y2={CH / 2} />
              {#each [100, 1000, 10000] as f (f)}<line x1={(Math.log10(f / 20) / 3) * CW} x2={(Math.log10(f / 20) / 3) * CW} y1="0" y2={CH} />{/each}
              <path d={curvePath} />
            </svg>
            <div class="sliders">
              {#each TONE_SLIDERS as k (k)}{@render slider(k)}{/each}
              {@render slider("air", true)}
            </div>
          {:else if m.key === "fx"}
            <div class="one">{@render knob("fx", 112)}</div>
          {:else}
            <div class="knobs">
              {@render knob("reverb", 74)}
              {@render knob("delay", 74)}
            </div>
            <div class="sync">
              <select aria-label="Delay time" value={Math.round(val("note"))} onchange={(e) => set("note", Number(e.currentTarget.value), true)} disabled={style === "Slap"} title={style === "Slap" ? "Slap uses its own short echo" : "Delay time, as a note value at the tempo"}>
                {#each choicesOf("note") as c, i (c)}<option value={i}>{c}</option>{/each}
              </select>
              <span class="bpm">{Math.round(val("tempo"))} BPM</span>
              {#if tempo && Math.abs(tempo - val("tempo")) > 0.05}
                <button type="button" title="Use the session tempo ({tempo} BPM)" onclick={() => set("tempo", tempo, true)}>Sync</button>
              {/if}
            </div>
          {/if}
        </div>
        <footer>
          {@render knob(m.trim, 40, "Trim")}
        </footer>
      </section>
    {/each}
  </div>

  <div class="io">
    <div class="k">{@render knob("input", 44)}</div>
    <div class="ladder" title="Output level: {out.toFixed(1)} dBFS">
      {#each Array.from({ length: 30 }, (_, i) => i) as i (i)}<i class:on={outFill(out) * 30 > i} class:hot={i >= 27}></i>{/each}
    </div>
    <div class="k">{@render knob("output", 44)}</div>
  </div>
</div>

<style>
  .vox {
    --panel-ink: var(--text-dim);
    --panel-accent: var(--accent);
    --knob-skirt: var(--surface-1);
    --knob-edge: var(--border-strong);
    --knob-cap: var(--surface-3);
    --knob-cap-edge: var(--border-strong);
    --knob-grip: var(--surface-1);
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-bottom: 6px;
    color: var(--text);
    font-size: 12px;
    user-select: none;
  }
  .tools {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }
  .tag {
    color: var(--text-dim);
    font-size: 11px;
  }
  .lowcut {
    padding: 2px 10px;
    border: 1px solid var(--border-strong);
    border-radius: 3px;
    background: var(--surface-0);
    color: var(--text-dim);
    font: inherit;
  }
  .lowcut.lit {
    border-color: var(--accent);
    color: var(--accent);
  }
  .modules {
    display: grid;
    grid-template-columns: repeat(4, minmax(150px, 1fr));
    gap: 8px;
  }
  .mod {
    --mod: var(--accent);
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: 0;
    padding: 8px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--surface-1);
  }
  .mod.off {
    --mod: var(--text-faint);
  }
  .mod.off .stepper,
  .mod.off .body {
    opacity: 0.45;
  }
  .mod header {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .mname {
    font-weight: 600;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    font-size: 11px;
  }
  .led {
    display: grid;
    width: 16px;
    height: 16px;
    padding: 0;
    place-items: center;
    border: 1px solid var(--border-strong);
    border-radius: 50%;
    background: var(--surface-0);
  }
  .led span {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--text-faint);
  }
  .led.lit span {
    background: var(--accent);
  }
  .stepper {
    display: flex;
    align-items: stretch;
    gap: 3px;
  }
  .stepper > button {
    width: 18px;
    padding: 0;
    border: 1px solid var(--border-strong);
    border-radius: 3px;
    background: var(--surface-0);
    color: var(--text-dim);
    font-size: 14px;
  }
  .stepper > button:hover {
    color: var(--accent);
  }
  .stylebox {
    display: flex;
    flex: 1;
    flex-direction: column;
    align-items: center;
    gap: 3px;
    min-width: 0;
    padding: 6px 2px 5px;
    border: 1px solid var(--border);
    border-radius: 3px;
    background: var(--surface-0);
  }
  .stylebox > span:first-of-type {
    max-width: 100%;
    overflow: hidden;
    font-weight: 600;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .dots {
    display: flex;
    gap: 3px;
  }
  .dots i {
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: var(--border-strong);
  }
  .dots i.here {
    background: var(--accent);
  }
  .body {
    display: flex;
    flex: 1;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    min-height: 210px;
  }
  .dyn {
    display: flex;
    gap: 8px;
    align-items: stretch;
  }
  .bars {
    display: flex;
    gap: 5px;
  }
  .bar {
    position: relative;
    display: flex;
    flex-direction: column;
    justify-content: flex-start;
    width: 12px;
    height: 190px;
    border: 1px solid var(--border);
    border-radius: 2px;
    background: var(--surface-0);
    overflow: hidden;
  }
  .bar i {
    display: block;
    width: 100%;
    background: var(--accent);
  }
  .bar.ds i {
    background: var(--text-dim);
  }
  .bar span {
    position: absolute;
    bottom: 2px;
    left: 0;
    right: 0;
    text-align: center;
    font-size: 8px;
    color: var(--text-dim);
  }
  .knobs {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
  }
  .body > .knobs {
    flex-direction: row;
    flex-wrap: wrap;
    justify-content: center;
  }
  .one {
    display: grid;
    flex: 1;
    place-items: center;
  }
  .curve {
    width: 100%;
    height: 64px;
    border: 1px solid var(--border);
    border-radius: 3px;
    background: var(--surface-0);
  }
  .curve line {
    stroke: var(--border);
    stroke-width: 1;
  }
  .curve path {
    fill: none;
    stroke: var(--accent);
    stroke-width: 2;
    vector-effect: non-scaling-stroke;
  }
  .sliders {
    display: flex;
    justify-content: space-around;
    width: 100%;
  }
  .vs {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
  }
  .vs input {
    writing-mode: vertical-lr;
    direction: rtl;
    width: 22px;
    height: 104px;
    accent-color: var(--accent);
  }
  .vname {
    color: var(--text-dim);
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    font-size: 9px;
  }
  .vs.air .vname {
    color: var(--text);
  }
  .vval {
    color: var(--value);
    font-variant-numeric: tabular-nums;
    font-size: 9px;
  }
  .sync {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .sync select,
  .sync button {
    padding: 1px 4px;
    border: 1px solid var(--border-strong);
    border-radius: 2px;
    background: var(--surface-0);
    color: var(--text);
    font: inherit;
    font-size: 11px;
  }
  .bpm {
    color: var(--text-dim);
    font-variant-numeric: tabular-nums;
  }
  .mod footer {
    display: flex;
    justify-content: center;
    min-width: 0;
  }
  .mod footer :global(*),
  .io .k :global(*) {
    max-width: 100%;
    box-sizing: border-box;
  }
  .io .k {
    display: flex;
    justify-content: center;
    width: 84px;
    flex: none;
  }
  .io {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--surface-1);
  }
  .ladder {
    display: flex;
    flex: 1;
    gap: 2px;
    height: 10px;
  }
  .ladder i {
    flex: 1;
    border-radius: 1px;
    background: var(--surface-3);
  }
  .ladder i.on {
    background: var(--accent);
  }
  .ladder i.on.hot {
    background: var(--danger);
  }
</style>
