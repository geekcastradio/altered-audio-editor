<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  One track lane: clips with their waveforms, drawn on a canvas sized to
  the visible width. Redraws when the view, the track, or peaks change.
-->
<script lang="ts">
  import type { PeakStore } from "../../stores/peaks";
  import { clipLength, stretchOf, type ClipDoc, type FadeDoc, type MediaRefDoc, type TrackDoc } from "../../stores/session";
  import { LIVE_BLOCK } from "../../app/core";
  import { defaultTrackColor, trackHeight, peakRange, pickLevel, xOf, type Viewport, TRACK_GAP } from "./timeline";
  import { chipBox, chipLabel, fxLaneOf, pointsOf, rangeOf, restValue, valueToY, type EnvKind, type FxLane } from "./envelope";

  interface Props {
    track: TrackDoc;
    media: ReadonlyMap<string, MediaRefDoc>;
    sessionRate: number;
    view: Viewport;
    width: number;
    playhead: number;
    selectedClips: ReadonlySet<string>;
    trackSelected: boolean;
    /** Row number, for alternating lane shades. */
    index?: number;
    /** Waveform display scale. */
    ampZoom?: number;
    /** A take being recorded on this track: where it starts, the input rate, and min/max blocks. */
    liveTake?: { at: number; rate: number; blocks: [number, number][] } | null;
    peaks: PeakStore;
    /** Changes whenever any peak data changes. */
    peakTick: number;
    /** Draw clip volume envelopes. */
    showVolume?: boolean;
    /** Draw clip pan envelopes. */
    showPan?: boolean;
    /** Effect-setting envelopes, and which setting each clip shows. */
    showFx?: boolean;
    fxLanes?: Record<string, FxLane>;
  }

  const { track, media, sessionRate, view, width, playhead, selectedClips, trackSelected, index = 0, ampZoom = 1, liveTake = null, peaks, peakTick, showVolume = false, showPan = false, showFx = false, fxLanes = {} }: Props = $props();

  let canvas = $state<HTMLCanvasElement | null>(null);

  function css(name: string): string {
    return getComputedStyle(document.documentElement).getPropertyValue(name).trim() || "#888";
  }

  /** Fade gain for t from 0 to 1. Matches the playback engine. */
  function fadeGain(shape: FadeDoc["shape"], t: number): number {
    const u = Math.min(1, Math.max(0, t));
    if (shape === "logarithmic") return 1 - (1 - u) * (1 - u);
    if (shape === "cosine") return 0.5 - 0.5 * Math.cos(Math.PI * u);
    return u;
  }

  /** Shades the silent side of a fade and draws its curve. */
  function drawFade(g: CanvasRenderingContext2D, fade: FadeDoc, from: number, to: number, top: number, height: number, rising: boolean) {
    if (fade.length <= 0 || to - from < 1) return;
    const bottom = top + height;
    const steps = Math.max(2, Math.ceil(to - from));
    g.beginPath();
    g.moveTo(from, top);
    for (let i = 0; i <= steps; i++) {
      const x = from + ((to - from) * i) / steps;
      const t = i / steps;
      const gain = fadeGain(fade.shape, rising ? t : 1 - t);
      g.lineTo(x, bottom - gain * height);
    }
    g.lineTo(to, top);
    g.closePath();
    g.fillStyle = "rgba(0, 0, 0, 0.38)";
    g.fill();
    g.beginPath();
    for (let i = 0; i <= steps; i++) {
      const x = from + ((to - from) * i) / steps;
      const t = i / steps;
      const y = bottom - fadeGain(fade.shape, rising ? t : 1 - t) * height;
      if (i === 0) g.moveTo(x, y);
      else g.lineTo(x, y);
    }
    g.strokeStyle = "rgba(255, 255, 255, 0.7)";
    g.lineWidth = 1;
    g.stroke();
  }

  function clipColor(clip: ClipDoc): string {
    const c = clip.color ?? track.color ?? defaultTrackColor(index);
    return c.startsWith("--") ? css(c) : c;
  }

  $effect(() => {
    void peakTick;
    const c = canvas;
    if (!c) return;
    const dpr = window.devicePixelRatio || 1;
    const h = trackHeight(track);
    c.width = Math.max(1, Math.round(width * dpr));
    c.height = Math.max(1, Math.round(h * dpr));
    c.style.width = `${width}px`;
    c.style.height = `${h}px`;
    const g = c.getContext("2d");
    if (!g) return;
    g.setTransform(dpr, 0, 0, dpr, 0, 0);
    g.clearRect(0, 0, width, h);
    // The bottom TRACK_GAP pixels are the dark break between tracks.
    const lh = h - TRACK_GAP;
    g.fillStyle = trackSelected ? css("--lane-selected") : css(index % 2 ? "--lane-b" : "--lane-a");
    g.fillRect(0, 0, width, lh);

    const titleH = 14;
    for (const clip of track.clips) {
      const m = media.get(clip.media_id);
      const len = clipLength(clip, m, sessionRate);
      const x0 = xOf(clip.start, view);
      const x1 = xOf(clip.start + len, view);
      if (x1 < 0 || x0 > width) continue;
      const left = Math.max(0, x0);
      const right = Math.min(width, x1);
      const selected = selectedClips.has(clip.id);
      const color = clipColor(clip);

      // Body and title bar.
      g.globalAlpha = clip.muted ? 0.35 : 1;
      g.fillStyle = color;
      g.globalAlpha *= 0.28;
      g.fillRect(left, 1, right - left, lh - 2);
      g.globalAlpha = clip.muted ? 0.35 : 1;
      g.fillStyle = color;
      g.fillRect(left, 1, right - left, titleH);
      g.fillStyle = "#0e0e10";
      g.font = `11px ${css("--font-ui")}`;
      g.textBaseline = "middle";
      g.save();
      g.beginPath();
      g.rect(left, 1, right - left, titleH);
      g.clip();
      const st = stretchOf(clip);
      // A stretched clip says so in its title, with the pitch mode.
      const label = st === 1 ? clip.name : `${clip.name}  ${Math.round(st * 1000) / 10}%${clip.stretch?.keep_pitch === false ? " tape" : ""}`;
      g.fillText(label, Math.max(x0, 0) + 4, 1 + titleH / 2);
      g.restore();

      // Waveform.
      const body = { top: 1 + titleH, height: lh - 2 - titleH };
      const entry = m ? peaks.get(m.path) : null;
      if (entry?.status === "ready" && entry.summary) {
        const ch = entry.summary.channels;
        const srcRate = m!.sample_rate || entry.summary.sampleRate;
        // A stretched clip covers less (or more) of its file per pixel.
        const ratio = stretchOf(clip);
        const framesPerPx = (view.spp * srcRate) / sessionRate / ratio;
        const level = pickLevel(entry.levels, framesPerPx);
        if (!level || level.data.length === 0) {
          // Ready but nothing to draw: say so instead of leaving the clip blank.
          g.save();
          g.fillStyle = css("--danger");
          g.font = `11px ${css("--font-ui")}`;
          g.fillText(`No waveform data (${entry.levels.length} levels)`, left + 6, body.top + 12);
          g.restore();
        }
        if (level) {
          g.fillStyle = color;
          g.globalAlpha = clip.muted ? 0.35 : 0.95;
          const laneH = body.height / ch;
          // A looped clip wraps back to the start of its region.
          const region = clip.source_out - clip.source_in;
          const phase = clip.looped ? clip.looped.phase : 0;
          for (let px = Math.floor(left); px < Math.ceil(right); px++) {
            const tl0 = view.scroll + px * view.spp - clip.start;
            let into = (tl0 * srcRate) / sessionRate / ratio;
            if (clip.looped && region > 0) into = (((into + phase) % region) + region) % region;
            const s0 = clip.source_in + into;
            const s1 = s0 + framesPerPx;
            if (s1 < clip.source_in || s0 >= clip.source_out) continue;
            for (let k = 0; k < ch; k++) {
              const [lo, hi] = peakRange(level, ch, k, Math.max(s0, clip.source_in), Math.min(s1, clip.source_out));
              const mid = body.top + laneH * (k + 0.5);
              const amp = (v: number) => Math.max(-1, Math.min(1, v * ampZoom));
              const y0 = mid - amp(hi) * (laneH / 2 - 1);
              const y1 = mid - amp(lo) * (laneH / 2 - 1);
              g.fillRect(px, y0, 1, Math.max(1, y1 - y0));
            }
          }
          g.globalAlpha = 1;
        }
        // Looped clips: a dashed line where each repeat begins.
        if (clip.looped && clip.source_out > clip.source_in) {
          const regionTl = ((clip.source_out - clip.source_in) * sessionRate) / srcRate;
          const firstTl = regionTl - (clip.looped.phase * sessionRate) / srcRate;
          if (regionTl / view.spp >= 4) {
            g.save();
            g.strokeStyle = "rgba(255, 255, 255, 0.55)";
            g.setLineDash([3, 3]);
            g.lineWidth = 1;
            for (let t = firstTl; t < clip.looped.length; t += regionTl) {
              const lx = Math.round(xOf(clip.start + t, view)) + 0.5;
              if (lx < left || lx > right) continue;
              g.beginPath();
              g.moveTo(lx, body.top);
              g.lineTo(lx, body.top + body.height);
              g.stroke();
            }
            g.restore();
          }
        }
      } else {
        // Status text, kept inside the clip so neighbors never overlap.
        g.save();
        g.beginPath();
        g.rect(left, body.top, right - left, body.height);
        g.clip();
        g.font = `11px ${css("--font-ui")}`;
        if (!entry || entry.status === "error") {
          g.fillStyle = css("--danger");
          const why = entry?.error === "File not found" || !entry ? "Media offline" : `No waveform: ${entry.error ?? ""}`;
          g.fillText(why, left + 6, body.top + 12);
        } else {
          g.fillStyle = css("--text-dim");
          g.fillText(`Building waveform ${Math.round(entry.progress * 100)}%`, left + 6, body.top + 12);
        }
        g.restore();
      }

      // Fades, and their handles in the title bar.
      g.globalAlpha = 1;
      const fi = x0 + clip.fade_in.length / view.spp;
      const fo = x1 - clip.fade_out.length / view.spp;
      g.save();
      g.beginPath();
      g.rect(left, body.top, right - left, body.height);
      g.clip();
      drawFade(g, clip.fade_in, x0, fi, body.top, body.height, true);
      drawFade(g, clip.fade_out, fo, x1, body.top, body.height, false);
      g.restore();
      if (x1 - x0 > 24) {
        g.fillStyle = "#f2f2f2";
        for (const hx of [fi, fo]) {
          if (hx < -4 || hx > width + 4) continue;
          const cx = Math.min(Math.max(hx, x0 + 4), x1 - 4);
          g.fillRect(Math.round(cx) - 3, 4, 7, 7);
          g.strokeStyle = "#0e0e10";
          g.strokeRect(Math.round(cx) - 2.5, 4.5, 6, 6);
        }
      }
      // Envelope chip: which envelope lines show; click it to change.
      const chip = chipBox(x0, x1, chipLabel(showVolume, showPan, showFx));
      let tagRight = right - 14;
      if (chip && chip.x >= left && chip.x + chip.w <= right) {
        g.fillStyle = "rgba(14, 14, 16, 0.55)";
        g.beginPath();
        g.roundRect(chip.x, 2, chip.w, titleH - 2, 3);
        g.fill();
        g.fillStyle = "#f2f2f2";
        g.font = `10px ${css("--font-ui")}`;
        g.textBaseline = "middle";
        g.fillText(`${chipLabel(showVolume, showPan, showFx)} \u25BE`, chip.x + 6, 1 + titleH / 2);
        tagRight = chip.x - 6;
      }
      if (clip.locked || clip.gain_db !== 0 || clip.group || clip.looped) {
        const tag = [
          clip.gain_db !== 0 ? `${clip.gain_db > 0 ? "+" : ""}${clip.gain_db.toFixed(1)} dB` : "",
          clip.looped ? "Looped" : "",
          clip.group ? "Grouped" : "",
          clip.locked ? "Locked" : "",
        ]
          .filter(Boolean)
          .join("  ");
        g.font = `10px ${css("--font-ui")}`;
        const w = g.measureText(tag).width;
        if (tagRight - left > w + 40) {
          g.fillStyle = "#0e0e10";
          g.textBaseline = "middle";
          g.fillText(tag, tagRight - w, 1 + titleH / 2);
        }
      }

      // Envelopes: a line across the body with a handle per point.
      // Volume is yellow, pan is blue (full left at the top).
      // An effect setting is violet, its top the control's highest value.
      const fxLane = showFx ? fxLaneOf(clip, fxLanes) : null;
      const shown: [EnvKind, string][] = [];
      if (showVolume) shown.push(["volume", "255, 215, 94"]);
      if (showPan) shown.push(["pan", "120, 235, 255"]);
      if (fxLane) shown.push(["fx", "205, 150, 255"]);
      for (const [kind, rgb] of shown) {
        const pts = pointsOf(clip, kind, fxLane);
        const range = rangeOf(kind, clip, fxLane);
        const rest = restValue(kind, clip, fxLane);
        const yAt = (v: number) => valueToY(kind, v, body.top + 2, body.height - 4, range);
        g.save();
        g.beginPath();
        g.rect(left, body.top, right - left, body.height);
        g.clip();
        g.strokeStyle = pts.length ? `rgb(${rgb})` : `rgba(${rgb}, 0.8)`;
        g.lineWidth = 1.5;
        g.beginPath();
        if (pts.length === 0) {
          g.moveTo(x0, yAt(rest));
          g.lineTo(x1, yAt(rest));
        } else {
          g.moveTo(x0, yAt(pts[0]!.value));
          for (const p of pts) g.lineTo(xOf(clip.start + p.position, view), yAt(p.value));
          g.lineTo(x1, yAt(pts[pts.length - 1]!.value));
        }
        g.stroke();
        g.fillStyle = `rgb(${rgb})`;
        for (const p of pts) {
          const px = xOf(clip.start + p.position, view);
          g.fillRect(Math.round(px) - 3, Math.round(yAt(p.value)) - 3, 6, 6);
        }
        g.restore();
      }

      // Raised edge: a light top and left, a dark bottom and right, so clips stand off the lane.
      g.globalAlpha = clip.muted ? 0.35 : 1;
      g.fillStyle = "rgba(255, 255, 255, 0.28)";
      g.fillRect(left, 1, right - left, 1);
      if (x0 >= 0) g.fillRect(x0 + 1, 1, 1, lh - 3);
      g.fillStyle = "rgba(0, 0, 0, 0.55)";
      g.fillRect(left, lh - 2, right - left, 1);
      if (x1 <= width) g.fillRect(x1 - 2, 1, 1, lh - 3);

      // Outline.
      g.globalAlpha = 1;
      g.strokeStyle = selected ? "#ffffff" : color;
      g.lineWidth = selected ? 1.5 : 1;
      g.strokeRect(x0 + 0.5, 1.5, x1 - x0 - 1, lh - 3);
    }

    // A take being recorded: drawn as it arrives.
    if (liveTake && liveTake.rate > 0 && liveTake.blocks.length) {
      const per = (LIVE_BLOCK * sessionRate) / liveTake.rate; // session samples per block
      const x0 = xOf(liveTake.at, view);
      const x1 = xOf(liveTake.at + per * liveTake.blocks.length, view);
      const left = Math.max(0, x0);
      const right = Math.min(width, x1);
      if (right > left) {
        g.fillStyle = "rgba(229, 83, 75, 0.22)";
        g.fillRect(left, 1, right - left, lh - 2);
        g.fillStyle = css("--danger");
        g.fillRect(left, 1, right - left, 14);
        g.fillStyle = "#0e0e10";
        g.font = `11px ${css("--font-ui")}`;
        g.textBaseline = "middle";
        g.fillText("Recording", left + 4, 8);
        const top = 16;
        const mid = top + (lh - 2 - top) / 2;
        const half = (lh - 2 - top) / 2 - 1;
        g.fillStyle = "#ff8a80";
        for (let px = Math.floor(left); px < Math.ceil(right); px++) {
          const b0 = Math.floor(((px + view.scroll / view.spp) * view.spp - liveTake.at) / per);
          const b1 = Math.max(b0 + 1, Math.floor(((px + 1 + view.scroll / view.spp) * view.spp - liveTake.at) / per));
          let lo = 0;
          let hi = 0;
          for (let b = Math.max(0, b0); b < Math.min(liveTake.blocks.length, b1); b++) {
            lo = Math.min(lo, liveTake.blocks[b]![0]);
            hi = Math.max(hi, liveTake.blocks[b]![1]);
          }
          const y0 = mid - Math.min(1, hi * ampZoom) * half;
          const y1 = mid - Math.max(-1, lo * ampZoom) * half;
          g.fillRect(px, y0, 1, Math.max(1, y1 - y0));
        }
      }
    }

    // Playhead.
    const px = xOf(playhead, view);
    if (px >= 0 && px <= width) {
      g.fillStyle = css("--playhead");
      g.fillRect(Math.round(px), 0, 1, h);
    }

    // Break between tracks.
    g.fillStyle = css("--track-gap");
    g.fillRect(0, lh, width, TRACK_GAP);
  });
</script>

<canvas bind:this={canvas} aria-hidden="true"></canvas>

<style>
  canvas {
    display: block;
  }
</style>
