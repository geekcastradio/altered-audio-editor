<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Edit > Controllers: pick the MIDI devices to listen to and say what each
  fader, knob and button does. Changes apply at once and are kept.
-->
<script lang="ts">
  import { onMount } from "svelte";
  import type { Controller } from "./controller/controller";
  import { controllerPorts } from "./core";
  import { mackiePreset, sourceName, targetChoices, targetName, type Binding, type Target } from "./controller/map";
  import { settings } from "../stores/settings";

  interface Props {
    controller: Controller;
    /** How many tracks the open session has. */
    tracks: number;
    onClose: () => void;
  }

  const { controller, tracks, onClose }: Props = $props();

  let ports = $state<string[]>([]);
  let looked = $state(false);
  let current = $state(settings.current);
  let picked = $state(0);
  let relative = $state(false);
  let waiting = $state(false);
  let stopLearning: (() => void) | null = null;
  let seen = $state("Nothing yet");

  const choices = $derived(targetChoices(Math.max(8, tracks)));
  const chosen = $derived(choices[picked] ?? choices[0]!);
  const asKnob = $derived(chosen.type === "volume" || chosen.type === "pan" || chosen.type === "master");

  async function refresh() {
    ports = await controllerPorts().catch(() => []);
    looked = true;
  }

  function save(next: Partial<typeof current>) {
    settings.set({ ...settings.current, ...next });
    current = settings.current;
  }

  function togglePort(name: string, on: boolean) {
    const set = new Set(current.controllerPorts);
    if (on) set.add(name);
    else set.delete(name);
    save({ controllerPorts: [...set] });
  }

  function learn() {
    waiting = true;
    const target: Target = chosen;
    const rel = relative && asKnob;
    stopLearning = controller.learn((source) => {
      waiting = false;
      stopLearning = null;
      const b: Binding = rel ? { source, target, relative: true } : { source, target };
      const keep = current.controllerBindings.filter((x) => !(x.source.kind === source.kind && x.source.channel === source.channel && x.source.number === source.number));
      save({ controllerBindings: [...keep, b] });
    });
  }

  function stopWaiting() {
    stopLearning?.();
    stopLearning = null;
    waiting = false;
  }

  function remove(i: number) {
    save({ controllerBindings: current.controllerBindings.filter((_, k) => k !== i) });
  }

  function close() {
    stopWaiting();
    onClose();
  }

  onMount(() => {
    void refresh();
    const t = setInterval(() => {
      const m = controller.lastMessage;
      if (m) seen = `${m.kind === "bend" ? "Fader" : m.kind === "note" ? "Button" : "Knob"} ${m.kind === "bend" ? "" : m.number + " "}on channel ${m.channel + 1}, value ${m.value} (${m.port})`;
    }, 200);
    return () => {
      clearInterval(t);
      stopWaiting();
    };
  });
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && close()} />

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="backdrop" onclick={close}>
  <div class="dialog" role="dialog" tabindex="-1" aria-modal="true" aria-labelledby="ctl-title" onclick={(e) => e.stopPropagation()}>
    <h1 id="ctl-title">Controllers</h1>
    <p class="fine">Use a MIDI fader box or control surface (a Behringer BCF2000, a Mackie Control and the like) to run the faders, pans, mute, solo, arm and transport. Nothing is sent to the device. Motorized faders do not follow the screen yet.</p>

    <h2>Devices</h2>
    <div class="ports">
      {#each ports as p (p)}
        <label><input type="checkbox" checked={current.controllerPorts.includes(p)} onchange={(e) => togglePort(p, e.currentTarget.checked)} /> {p}</label>
      {:else}
        <span class="fine">{looked ? "No MIDI inputs found. Connect the device, then press Refresh." : "Looking..."}</span>
      {/each}
      <button type="button" onclick={refresh}>Refresh</button>
    </div>
    <p class="fine">Last message: {seen}</p>

    <h2>What the controls do</h2>
    <div class="row">
      <button type="button" onclick={() => save({ controllerBindings: mackiePreset() })} title="Replaces the list with the standard Mackie Control layout: eight faders, pan knobs, arm, solo and mute buttons, bank buttons and transport">Use Mackie Control Layout</button>
      <button type="button" onclick={() => save({ controllerBindings: [] })} disabled={current.controllerBindings.length === 0}>Clear All</button>
      <span class="fine">{current.controllerBindings.length} control{current.controllerBindings.length === 1 ? "" : "s"} set</span>
    </div>
    <div class="list">
      {#each current.controllerBindings as b, i (i)}
        <div class="item">
          <span class="src">{sourceName(b.source)}</span>
          <span class="arrow">runs</span>
          <span class="tgt">{targetName(b.target)}{b.relative ? " (turns)" : ""}</span>
          <button type="button" aria-label="Remove" onclick={() => remove(i)}>Remove</button>
        </div>
      {:else}
        <span class="fine">Nothing set. Use the Mackie layout, or add controls one at a time below.</span>
      {/each}
    </div>

    <h2>Add one</h2>
    <div class="row">
      <select aria-label="What it should run" bind:value={picked} disabled={waiting}>
        {#each choices as t, i (i)}<option value={i}>{targetName(t)}</option>{/each}
      </select>
      {#if asKnob}
        <label title="Check this for an endless knob that sends how far it turned"><input type="checkbox" bind:checked={relative} disabled={waiting} /> Endless knob</label>
      {/if}
      {#if waiting}
        <span class="learning">Move or press a control on your device...</span>
        <button type="button" onclick={stopWaiting}>Cancel</button>
      {:else}
        <button type="button" onclick={learn}>Learn</button>
      {/if}
    </div>

    <div class="actions"><button type="button" class="primary" onclick={close}>Done</button></div>
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
    width: min(640px, 94vw);
    max-height: 90vh;
    padding: 18px 22px 14px;
    overflow-y: auto;
    border: 1px solid var(--border-strong);
    background: var(--surface-2);
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.6);
  }
  h1 {
    margin: 0 0 6px;
    font-size: 17px;
  }
  h2 {
    margin: 14px 0 6px;
    font-size: 13px;
  }
  .fine {
    margin: 0 0 6px;
    color: var(--text-dim);
    font-size: 12px;
  }
  .ports,
  .row {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
    align-items: center;
  }
  .list {
    max-height: 220px;
    margin-top: 8px;
    overflow-y: auto;
    border: 1px solid var(--border);
  }
  .item {
    display: grid;
    grid-template-columns: 1fr auto 1fr auto;
    gap: 8px;
    align-items: center;
    padding: 3px 8px;
    border-bottom: 1px solid var(--border);
  }
  .arrow {
    color: var(--text-dim);
    font-size: 12px;
  }
  .learning {
    color: var(--accent);
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    margin-top: 14px;
  }
</style>
