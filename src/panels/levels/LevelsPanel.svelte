<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Levels panel: the session's output level, left and right, on a dB
  scale from -60 to 0. Click a red over light to clear it.
-->
<script lang="ts">
  import { meterFraction, type MeterLevel, type TransportState, type TransportStore } from "../../stores/transport";

  interface Props {
    transport: TransportStore;
  }

  const { transport }: Props = $props();

  let s = $state<TransportState | null>(null);
  $effect(() =>
    transport.subscribe((v) => {
      s = v;
    }),
  );

  const TICKS = [-60, -48, -36, -24, -18, -12, -9, -6, -3, 0];
  const pos = (db: number) => ((db + 60) / 60) * 100;

  function db(m: MeterLevel | undefined): string {
    if (!m || m.peak <= 0.00001) return "-inf";
    const v = 20 * Math.log10(m.peak);
    return v > 0 ? `+${v.toFixed(1)}` : v.toFixed(1);
  }
</script>

<section class="levels" aria-label="Levels">
  {#each [0, 1] as const as ch (ch)}
    {@const m = s?.master[ch]}
    <div class="row">
      <span class="ch">{ch === 0 ? "L" : "R"}</span>
      <div
        class="bar"
        role="meter"
        aria-label={ch === 0 ? "Left level" : "Right level"}
        aria-valuemin={-60}
        aria-valuemax={0}
        aria-valuenow={m && m.peak > 0 ? Math.max(-60, Math.round(20 * Math.log10(m.peak))) : -60}
      >
        <div class="cover" style:width="{(1 - meterFraction(m?.level ?? 0)) * 100}%"></div>
        {#if m && m.peak > 0}<div class="peak" style:left="{meterFraction(m.peak) * 100}%"></div>{/if}
      </div>
      <button
        type="button"
        class="over"
        class:on={m?.over}
        title="Went over full scale. Click to clear."
        aria-label="Clear over light"
        onclick={() => transport.clearOver()}
      ></button>
      <span class="val">{db(m)}</span>
    </div>
  {/each}
  <div class="scale" aria-hidden="true">
    <span></span>
    <div class="ticks">
      {#each TICKS as t (t)}
        <span style:left="{pos(t)}%">{t}</span>
      {/each}
    </div>
  </div>
</section>

<style>
  .levels {
    display: grid;
    align-content: center;
    gap: 4px;
    height: 100%;
    padding: 6px 12px;
    box-sizing: border-box;
    background: var(--surface-1);
  }

  .row,
  .scale {
    display: grid;
    grid-template-columns: 14px 1fr 12px 44px;
    gap: 6px;
    align-items: center;
  }

  .ch {
    color: var(--text-dim);
    font-size: var(--font-size-small);
  }

  /* Green to -12 dB, yellow to -3 dB, red above. The cover hides the
     part of the scale above the current level. */
  .bar {
    position: relative;
    height: 9px;
    overflow: hidden;
    background: linear-gradient(to right, #3d8f6a 0 80%, #c9b33f 80% 95%, #e5534b 95% 100%);
  }

  .cover {
    position: absolute;
    top: 0;
    right: 0;
    height: 100%;
    background: var(--surface-0);
  }

  .peak {
    position: absolute;
    top: 0;
    width: 2px;
    height: 100%;
    background: var(--text);
    transform: translateX(-1px);
  }

  .over {
    width: 12px;
    height: 9px;
    padding: 0;
    border: 1px solid var(--border-strong);
    background: var(--surface-0);
  }

  .over.on {
    border-color: var(--danger);
    background: var(--danger);
  }

  .val {
    color: var(--value);
    font-family: var(--font-mono);
    font-size: var(--font-size-small);
    text-align: right;
  }

  .ticks {
    position: relative;
    height: 12px;
  }

  .ticks span {
    position: absolute;
    color: var(--text-faint);
    font-size: 9px;
    transform: translateX(-50%);
  }
</style>
