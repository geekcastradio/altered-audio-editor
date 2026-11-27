// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * The master's Mono button: hear the whole mix folded to mono, to check
 * how it holds up. Listening only, so it is not saved and never changes
 * an export (Export's own Mono choice writes a mono file).
 */

import { writable, type Readable } from "svelte/store";
import { setListenMute, setMonoCheck } from "../app/core";

const state = writable(false);
let on = false;

export const monoCheck: Readable<boolean> & { toggle(): Promise<void> } = {
  subscribe: state.subscribe,
  async toggle() {
    const next = !on;
    await setMonoCheck(next);
    on = next;
    state.set(next);
  },
};

/** Which sides are muted while listening. Not saved, and exports never see it. */
export interface ListenMute {
  left: boolean;
  right: boolean;
}

const muteState = writable<ListenMute>({ left: false, right: false });
let muted: ListenMute = { left: false, right: false };

export const listenMute: Readable<ListenMute> & { toggle(side: "left" | "right"): Promise<void> } = {
  subscribe: muteState.subscribe,
  async toggle(side) {
    const next = { ...muted, [side]: !muted[side] };
    await setListenMute(next.left, next.right);
    muted = next;
    muteState.set(next);
  },
};
