<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  A track's level meter: left and right bars on a -60 to 0 dB scale,
  green to -12 dB, yellow to -3 dB, red above, with a held peak line and
  an over light per channel that stays lit until clicked. While a track
  is armed it shows the input level instead, on both bars.
-->
<script lang="ts">
  import { meterFraction, type MeterLevel } from "../../stores/transport";

  interface Props {
    levels: readonly [MeterLevel, MeterLevel] | null;
    armed: boolean;
    /** Clears the over lights. */
    onClear: () => void;
  }

  const { levels, armed, onClear }: Props = $props();
</script>

<div class="meter" class:armed title={armed ? "Input level" : "Track level. Click a red light to clear it."}>
  <div class="overs">
    {#each [0, 1] as const as ch (ch)}
      <button
        type="button"
        class="over"
        class:on={levels?.[ch].over}
        aria-label="Clear over lights"
        tabindex="-1"
        onclick={(e) => {
          e.stopPropagation();
          onClear();
        }}
      ></button>
    {/each}
  </div>
  <div class="bars" aria-hidden="true">
    {#each [0, 1] as const as ch (ch)}
      {@const m = levels?.[ch]}
      <div class="bar">
        <div class="cover" style:height="{(1 - meterFraction(m?.level ?? 0)) * 100}%"></div>
        {#if m && m.peak > 0.001}<div class="peak" style:bottom="{meterFraction(m.peak) * 100}%"></div>{/if}
      </div>
    {/each}
  </div>
</div>

<style>
  .meter {
    position: absolute;
    top: 5px;
    right: 4px;
    bottom: 8px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    width: 15px;
  }

  .overs,
  .bars {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1px;
  }

  .bars {
    flex: 1;
    min-height: 0;
    padding: 1px;
    border: 1px solid var(--border-strong);
    border-radius: 2px;
    background: var(--surface-0);
  }

  .meter.armed .bars {
    border-color: color-mix(in srgb, var(--danger) 60%, var(--border-strong));
  }

  /* -60 to 0 dB: -12 dB sits at 80%, -3 dB at 95%. The cover hides the
     part of the scale above the current level. */
  .bar {
    position: relative;
    overflow: hidden;
    background: linear-gradient(to top, #3d8f6a 0 80%, #c9b33f 80% 95%, #e5534b 95% 100%);
  }

  .cover {
    position: absolute;
    top: 0;
    right: 0;
    left: 0;
    background: var(--surface-0);
  }

  .peak {
    position: absolute;
    right: 0;
    left: 0;
    height: 2px;
    background: var(--text);
    transform: translateY(1px);
  }

  .over {
    height: 6px;
    padding: 0;
    border: 1px solid var(--border-strong);
    border-radius: 1px;
    background: var(--surface-0);
    cursor: pointer;
  }

  .over.on {
    border-color: var(--danger);
    background: var(--danger);
  }
</style>
