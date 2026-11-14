<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Properties: the selected clip's details, and collapsible sections for
  Stretch, Remix and Source Channels, laid out as collapsible sections: a
  header with its state on the right, the settings under it. Remix sets
  a target length for a music clip and joins it at musical points (the
  Fit Music engine); Clip > Time Stretch and Clip > Fit Music to Length
  open these sections.
  Stretch makes a clip play longer or shorter; pitch moves it up or down
  in semitones without changing its length. Source Channels sets how
  much of each of the file's channels goes to the left and right. The type picks how: a
  single voice or instrument, music with several notes at once, or
  varispeed, where the pitch follows the speed like tape. Live and
  Rendered sound the same here; Rendered is kept so files from other
  editors keep their setting. With several clips selected, a change
  applies to all of them. Every change is one undoable step.
-->
<script lang="ts">
  import { editor, type EditorState } from "../../stores/editor";
  import { clipLength, type ClipChange, type ClipDoc, type SessionDoc, type SessionStore } from "../../stores/session";
  import { formatIn } from "../editor/timeline";
  import { naturalLength, parseSeconds } from "../editor/stretch";
  import { describeJoins, describePlan, sameLength, type FitPlan } from "../editor/musicfit";
  import { formatSeconds, planDuration } from "./remix";
  import { isFresh, sectionRequests, type PropertiesSection } from "./focus";
  import { untrack } from "svelte";
  import { changeFor, settingsOf, varispeedSemitones, type StretchSettings } from "./clip-props";
  import { channelLabels, mixOf, presetFor, presets } from "./channel-mix";
  import type { ChannelMixDoc } from "../../stores/session";

  interface Props {
    session: SessionStore;
    onEdit: (label: string, changes: ClipChange[]) => Promise<void>;
    onError?: (e: unknown) => void;
    /** Plans for a clip: where to cut or repeat it to reach a length. Leave out to hide Remix. */
    onFitPlan?: (clipId: string, seconds: number) => Promise<FitPlan[]>;
    onFitApply?: (clipId: string, plan: FitPlan) => Promise<void>;
    /** Stretch a clip to a length, optionally moving it onto the time selection. */
    onStretchTo?: (clipId: string, seconds: number, keepPitch: boolean, toSelection: boolean) => Promise<void>;
    /** Seconds of the time selection, or null. */
    selectionSeconds?: () => number | null;
  }

  const { session, onEdit, onError = console.error, onFitPlan, onFitApply, onStretchTo, selectionSeconds = () => null }: Props = $props();

  let doc = $state<SessionDoc | null>(null);
  $effect(() => session.subscribe((d) => (doc = d)));
  let ed = $state<EditorState>(editor.current());
  $effect(() => editor.subscribe((s) => (ed = s)));

  const picked = $derived.by(() => {
    const out: { clip: ClipDoc; track: string }[] = [];
    for (const t of doc?.tracks ?? []) for (const c of t.clips) if (ed.selectedClips.has(c.id)) out.push({ clip: c, track: t.name });
    return out;
  });
  const first = $derived(picked[0] ?? null);
  const rate = $derived(doc?.sample_rate ?? 48_000);
  const media = $derived(first ? doc?.media.find((m) => m.id === first.clip.media_id) : undefined);
  const t = (s: number) => formatIn(s, rate, ed.timeFormat);
  const st = $derived<StretchSettings | null>(first ? settingsOf(first.clip) : null);
  const looped = $derived(picked.some((p) => p.clip.looped));

  // Sections open and close; a menu command can open one.
  const sections = $state<Record<PropertiesSection, boolean>>({ stretch: true, remix: false, channels: true });
  $effect(() =>
    sectionRequests.subscribe((r) => {
      if (isFresh(r)) sections[r.section] = true;
    }),
  );
  // The time selection, read again whenever the session changes.
  const sel = $derived.by(() => {
    void doc;
    return selectionSeconds();
  });

  // Remix: a target length, the plans found for it, and the one picked.
  const curSecs = $derived(first ? clipLength(first.clip, media, rate) / rate : 0);
  const origin = $derived(first ? first.clip.source_in / (media?.sample_rate || rate) : 0);
  let remixFor = $state<string | null>(null);
  let remixText = $state("");
  let plans = $state<FitPlan[]>([]);
  let pick = $state(0);
  let busy = $state(false);
  let remixError = $state<string | null>(null);
  let planned = $state<number | null>(null);
  const target = $derived(parseSeconds(remixText));
  const same = $derived(target !== null && sameLength(target, curSecs));
  const chosen = $derived(plans[pick] ?? null);
  // A new length needs a new plan.
  const stale = $derived(chosen !== null && planned !== target);
  const remixable = $derived(picked.length === 1 && !!first && !first.clip.looped && !first.clip.locked);
  // A different clip starts with its own length and no plans.
  $effect(() => {
    const id = first?.clip.id ?? null;
    if (id === untrack(() => remixFor)) return;
    remixFor = id;
    remixText = formatSeconds(untrack(() => curSecs));
    plans = [];
    pick = 0;
    planned = null;
    remixError = null;
  });

  async function findJoins() {
    const c = first;
    if (!c || !onFitPlan || target === null || same || busy) return;
    busy = true;
    remixError = null;
    plans = [];
    pick = 0;
    try {
      plans = await onFitPlan(c.clip.id, target);
      planned = target;
    } catch (e) {
      remixError = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }
  async function applyRemix() {
    const c = first;
    const p = chosen;
    if (!c || !p || stale || !onFitApply) return;
    try {
      await onFitApply(c.clip.id, p);
    } catch (e) {
      onError(e);
    }
    plans = [];
    planned = null;
  }
  async function fitToSelection() {
    const c = first;
    if (!c || !st || sel === null || !onStretchTo) return;
    await onStretchTo(c.clip.id, sel, st.type !== "varispeed", true).catch(onError);
  }
  const modeText = (m: string) => (m === "live" ? "Real-time" : m === "rendered" ? "Rendered" : "Off");

  async function apply(patch: Partial<StretchSettings>, label = "Clip Stretch") {
    const d = doc;
    if (!d || !st) return;
    const next: StretchSettings = { ...st, ...patch, advanced: { ...st.advanced, ...(patch.advanced ?? {}) } };
    // Turning on a pitch or a length from Off starts a live stretch.
    if (st.mode === "off" && patch.mode === undefined && (patch.percent !== undefined || patch.pitch !== undefined || patch.type !== undefined)) next.mode = "live";
    const changes = picked.flatMap(({ clip }) => {
      const m = d.media.find((x) => x.id === clip.media_id);
      const ch = changeFor(clip, m, d.sample_rate, { ...settingsOf(clip), ...next, percent: next.percent, advanced: next.advanced });
      return ch ? [ch] : [];
    });
    if (changes.length) await onEdit(label, changes).catch(onError);
  }

  const num = (e: Event) => Number((e.currentTarget as HTMLInputElement).value);
  function setPercent(e: Event) {
    const v = num(e);
    if (Number.isFinite(v) && v > 0) void apply({ percent: Math.max(25, Math.min(400, v)) });
  }
  function setPitch(e: Event) {
    const v = num(e);
    if (Number.isFinite(v)) void apply({ pitch: Math.max(-24, Math.min(24, v)) });
  }
  // Source Channels: one setting for every selected clip whose file has the same channels.
  const chans = $derived(media?.channels ?? 0);
  const sameChans = $derived(picked.every((p) => doc?.media.find((m) => m.id === p.clip.media_id)?.channels === chans));
  const mix = $derived(first && chans > 0 ? mixOf(first.clip.channel_mix, chans) : null);
  const names = $derived(channelLabels(chans));
  const quick = $derived(presets(chans));
  const current = $derived(mix ? presetFor(mix, chans) : null);
  const custom = $derived(picked.some((p) => p.clip.channel_mix));
  // The section shows only for files of more than two channels; mono and stereo clips look as before.
  const multi = $derived(picked.some((p) => (doc?.media.find((m) => m.id === p.clip.media_id)?.channels ?? 0) > 2));

  function setMix(next: ChannelMixDoc | null, label = "Source Channels") {
    const changes: ClipChange[] = picked.map(({ clip }) => (next ? { id: clip.id, channel_mix: next } : { id: clip.id, clear_channel_mix: true }));
    if (changes.length) void onEdit(label, changes).catch(onError);
  }
  function setSend(k: number, side: "left" | "right", pct: number) {
    if (!mix || !Number.isFinite(pct)) return;
    const sends = mix.sends.map((s) => ({ ...s }));
    sends[k]![side] = Math.max(-100, Math.min(100, pct)) / 100;
    setMix({ sends });
  }
  function invert(k: number) {
    if (!mix) return;
    setMix({ sends: mix.sends.map((s, i) => (i === k ? { left: -s.left, right: -s.right } : { ...s })) });
  }
  const pct = (v: number) => Math.round(v * 1000) / 10;

  const tone = (v: number) => `${v > 0 ? "+" : ""}${(Math.round(v * 100) / 100).toFixed(2)}`;
</script>

<section class="props" aria-label="Properties">
  {#if !first || !st}
    <p class="empty">Select a clip in the Multitrack editor to see its properties.</p>
  {:else}
    <header>
      <div class="name" title={first.clip.name}>{first.clip.name}</div>
      <div class="sub">{picked.length > 1 ? `${picked.length} clips selected; changes apply to all of them` : first.track}</div>
    </header>
    <dl class="facts">
      <dt>Start</dt><dd>{t(first.clip.start)}</dd>
      <dt>Length</dt><dd>{t(clipLength(first.clip, media, rate))}</dd>
      <dt>Natural length</dt><dd>{t(naturalLength(first.clip, media, rate))}</dd>
      <dt>File</dt><dd title={media?.path ?? ""}>{media?.path.split(/[\\/]/).pop() ?? "Missing"}</dd>
    </dl>

    {@render head("stretch", "Stretch", looped ? "Off" : modeText(st.mode))}
    {#if !sections.stretch}
      <!-- collapsed -->
    {:else if looped}
      <p class="note">Looped clips cannot stretch. Turn off Loop Clip first.</p>
    {:else}
      <div class="grid">
        <label for="pp-mode">Mode</label>
        <select id="pp-mode" value={st.mode} onchange={(e) => void apply({ mode: e.currentTarget.value as StretchSettings["mode"] })}>
          <option value="off">Off</option>
          <option value="live">Real-time</option>
          <option value="rendered">Rendered (HQ)</option>
        </select>

        <label for="pp-type">Type</label>
        <select id="pp-type" value={st.type} onchange={(e) => void apply({ type: e.currentTarget.value as StretchSettings["type"] })}>
          <option value="mono" title="One voice or instrument at a time: speech, a lead vocal">Single voice</option>
          <option value="poly" title="Several notes at once: music, chords, a full mix">Several notes</option>
          <option value="varispeed" title="Pitch follows speed, like tape running faster or slower">Varispeed</option>
        </select>

        <label for="pp-pct">Stretch</label>
        <span class="field">
          <input id="pp-pct" type="number" min="25" max="400" step="0.1" value={st.percent} onchange={setPercent} />
          <span class="unit">%</span>
        </span>

        <label for="pp-pitch">Pitch</label>
        {#if st.type === "varispeed"}
          <span class="note" title="Varispeed sets the pitch by the speed">{tone(varispeedSemitones(st.percent / 100))} semitones (follows speed)</span>
        {:else}
          <span class="field">
            <input id="pp-pitch" type="number" min="-24" max="24" step="0.1" value={st.pitch} onchange={setPitch} />
            <span class="unit">semitones</span>
          </span>
        {/if}
      </div>
      {#if st.mode !== "off"}
        <div class="quick">
          <button type="button" onclick={() => void apply({ pitch: st.pitch - 1 })} disabled={st.type === "varispeed"}>-1 st</button>
          <button type="button" onclick={() => void apply({ pitch: st.pitch + 1 })} disabled={st.type === "varispeed"}>+1 st</button>
          <button type="button" onclick={() => void apply({ percent: 100, pitch: 0 })}>Reset</button>
        </div>
      {/if}
      {#if sel !== null && picked.length === 1 && onStretchTo && !first.clip.locked}
        <div class="quick">
          <button type="button" title="Stretch the clip to the time selection and move it there" onclick={() => void fitToSelection()}>Fit to Time Selection ({formatSeconds(sel)} s)</button>
        </div>
      {/if}

      <details>
        <summary>Advanced</summary>
        <div class="grid">
          <label for="pp-win" title="Length of each analysis frame. Longer suits music, shorter a single voice.">Window</label>
          <span class="field">
            <input id="pp-win" type="number" min="5" max="200" step="1" value={st.advanced.window_ms} onchange={(e) => { const v = num(e); if (Number.isFinite(v)) void apply({ advanced: { ...st.advanced, window_ms: Math.max(5, Math.min(200, v)) } }); }} />
            <span class="unit">ms</span>
          </span>
          <label for="pp-trans" title="How readily a sharp attack starts a new frame. Stored for files from other editors.">Transients</label>
          <input id="pp-trans" type="number" min="0" max="100" step="1" value={st.advanced.transient_sensitivity} onchange={(e) => { const v = num(e); if (Number.isFinite(v)) void apply({ advanced: { ...st.advanced, transient_sensitivity: Math.max(0, Math.min(100, v)) } }); }} />
          <span></span>
          <label class="check"><input type="checkbox" checked={st.advanced.preserve_formants} onchange={(e) => void apply({ advanced: { ...st.advanced, preserve_formants: e.currentTarget.checked } })} /> Keep voice character</label>
          <span></span>
          <label class="check"><input type="checkbox" checked={st.advanced.high_precision} onchange={(e) => void apply({ advanced: { ...st.advanced, high_precision: e.currentTarget.checked } })} /> High precision</label>
        </div>
        <p class="note">The window changes the sound now; the rest are kept with the clip for files that use them.</p>
      </details>
    {/if}

    {#if onFitPlan && onFitApply}
      {@render head("remix", "Remix", chosen && !stale ? `${formatSeconds(planDuration(chosen))} s` : "Off")}
      {#if sections.remix}
        {#if picked.length > 1}
          <p class="note">Select one clip to remix.</p>
        {:else if first.clip.looped}
          <p class="note">Looped clips cannot be remixed. Turn off Loop Clip first.</p>
        {:else if first.clip.locked}
          <p class="note">The clip is locked. Unlock it first (Clip > Lock Position).</p>
        {:else}
          <p class="note">Makes a music clip an exact length by cutting out or repeating whole sections where the music lines up, so the speed and pitch stay the same.</p>
          <div class="grid">
            <span class="lbl">Current Duration</span>
            <span class="val">{formatSeconds(curSecs)} s</span>
            <label for="pp-rt">Target Duration</label>
            <span class="field">
              <input id="pp-rt" class="target" type="text" spellcheck="false" aria-label="Target length in seconds, or minutes:seconds" bind:value={remixText} onkeydown={(e) => e.key === "Enter" && void findJoins()} />
              <span class="unit">s</span>
            </span>
          </div>
          <div class="quick">
            {#each [15, 30, 60] as q (q)}<button type="button" onclick={() => (remixText = formatSeconds(q))}>{q} s</button>{/each}
            {#if sel !== null}<button type="button" title="Use the length of the time selection" onclick={() => (remixText = formatSeconds(sel))}>Selection</button>{/if}
          </div>
          {#if target === null}<p class="warn">Type a length like 30 or 1:15.5.</p>{:else if same}<p class="note">That is already this clip's length. Type the length you want.</p>{/if}
          <div class="quick">
            <button type="button" disabled={target === null || same || busy} onclick={() => void findJoins()}>Find Joins</button>
          </div>
          {#if busy}
            <p class="note">Listening for the beat and the sections…</p>
          {:else if remixError}
            <p class="warn">{remixError}</p>
          {:else if chosen}
            {#if stale}<p class="note">The length changed: find joins again.</p>{/if}
            <div class="plans" role="radiogroup" aria-label="Plans" class:dim={stale}>
              {#each plans as p, k (k)}
                <label class="plan">
                  <input type="radio" name="pp-plan" value={k} bind:group={pick} disabled={stale} />
                  <span>
                    <strong>{k === 0 ? "Best" : `Option ${k + 1}`}</strong>: {describePlan(p)}
                    <span class="joins">{describeJoins(p, origin)}</span>
                  </span>
                </label>
              {/each}
            </div>
            <div class="grid">
              <span class="lbl">Remix Duration</span>
              <span class="val">{formatSeconds(planDuration(chosen))} s</span>
            </div>
            {#if chosen.quality < 0.8 && !stale}<p class="warn">The join is not a close match and may be heard. A length a few seconds either way, or one that fits whole bars, often joins cleaner.</p>{/if}
            <div class="quick">
              <button type="button" class="main" disabled={stale || busy} onclick={() => void applyRemix()}>Apply Remix</button>
            </div>
            <p class="note">Apply puts the pieces on the track, joined with short crossfades, as one undo.</p>
          {/if}
        {/if}
      {/if}
    {/if}

    {#if multi}
    {@render head("channels", "Source Channels", `${chans} channels`)}
    {#if !sections.channels}
      <!-- collapsed -->
    {:else if !mix}
      <p class="note">The clip's file is missing.</p>
    {:else if !sameChans}
      <p class="note">The selected clips' files have different channels. Select clips of one kind to set their Source Channels.</p>
    {:else}
      <p class="note">{chans} channel{chans === 1 ? "" : "s"}. How much of each goes to the track's left and right. Negative flips polarity.</p>
      <div class="grid">
        <label for="pp-mixp">Preset</label>
        <select id="pp-mixp" value={current ?? ""} onchange={(e) => { const p = quick.find((x) => x.id === e.currentTarget.value); if (p) setMix(p.id === "default" ? null : p.mix, `Source Channels: ${p.label}`); }}>
          {#if !current}<option value="" disabled>Custom</option>{/if}
          {#each quick as p (p.id)}<option value={p.id}>{p.label}</option>{/each}
        </select>
      </div>
      <table class="mixer">
        <thead><tr><th>Channel</th><th>Left</th><th>Right</th><th></th></tr></thead>
        <tbody>
          {#each mix.sends as s, k (k)}
            <tr>
              <th scope="row">{names[k]}</th>
              {#each ["left", "right"] as const as side (side)}
                <td>
                  <span class="send">
                    <span class="bar" class:neg={s[side] < 0} style:width="{Math.abs(s[side]) * 100}%"></span>
                    <input type="number" min="-100" max="100" step="1" aria-label="{names[k]} to {side}" value={pct(s[side])} onchange={(e) => setSend(k, side, num(e))} />
                  </span>
                </td>
              {/each}
              <td><button type="button" class="inv" class:on={s.left < 0 || s.right < 0} title="Flip this channel's polarity" aria-label="Flip {names[k]} polarity" onclick={() => invert(k)}>ø</button></td>
            </tr>
          {/each}
        </tbody>
      </table>
      <div class="quick">
        <button type="button" disabled={!custom} onclick={() => setMix(null, "Source Channels: Reset")}>Reset</button>
      </div>
    {/if}
    {/if}
  {/if}
</section>

{#snippet head(key: PropertiesSection, title: string, summary: string)}
  <button type="button" class="sec" aria-expanded={sections[key]} onclick={() => (sections[key] = !sections[key])}>
    <span class="chev" aria-hidden="true">{sections[key] ? "▾" : "▸"}</span>
    <span class="title">{title}</span>
    <span class="state">{summary}</span>
  </button>
{/snippet}

<style>
  .props {
    display: flex;
    flex-direction: column;
    gap: 8px;
    height: 100%;
    padding: 8px;
    overflow: auto;
    box-sizing: border-box;
    background: var(--surface-1);
    font-size: var(--font-size-small);
  }
  .empty,
  .note {
    margin: 0;
    color: var(--text-faint);
  }
  header .name {
    overflow: hidden;
    color: var(--text);
    font-weight: 600;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  header .sub {
    color: var(--text-dim);
  }
  .facts {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 2px 10px;
    margin: 0;
  }
  dt {
    color: var(--text-dim);
  }
  dd {
    margin: 0;
    overflow: hidden;
    font-family: var(--font-mono);
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .sec {
    display: flex;
    gap: 6px;
    align-items: center;
    width: 100%;
    height: auto;
    margin: 4px 0 0;
    padding: 6px 0 2px;
    border: 0;
    border-top: 1px solid var(--border);
    border-radius: 0;
    background: transparent;
    color: var(--text);
    font-weight: 600;
    text-align: left;
  }
  .sec .chev {
    width: 10px;
    color: var(--text-dim);
  }
  .sec .title {
    flex: 1;
  }
  .sec .state {
    color: var(--text-dim);
    font-weight: normal;
  }
  .lbl {
    color: var(--text-dim);
  }
  .val {
    font-family: var(--font-mono);
  }
  .warn {
    margin: 0;
    color: var(--danger);
  }
  input.target {
    width: 96px;
    color: var(--accent);
    font-family: var(--font-mono);
  }
  .quick {
    flex-wrap: wrap;
  }
  button.main {
    border-color: var(--accent);
    background: var(--accent);
    color: var(--on-accent, #fff);
  }
  .plans {
    display: grid;
    gap: 4px;
  }
  .plan {
    display: flex;
    gap: 6px;
    align-items: flex-start;
  }
  .plan input {
    margin: 2px 0 0;
  }
  .joins {
    display: block;
    color: var(--text-dim);
  }
  .dim {
    color: var(--text-faint);
  }
  .grid {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 6px 10px;
    align-items: center;
  }
  label {
    color: var(--text-dim);
  }
  .check {
    display: flex;
    gap: 4px;
    align-items: center;
  }
  .field {
    display: flex;
    gap: 4px;
    align-items: center;
  }
  .unit {
    color: var(--text-faint);
  }
  select,
  input[type="number"] {
    min-width: 0;
    height: 22px;
    padding: 0 4px;
    border: 1px solid var(--border-strong);
    border-radius: 2px;
    background: var(--surface-2);
    color: var(--text);
    font: inherit;
  }
  input[type="number"] {
    width: 72px;
  }
  .quick {
    display: flex;
    gap: 6px;
  }
  button {
    height: 22px;
    padding: 0 8px;
    border: 1px solid var(--border-strong);
    border-radius: 2px;
    background: var(--surface-2);
    color: var(--text);
    font: inherit;
    cursor: pointer;
  }
  button:disabled {
    opacity: 0.5;
    cursor: default;
  }
  .mixer {
    border-collapse: collapse;
    width: 100%;
  }
  .mixer th,
  .mixer td {
    padding: 1px 4px 1px 0;
    font-weight: normal;
    text-align: left;
  }
  .mixer thead th {
    color: var(--text-faint);
  }
  .mixer tbody th {
    color: var(--text-dim);
    font-family: var(--font-mono);
  }
  .send {
    position: relative;
    display: block;
  }
  .send input {
    position: relative;
    width: 100%;
    max-width: 72px;
    background: transparent;
  }
  .send .bar {
    position: absolute;
    top: 1px;
    bottom: 1px;
    left: 1px;
    max-width: calc(min(100%, 72px) - 2px);
    background: color-mix(in srgb, var(--accent) 30%, transparent);
    pointer-events: none;
  }
  .send .bar.neg {
    background: color-mix(in srgb, var(--danger) 35%, transparent);
  }
  .inv {
    width: 22px;
    padding: 0;
  }
  .inv.on {
    border-color: var(--danger);
    color: var(--danger);
  }
  details summary {
    color: var(--text-dim);
    cursor: pointer;
  }
  details .grid {
    margin-top: 6px;
  }
</style>
