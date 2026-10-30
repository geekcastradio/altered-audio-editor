<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  The Dynamics window: a graph of output level against input level that
  shows what the settings do, with two handles to drag (the threshold,
  and the ratio at 0 dB in), and sliders for every setting underneath.
  Built-in presets are in the Presets menu above.
-->
<script lang="ts">
  import type { EffectInfo, EffectParamInfo } from "../../app/core";
  import type { EffectSlotDoc } from "../../stores/session";
  import { paramValue } from "./chain";
  import { curvePoints, ratioFromSlope, transfer } from "./dynamics";

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

  const SIZE = 200;
  const LO = -60;
  const HI = 0;
  const curve = $derived({ threshold: val("threshold"), ratio: val("ratio"), knee: val("knee"), makeup: val("makeup") });
  const x = (db: number) => ((db - LO) / (HI - LO)) * SIZE;
  const y = (db: number) => SIZE - ((Math.max(LO, Math.min(HI + 24, db)) - LO) / (HI + 24 - LO)) * SIZE;
  const OUT_HI = HI + 24;
  const inFrom = (px: number, b: DOMRect) => LO + ((px - b.left) / b.width) * (HI - LO);
  const outFrom = (py: number, b: DOMRect) => OUT_HI - ((py - b.top) / b.height) * (OUT_HI - LO);
  const path = $derived(
    curvePoints(curve)
      .map(([i, o], n) => `${n ? "L" : "M"}${x(i).toFixed(1)},${(((OUT_HI - Math.max(LO, Math.min(OUT_HI, o))) / (OUT_HI - LO)) * SIZE).toFixed(1)}`)
      .join(" "),
  );
  const oy = (db: number) => ((OUT_HI - Math.max(LO, Math.min(OUT_HI, db))) / (OUT_HI - LO)) * SIZE;
  const tY = $derived(oy(transfer(curve.threshold, curve)));
  const rY = $derived(oy(transfer(0, curve)));

  let svg = $state<SVGSVGElement | null>(null);
  function drag(e: PointerEvent, which: "threshold" | "ratio") {
    if (!svg) return;
    (e.currentTarget as Element).setPointerCapture(e.pointerId);
    const move = (ev: PointerEvent, done: boolean) => {
      const b = svg!.getBoundingClientRect();
      if (which === "threshold") set("threshold", Math.round(inFrom(ev.clientX, b) * 10) / 10, done);
      else {
        // Ratio from where the handle at 0 dB in sits above the threshold.
        const over = 0 - curve.threshold;
        const out = outFrom(ev.clientY, b) - curve.makeup;
        const slope = over > 0.5 ? (out - curve.threshold) / over : 1;
        set("ratio", Math.round(ratioFromSlope(slope, P("ratio").min, P("ratio").max) * 10) / 10, done);
      }
    };
    const m = (ev: PointerEvent) => move(ev, false);
    const up = (ev: PointerEvent) => {
      window.removeEventListener("pointermove", m);
      window.removeEventListener("pointerup", up);
      move(ev, true);
    };
    window.addEventListener("pointermove", m);
    window.addEventListener("pointerup", up);
  }
  const grid = [-50, -40, -30, -20, -10];
  const fmt = (v: number, step: number) => (step < 1 ? v.toFixed(1) : String(Math.round(v)));
</script>

{#snippet slider(k: string, label: string, unit: string)}
  {@const p = P(k)}
  {@const v = val(k)}
  <label class="srow">
    <span class="lab">{label}</span>
    <input
      type="range"
      min={p.min}
      max={p.max}
      step={p.step}
      value={p.log ? Math.log(v) : v}
      aria-label={label}
      oninput={(e) => set(k, p.log ? Math.exp(Number(e.currentTarget.value)) : Number(e.currentTarget.value), false)}
      onchange={(e) => set(k, p.log ? Math.exp(Number(e.currentTarget.value)) : Number(e.currentTarget.value), true)}
      ondblclick={() => set(k, p.default, true)}
    />
    <span class="num">{fmt(v, p.step)}{unit}</span>
  </label>
{/snippet}

<div class="dyn">
  <svg bind:this={svg} class="graph" viewBox="0 0 {SIZE} {SIZE}" role="img" aria-label="Input level against output level">
    {#each grid as g (g)}
      <line class="grid" x1={x(g)} x2={x(g)} y1="0" y2={SIZE} />
      <line class="grid" x1="0" x2={SIZE} y1={oy(g)} y2={oy(g)} />
    {/each}
    <line class="unity" x1="0" y1={oy(LO)} x2={SIZE} y2={oy(HI)} />
    <path class="curve" d={path} />
    <circle class="dot" cx={x(curve.threshold)} cy={tY} r="5" role="slider" tabindex="-1" aria-label="Threshold" aria-valuenow={curve.threshold} onpointerdown={(e) => drag(e, "threshold")} />
    <circle class="dot" cx={x(0)} cy={rY} r="5" role="slider" tabindex="-1" aria-label="Ratio" aria-valuenow={curve.ratio} onpointerdown={(e) => drag(e, "ratio")} />
  </svg>
  <div class="sliders">
    {@render slider("threshold", "Threshold", " dB")}
    {@render slider("ratio", "Ratio", ":1")}
    {@render slider("attack", "Attack", " ms")}
    {@render slider("release", "Release", " ms")}
    {@render slider("knee", "Knee", " dB")}
    {@render slider("makeup", "Output gain", " dB")}
    <label class="check"><input type="checkbox" checked={Math.round(val("linked")) === 1} onchange={(e) => set("linked", e.currentTarget.checked ? 1 : 0, true)} /> Link channels</label>
  </div>
</div>

<style>
  .dyn {
    display: grid;
    grid-template-columns: minmax(120px, 200px) 1fr;
    gap: 12px;
    align-items: start;
  }
  .graph {
    width: 100%;
    aspect-ratio: 1;
    border: 1px solid var(--border);
    background: var(--surface-0);
    touch-action: none;
  }
  .grid {
    stroke: var(--border);
    stroke-width: 0.5;
  }
  .unity {
    stroke: var(--text-faint, #777);
    stroke-dasharray: 3 3;
    stroke-width: 0.7;
  }
  .curve {
    fill: none;
    stroke: var(--accent);
    stroke-width: 1.6;
  }
  .dot {
    fill: var(--surface-3);
    stroke: var(--accent);
    stroke-width: 1.5;
    cursor: grab;
  }
  .sliders {
    display: grid;
    gap: 4px;
  }
  .srow {
    display: grid;
    grid-template-columns: 5.5em 1fr 5em;
    gap: 6px;
    align-items: center;
  }
  .lab {
    color: var(--text-dim);
  }
  .num {
    text-align: right;
    font-variant-numeric: tabular-nums;
  }
  .check {
    display: flex;
    gap: 6px;
    align-items: center;
  }
</style>
