<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Time Info: where the playhead is, the span of the selected clips, and
  the session's length, format and mixdown size.
-->
<script lang="ts">
  import { editor, type EditorState } from "../../stores/editor";
  import { clipLength, sessionEnd, type SessionDoc, type SessionStore } from "../../stores/session";
  import type { TransportState, TransportStore } from "../../stores/transport";
  import { formatIn } from "../editor/timeline";
  import { formatSize } from "../files/format";
  import { mixdownBytes } from "../../app/session-properties";

  interface Props {
    session: SessionStore;
    transport: TransportStore;
  }

  const { session, transport }: Props = $props();

  let doc = $state<SessionDoc | null>(null);
  $effect(() =>
    session.subscribe((d) => {
      doc = d;
    }),
  );
  let tp = $state<TransportState | null>(null);
  $effect(() =>
    transport.subscribe((t) => {
      tp = t;
    }),
  );
  let ed = $state<EditorState>(editor.current());
  $effect(() =>
    editor.subscribe((e) => {
      ed = e;
    }),
  );

  const rate = $derived(doc?.sample_rate ?? 48000);
  const t = (samples: number) => formatIn(samples, rate, ed.timeFormat);

  /** The time selection if there is one, otherwise the span of the selected clips. */
  const span = $derived.by(() => {
    if (doc?.view.selection) return doc.view.selection;
    if (!doc || ed.selectedClips.size === 0) return null;
    const media = new Map(doc.media.map((m) => [m.id, m]));
    let start = Infinity;
    let end = 0;
    for (const tr of doc.tracks)
      for (const c of tr.clips)
        if (ed.selectedClips.has(c.id)) {
          start = Math.min(start, c.start);
          end = Math.max(end, c.start + clipLength(c, media.get(c.media_id), rate));
        }
    return start === Infinity ? null : { start, end };
  });

  const length = $derived(doc ? sessionEnd(doc) : 0);
  const playhead = $derived(tp?.playing ? tp.position : (doc?.view.playhead ?? 0));
  /** A stereo WAV mixdown of the whole session at the session bit depth. */
  const size = $derived(mixdownBytes(length, doc?.bit_depth ?? 32));
</script>

<section class="info" aria-label="Time Info">
  <div class="grid">
    <span class="k">Playhead</span>
    <span class="v big">{t(playhead)}</span>

    <span class="k" title={doc?.view.selection ? "Time selection" : "Selected clips"}>Start</span>
    <span class="v">{span ? t(span.start) : "-"}</span>
    <span class="k">End</span>
    <span class="v">{span ? t(span.end) : "-"}</span>
    <span class="k">Duration</span>
    <span class="v">{span ? t(span.end - span.start) : "-"}</span>
  </div>
  <div class="foot">
    <span title="Session length">{t(length)}</span>
    <span>{rate.toLocaleString("en-US")} Hz</span>
    <span>{doc?.bit_depth === 32 ? "32-bit float" : `${doc?.bit_depth ?? 32}-bit`}</span>
    <span title="Size of a stereo WAV mixdown at the session bit depth">{formatSize(size)}</span>
  </div>
</section>

<style>
  .info {
    display: grid;
    grid-template-rows: auto auto;
    align-content: start;
    gap: 2px;
    height: 100%;
    overflow: auto;
    padding: 3px 10px;
    line-height: 1.2;
    box-sizing: border-box;
    background: var(--surface-1);
  }

  .grid {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 0 10px;
  }

  .k {
    color: var(--text-dim);
    font-size: var(--font-size-small);
  }

  .v {
    color: var(--value);
    font-family: var(--font-mono);
    font-size: var(--font-size-small);
    font-variant-numeric: tabular-nums;
    text-align: right;
  }

  .v.big {
    font-size: 14px;
  }

  .foot {
    display: flex;
    flex-wrap: wrap;
    gap: 2px 10px;
    padding-top: 4px;
    border-top: 1px solid var(--border);
    color: var(--text-faint);
    font-size: 10px;
  }
</style>
