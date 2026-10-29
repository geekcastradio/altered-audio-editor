<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Analysis > Frequency Analysis, Amplitude Statistics and Histogram:
  one window with a tab for each, measuring the mix (or the time selection).
-->
<script lang="ts">
  import { binDb, dbY, findings, fmtDb, fmtHz, fmtSample, FREQ_TICKS, histogramBars, logX, type AnalysisReport, type AnalysisTab } from "./analysis";

  interface Props {
    report: AnalysisReport | null;
    /** 0 to 1 while measuring, null when idle. */
    progress: number | null;
    tab: AnalysisTab;
    /** "the whole session" or "the time selection". */
    scope: string;
    onTab: (t: AnalysisTab) => void;
    onMeasure: () => void;
    onCancel: () => void;
    onClose: () => void;
  }

  const { report, progress, tab, scope, onTab, onMeasure, onCancel, onClose }: Props = $props();

  const tabs: { id: AnalysisTab; label: string }[] = [
    { id: "frequency", label: "Frequency" },
    { id: "statistics", label: "Statistics" },
    { id: "histogram", label: "Histogram" },
    { id: "problems", label: "Problems" },
  ];

  const W = 560;
  const H = 220;
  const TOP = 0;
  const BOTTOM = -100;

  const spectrumPath = $derived.by(() => {
    if (!report || report.spectrumHz.length === 0) return "";
    return report.spectrumHz
      .map((hz, i) => `${i === 0 ? "M" : "L"}${(logX(hz) * W).toFixed(1)},${(dbY(report.spectrumDb[i] ?? BOTTOM, TOP, BOTTOM) * H).toFixed(1)}`)
      .join(" ");
  });

  const bars = $derived(report ? histogramBars(report.histogram) : []);
  const time = (s: number) => `${Math.floor(s / 60)}:${String(Math.floor(s % 60)).padStart(2, "0")}`;
  const sides = $derived(
    report
      ? ([
          ["Left", report.left],
          ["Right", report.right],
        ] as const)
      : [],
  );
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && progress === null && onClose()} />

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="backdrop" onclick={() => progress === null && onClose()}>
  <div class="dialog" role="dialog" aria-modal="true" aria-labelledby="an-title" tabindex="-1" onclick={(e) => e.stopPropagation()}>
    <h1 id="an-title">Analysis</h1>
    <div class="tabs" role="tablist">
      {#each tabs as t (t.id)}
        <button type="button" role="tab" aria-selected={tab === t.id} class:on={tab === t.id} onclick={() => onTab(t.id)}>{t.label}</button>
      {/each}
    </div>
    <p class="scope">Measuring {scope}.</p>

    {#if progress !== null}
      <div class="bar"><div style:width="{Math.round(progress * 100)}%"></div></div>
      <p class="scope">Analyzing... {Math.round(progress * 100)}%</p>
    {:else if report}
      {#if tab === "frequency"}
        <svg viewBox="0 0 {W} {H + 22}" class="plot" role="img" aria-label="Average spectrum">
          {#each [0, -20, -40, -60, -80, -100] as d (d)}
            <line x1="0" x2={W} y1={dbY(d, TOP, BOTTOM) * H} y2={dbY(d, TOP, BOTTOM) * H} class="grid" />
            <text x="3" y={dbY(d, TOP, BOTTOM) * H - 2} class="lab">{d}</text>
          {/each}
          {#each FREQ_TICKS as f (f)}
            <line x1={logX(f) * W} x2={logX(f) * W} y1="0" y2={H} class="grid" />
            <text x={logX(f) * W} y={H + 14} class="lab mid">{fmtHz(f)}</text>
          {/each}
          <path d={spectrumPath} class="curve" />
        </svg>
        <p class="scope">Average level at each frequency, in dB below full scale. A steady full-scale sine reads about 0 dB.</p>
      {:else if tab === "statistics"}
        <table>
          <thead><tr><th></th><th>Left</th><th>Right</th></tr></thead>
          <tbody>
            <tr><td>Peak</td><td>{fmtDb(report.left.peakDb)}</td><td>{fmtDb(report.right.peakDb)}</td></tr>
            <tr><td>Min sample</td><td>{fmtSample(report.left.minSample)}</td><td>{fmtSample(report.right.minSample)}</td></tr>
            <tr><td>Max sample</td><td>{fmtSample(report.left.maxSample)}</td><td>{fmtSample(report.right.maxSample)}</td></tr>
            <tr><td>RMS (average power)</td><td>{fmtDb(report.left.rmsDb)}</td><td>{fmtDb(report.right.rmsDb)}</td></tr>
            <tr><td>DC offset</td><td>{fmtSample(report.left.dcOffset)}</td><td>{fmtSample(report.right.dcOffset)}</td></tr>
            <tr class:warn={sides.some(([, c]) => c.clipped > 0)}><td>Clipped samples</td><td>{report.left.clipped}</td><td>{report.right.clipped}</td></tr>
          </tbody>
        </table>
        <dl>
          <dt>Quietest 50 ms</dt>
          <dd>{fmtDb(report.minWindowDb)}</dd>
          <dt>Average 50 ms</dt>
          <dd>{fmtDb(report.avgWindowDb)}</dd>
          <dt>Loudest 50 ms</dt>
          <dd>{fmtDb(report.maxWindowDb)}</dd>
          <dt>Peak over average (crest)</dt>
          <dd>{fmtDb(report.crestDb)}</dd>
          <dt>Length</dt>
          <dd>{time(report.seconds)} at {report.sampleRate} Hz</dd>
        </dl>
        {#if Math.abs(report.left.dcOffset) > 0.005 || Math.abs(report.right.dcOffset) > 0.005}
          <p class="advice">There is a noticeable DC offset. The DC Offset Fix effect removes it.</p>
        {/if}
      {:else if tab === "problems"}
        <ul class="finds">
          {#each findings(report) as f (f.id)}
            <li class:warn={f.found}>
              <strong>{f.text}</strong>
              {#if f.at.length > 0}<span class="at">At {f.at.join(", ")}</span>{/if}
              {#if f.found}<span class="fix">{f.advice}</span>{/if}
            </li>
          {/each}
        </ul>
        <p class="scope">A quick check, not a verdict: it flags what is worth a listen. Times count from the start of what was analyzed.</p>
      {:else}
        <svg viewBox="0 0 {W} {H + 22}" class="plot" role="img" aria-label="Level histogram">
          {#each bars as b, i (i)}
            <rect x={(i / bars.length) * W} y={H - b * H} width={W / bars.length - 1} height={b * H} class="bar-rect" />
          {/each}
          {#each [0, 20, 40, 60, 80] as i (i)}
            <text x={(i / bars.length) * W} y={H + 14} class="lab">{binDb(i)}</text>
          {/each}
        </svg>
        <p class="scope">How many samples sit at each level, in 1 dB steps from full scale (left) down to quiet (right). Counts use a log scale so quiet detail shows. {report.silentSamples.toLocaleString()} samples are exactly zero.</p>
      {/if}
    {:else}
      <p class="scope">Measure to see the results.</p>
    {/if}

    <footer>
      {#if progress !== null}
        <button type="button" onclick={onCancel}>Cancel</button>
      {:else}
        <button type="button" onclick={onMeasure}>{report ? "Measure Again" : "Measure"}</button>
        <button type="button" class="primary" onclick={onClose}>Close</button>
      {/if}
    </footer>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 5000;
    display: grid;
    place-items: center;
    background: rgba(0, 0, 0, 0.5);
  }

  .dialog {
    --dialog-pad-x: 14px;
    --dialog-pad-y: 14px;
    display: grid;
    gap: 10px;
    width: min(620px, 94vw);
    padding: 14px;
    border: 1px solid var(--border-strong);
    background: var(--surface-2);
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.6);
  }

  h1 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
  }

  .tabs {
    display: flex;
    gap: 2px;
    border-bottom: 1px solid var(--border);
  }

  .tabs button {
    height: 24px;
    padding: 0 12px;
    border: 1px solid transparent;
    border-bottom: 0;
    border-radius: 3px 3px 0 0;
    background: transparent;
    color: var(--text-dim);
    font: inherit;
  }

  .tabs button.on {
    border-color: var(--border);
    background: var(--surface-1);
    color: var(--text);
  }

  .scope {
    margin: 0;
    color: var(--text-faint);
    font-size: var(--font-size-small);
  }

  .bar {
    height: 6px;
    border-radius: 3px;
    background: var(--surface-0);
    overflow: hidden;
  }

  .bar div {
    height: 100%;
    background: var(--accent);
  }

  .plot {
    width: 100%;
    background: var(--surface-0);
    border: 1px solid var(--border);
  }

  .grid {
    stroke: var(--border);
    stroke-width: 0.5;
  }

  .lab {
    fill: var(--text-faint);
    font-size: 9px;
  }

  .lab.mid {
    text-anchor: middle;
  }

  .curve {
    fill: none;
    stroke: var(--accent);
    stroke-width: 1.2;
  }

  .bar-rect {
    fill: var(--accent);
  }

  table {
    border-collapse: collapse;
    font-size: var(--font-size-small);
  }

  th,
  td {
    padding: 2px 8px;
    border-bottom: 1px solid var(--border);
    text-align: right;
    font-family: var(--font-mono);
    color: var(--value);
  }

  th:first-child,
  td:first-child {
    text-align: left;
    font-family: inherit;
    color: var(--text-dim);
  }

  tr.warn td {
    color: var(--danger);
  }

  dl {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    gap: 2px 12px;
    margin: 0;
    font-size: var(--font-size-small);
  }

  dt {
    color: var(--text-dim);
  }

  dd {
    margin: 0;
    color: var(--value);
    font-family: var(--font-mono);
    text-align: right;
  }

  .finds {
    display: grid;
    gap: 8px;
    margin: 0;
    padding: 0;
    list-style: none;
  }
  .finds li {
    display: grid;
    gap: 2px;
    padding: 6px 8px;
    border: 1px solid var(--border);
    border-radius: 3px;
    color: var(--text-dim);
  }
  .finds li.warn {
    border-color: var(--warning, #c9a227);
    color: var(--text);
  }
  .finds .at,
  .finds .fix {
    color: var(--text-faint);
  }

  .advice {
    margin: 0;
    padding: 6px 8px;
    border-left: 2px solid var(--accent);
    background: var(--surface-1);
    color: var(--text);
    font-size: var(--font-size-small);
  }

  footer {
    display: flex;
    gap: 6px;
    justify-content: flex-end;
  }

  footer button {
    min-width: 72px;
    height: 22px;
    padding: 0 10px;
    border: 1px solid var(--border-strong);
    border-radius: 2px;
    background: var(--surface-3);
    color: var(--text);
    font: inherit;
  }

  footer button:hover {
    border-color: var(--accent);
  }

  footer button.primary {
    border-color: var(--accent);
  }
</style>
