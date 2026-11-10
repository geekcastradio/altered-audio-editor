// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * Audible scrubbing: dragging the playhead along the ruler plays the audio
 * under it. Playback restarts at the pointer at most every `gap` ms, so the
 * sound follows the drag in short pieces, and stops when the button is let
 * go with the playhead where the pointer ended. Only while stopped, and only
 * when the setting is on; a drag that began during playback keeps its old
 * behavior (playback jumps when the button is let go).
 */

export interface ScrubTransport {
  readonly current: { readonly playing: boolean };
  play(from?: number): Promise<void>;
  pause(): Promise<void>;
  seek(to: number): Promise<void>;
}

export class Scrubber {
  private started = false;
  private lastAt = -Infinity;
  private lastPos = Number.NaN;
  /** True when this drag plays audio: the setting is on and nothing was playing when it began. */
  readonly active: boolean;

  constructor(
    private readonly t: ScrubTransport,
    enabled: boolean,
    private readonly now: () => number = () => performance.now(),
    private readonly gap = 70,
  ) {
    this.active = enabled && !t.current.playing;
  }

  /** The pointer is at `pos` (samples). */
  move(pos: number): void {
    if (!this.active || pos === this.lastPos) return;
    const at = this.now();
    if (at - this.lastAt < this.gap) return;
    this.lastAt = at;
    this.lastPos = pos;
    this.started = true;
    void this.t.play(pos);
  }

  /** The button was let go at `pos`: stop and leave the playhead there. */
  async end(pos: number): Promise<void> {
    if (this.started) await this.t.pause();
    await this.t.seek(pos);
  }
}
