<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Video panel. Shows the session's reference video at the playhead, with a
  timecode. The picture comes from the decoder built into the system and is
  drawn on a canvas, so no web video player is involved. It follows the
  playhead to the nearest picture, not frame for frame while playing.
-->
<script lang="ts">
  import type { CommandRunner } from "../../stores/media-pool";
  import type { SessionDoc, SessionStore, VideoRefDoc } from "../../stores/session";
  import type { TransportState, TransportStore } from "../../stores/transport";
  import { videoClose, videoFrame, videoOpen, videoPick, videoSupport } from "../../app/core";
  import { askWidth, decodePicture, needsFrame, timecode, videoTime, type VideoMeta } from "../../app/video";

  interface Props {
    session: SessionStore;
    transport: TransportStore;
    runner: CommandRunner;
    onError?: (error: unknown) => void;
  }

  const { session, transport, runner, onError = console.error }: Props = $props();

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

  /** Why pictures can't be shown on this system, or null when they can. */
  let unsupported = $state<string | null>(null);
  void videoSupport().catch((e) => {
    unsupported = typeof e === "string" ? e : "Video is not available on this system.";
  });

  const video = $derived<VideoRefDoc | null>(doc?.video ?? null);
  const rate = $derived(doc?.sample_rate ?? 48000);
  const seconds = $derived(((tp?.playing ? tp.position : (doc?.view.playhead ?? 0)) as number) / rate);

  // The open reader for the current file.
  let open = $state<{ id: number; path: string; meta: VideoMeta } | null>(null);
  let problem = $state<string | null>(null);
  let opening = 0;

  $effect(() => {
    const path = video?.path ?? null;
    if (unsupported) return;
    if (open && open.path === path) return;
    const old = open;
    open = null;
    problem = null;
    last = null;
    clear();
    if (old) void videoClose(old.id).catch(() => undefined);
    if (!path) return;
    const mine = ++opening;
    void videoOpen(path)
      .then((o) => {
        if (mine !== opening) {
          void videoClose(o.id).catch(() => undefined);
          return;
        }
        open = { id: o.id, path, meta: o.meta };
      })
      .catch((e) => {
        if (mine === opening) problem = typeof e === "string" ? e : "Could not open that video.";
      });
  });
  // Let go of the file when the panel goes away.
  $effect(() => () => {
    opening++;
    if (open) void videoClose(open.id).catch(() => undefined);
  });

  let canvas = $state<HTMLCanvasElement | null>(null);
  let wrap = $state<HTMLDivElement | null>(null);
  let width = $state(320);

  const target = $derived(open && video ? videoTime(seconds, video.offset, open.meta.duration) : null);
  let last: number | null = null;
  let busy = false;
  let wanted: { time: number | null; id: number; width: number } | null = null;

  function clear() {
    const c = canvas?.getContext("2d");
    if (c && canvas) {
      c.fillStyle = "#000";
      c.fillRect(0, 0, canvas.width, canvas.height);
    }
  }

  // Ask for a picture when the playhead moved a frame; one request at a time, newest wins.
  $effect(() => {
    if (!open) return;
    const next = target;
    if (!needsFrame(last, next, open.meta.fps)) return;
    wanted = {
      time: next,
      id: open.id,
      width: askWidth(width, window.devicePixelRatio || 1, open.meta.width),
    };
    void pump();
  });

  async function pump() {
    if (busy) return;
    busy = true;
    try {
      while (wanted) {
        const w = wanted;
        wanted = null;
        if (w.time === null) {
          last = null;
          clear();
          continue;
        }
        try {
          const pic = decodePicture(await videoFrame(w.id, w.time, w.width));
          if (!pic || !canvas || open?.id !== w.id) continue;
          if (canvas.width !== pic.width || canvas.height !== pic.height) {
            canvas.width = pic.width;
            canvas.height = pic.height;
          }
          canvas.getContext("2d")?.putImageData(new ImageData(pic.rgba, pic.width, pic.height), 0, 0);
          last = w.time;
          problem = null;
        } catch (e) {
          problem = typeof e === "string" ? e : "Could not read a picture.";
          last = w.time;
        }
      }
    } finally {
      busy = false;
    }
  }

  // The panel's width sets the picture's size.
  $effect(() => {
    if (!wrap) return;
    const ro = new ResizeObserver((entries) => {
      const w = entries[0]?.contentRect.width;
      if (w) width = w;
    });
    ro.observe(wrap);
    return () => ro.disconnect();
  });

  async function choose() {
    try {
      const path = await videoPick();
      if (!path) return;
      await session.edit("Open Video Reference", { op: "set_video", video: { path, offset: video?.offset ?? 0 } }, runner);
    } catch (e) {
      onError(e);
    }
  }

  async function remove() {
    try {
      await session.edit("Remove Video Reference", { op: "set_video", video: null }, runner);
    } catch (e) {
      onError(e);
    }
  }

  async function setOffset(text: string) {
    if (!video) return;
    const v = Number.parseFloat(text);
    if (!Number.isFinite(v) || v === video.offset) return;
    try {
      await session.edit("Move Video", { op: "set_video", video: { path: video.path, offset: v } }, runner);
    } catch (e) {
      onError(e);
    }
  }

  const fps = $derived(open?.meta.fps ?? 25);
  const shown = $derived(target ?? 0);
</script>

<div class="video">
  {#if unsupported}
    <p class="note">{unsupported}</p>
  {:else if !video}
    <p class="note">No video yet. Pick one to line your edit up against the picture.</p>
    <button type="button" onclick={choose} disabled={!doc}>Open Video...</button>
  {:else}
    <div class="stage" bind:this={wrap}>
      <canvas bind:this={canvas} aria-label="Video picture"></canvas>
      <span class="tc" aria-label="Video timecode">{target === null ? "--:--:--:--" : timecode(shown, fps)}</span>
    </div>
    {#if problem}<p class="note err">{problem}</p>{/if}
    <div class="row">
      <label>
        Starts at (s)
        <input
          type="number"
          step="0.001"
          value={video.offset}
          onchange={(e) => void setOffset(e.currentTarget.value)}
        />
      </label>
      <button type="button" onclick={choose}>Change...</button>
      <button type="button" onclick={remove}>Remove</button>
    </div>
    <p class="path" title={video.path}>{video.path}</p>
  {/if}
</div>

<style>
  .video {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 6px;
    height: 100%;
    box-sizing: border-box;
    overflow: auto;
  }
  .stage {
    position: relative;
    background: #000;
    line-height: 0;
  }
  canvas {
    width: 100%;
    height: auto;
    display: block;
    background: #000;
  }
  .tc {
    position: absolute;
    left: 6px;
    bottom: 6px;
    line-height: 1;
    padding: 2px 5px;
    font: 12px/1 ui-monospace, Consolas, monospace;
    color: #fff;
    background: rgba(0, 0, 0, 0.6);
    border-radius: 3px;
  }
  .row {
    display: flex;
    gap: 6px;
    align-items: center;
    flex-wrap: wrap;
  }
  .row label {
    display: flex;
    gap: 4px;
    align-items: center;
  }
  .row input {
    width: 7em;
  }
  .note {
    margin: 0;
    opacity: 0.8;
  }
  .err {
    color: var(--error, #e5484d);
    opacity: 1;
  }
  .path {
    margin: 0;
    font-size: 11px;
    opacity: 0.6;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
