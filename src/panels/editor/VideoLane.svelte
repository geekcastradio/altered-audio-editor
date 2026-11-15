<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  The Video Reference bar under the Multitrack ruler: one bar with a film icon
  and the file's name, from where the picture starts to where it ends. Drag it
  to change when the picture starts. It belongs to the session, not to a track.
-->
<script lang="ts">
  import type { VideoRefDoc } from "../../stores/session";
  import type { Viewport } from "./timeline";
  import { videoProbe } from "../../app/core";
  import { baseName, draggedOffset, laneBar } from "./video-lane";

  interface Props {
    video: VideoRefDoc;
    view: Viewport;
    width: number;
    rate: number;
    onMove: (offset: number) => void;
  }

  const { video, view, width, rate, onMove }: Props = $props();

  /** Length of each video seen so far, so the bar does not re-read the file on every redraw. */
  const lengths = new Map<string, number>();
  let duration = $state(0);
  $effect(() => {
    const path = video.path;
    const known = lengths.get(path);
    if (known !== undefined) {
      duration = known;
      return;
    }
    duration = 0;
    let gone = false;
    void videoProbe(path)
      .then((m) => {
        lengths.set(path, m.duration);
        if (!gone) duration = m.duration;
      })
      .catch(() => undefined);
    return () => {
      gone = true;
    };
  });

  /** The offset while the bar is being dragged, shown before it is saved. */
  let dragging = $state<number | null>(null);
  const offset = $derived(dragging ?? video.offset);
  const bar = $derived(laneBar(offset, duration, view.spp, view.scroll, rate, width));

  function onDown(e: PointerEvent) {
    if (e.button !== 0) return;
    const el = e.currentTarget as HTMLElement;
    el.setPointerCapture(e.pointerId);
    const x0 = e.clientX;
    const from = video.offset;
    let moved = false;
    const move = (ev: PointerEvent) => {
      if (!moved && Math.abs(ev.clientX - x0) < 3) return;
      moved = true;
      dragging = draggedOffset(from, ev.clientX - x0, view.spp, rate);
    };
    const up = () => {
      el.removeEventListener("pointermove", move);
      el.removeEventListener("pointerup", up);
      el.removeEventListener("pointercancel", up);
      const to = dragging;
      dragging = null;
      if (moved && to !== null && to !== from) onMove(to);
    };
    el.addEventListener("pointermove", move);
    el.addEventListener("pointerup", up);
    el.addEventListener("pointercancel", up);
  }
</script>

<div class="strip" style:width="{width}px" aria-label="Video Reference">
  {#if bar}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="bar" class:dragging={dragging !== null} style:left="{bar.left}px" style:width="{bar.width}px" title="{video.path}. Drag to change when the picture starts." onpointerdown={onDown}>
      <svg viewBox="0 0 16 16" aria-hidden="true"><rect x="2" y="3" width="12" height="10" rx="1.5" fill="none" stroke="currentColor" stroke-width="1.3" /><path d="M2 6.5h12M2 9.5h12M5.5 3v10M10.5 3v10" stroke="currentColor" stroke-width="1" opacity="0.7" /></svg>
      <span>{baseName(video.path)}</span>
    </div>
  {/if}
</div>

<style>
  .strip {
    position: relative;
    flex: none;
    height: 16px;
    border-bottom: 1px solid var(--border);
    background: var(--surface-1);
    overflow: hidden;
  }
  .bar {
    position: absolute;
    top: 1px;
    bottom: 1px;
    display: flex;
    gap: 4px;
    align-items: center;
    padding: 0 5px;
    box-sizing: border-box;
    border-radius: 3px;
    background: color-mix(in srgb, var(--accent) 28%, var(--surface-2));
    color: var(--text);
    font-size: 10px;
    line-height: 1;
    white-space: nowrap;
    overflow: hidden;
    cursor: ew-resize;
    user-select: none;
    touch-action: none;
  }
  .bar.dragging {
    opacity: 0.75;
  }
  svg {
    flex: none;
    width: 11px;
    height: 11px;
  }
  span {
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
