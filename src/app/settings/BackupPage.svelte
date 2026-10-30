<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- Copyright (c) The GeekCast Radio Network, LLC. -->
<!-- This file is part of Altered Audio Editor. -->

<script lang="ts">
  /**
   * Settings > Backup and Restore: where settings live, a snapshot or export on demand,
   * bringing a backup back (merge keeps what is here and adds what is missing; replace
   * swaps it, after a safety snapshot), the automatic snapshots, and a sync folder.
   */
  import { onMount } from "svelte";
  import {
    backupExport,
    backupImport,
    backupInfo,
    backupInspect,
    backupPickOpen,
    backupPickSave,
    backupSetOptions,
    backupSnapshotNow,
    exitApp,
    libraryPickFolder,
    type BackupInfo,
    type BackupManifest,
    type BackupReport,
  } from "../core";
  import { sizeText } from "./pages";

  let info = $state<BackupInfo | null>(null);
  let busy = $state(false);
  let message = $state("");
  let secrets = $state(false);
  /** A backup chosen to bring in, with the categories ticked. */
  let incoming = $state<{ path: string; manifest: BackupManifest; picked: string[]; mode: "merge" | "replace" } | null>(null);
  let report = $state<BackupReport | null>(null);

  const categories = $derived(incoming ? [...new Set(incoming.manifest.entries.map((e) => e.category))] : []);

  async function refresh() {
    info = await backupInfo();
  }

  onMount(() => {
    void refresh().catch((e) => (message = String(e)));
  });

  async function run(task: () => Promise<void>) {
    busy = true;
    message = "";
    try {
      await task();
    } catch (e) {
      message = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }

  const snapshotNow = () =>
    run(async () => {
      const made = await backupSnapshotNow();
      message = made ? "Snapshot saved." : "Nothing to back up yet.";
      await refresh();
    });

  const exportAll = () =>
    run(async () => {
      const dest = await backupPickSave();
      if (!dest) return;
      const m = await backupExport(dest, secrets);
      message = `Saved ${m.entries.length} files${m.includesSecrets ? " (including keys and tokens)" : ""}.`;
      await refresh();
    });

  const choose = (path: string | null) =>
    run(async () => {
      if (!path) return;
      const manifest = await backupInspect(path);
      report = null;
      incoming = { path, manifest, picked: [...new Set(manifest.entries.map((e) => e.category))], mode: "merge" };
    });

  const pickBackup = async () => choose(await backupPickOpen());

  const doImport = () =>
    run(async () => {
      if (!incoming) return;
      report = await backupImport(incoming.path, incoming.mode, incoming.picked);
      incoming = null;
      await refresh();
    });

  const syncFolder = (clear: boolean) =>
    run(async () => {
      if (!info) return;
      const folder = clear ? null : await libraryPickFolder("Folder to copy backups into");
      if (!clear && !folder) return;
      await backupSetOptions({ ...info.options, syncFolder: folder });
      await refresh();
    });

  const setKeep = (n: number) =>
    run(async () => {
      if (!info) return;
      await backupSetOptions({ ...info.options, keep: Math.min(100, Math.max(1, Math.round(n) || 10)) });
      await refresh();
    });

  const reasonText: Record<string, string> = {
    launch: "On start-up",
    "before-upgrade": "Before an upgrade",
    "before-restore": "Before a restore",
    manual: "Made by you",
  };
  const countText = (r: BackupReport) =>
    [
      r.added.length && `${r.added.length} added`,
      r.merged.length && `${r.merged.length} merged`,
      r.replaced.length && `${r.replaced.length} replaced`,
      r.renamed.length && `${r.renamed.length} kept beside yours as "(restored)"`,
      r.unchanged.length && `${r.unchanged.length} already the same`,
    ]
      .filter(Boolean)
      .join(", ") || "Nothing needed changing";
</script>

<div class="page">
  {#if info}
    <p class="note">
      Your settings, presets, shortcuts, layouts and templates live in <code>{info.configDir}</code>.
      {#if info.portable}Portable mode is on: they stay beside the program.{:else}An uninstall or an upgrade leaves that folder alone.{/if}
    </p>
    <div class="row">
      <button type="button" disabled={busy} onclick={() => void snapshotNow()}>Back Up Now</button>
      <button type="button" disabled={busy} onclick={() => void exportAll()}>Export Everything...</button>
      <button type="button" disabled={busy} onclick={() => void pickBackup()}>Import a Backup...</button>
    </div>
    <label class="check"><input type="checkbox" bind:checked={secrets} /> Include keys and tokens when exporting <span class="why">Leave this off for a backup you will share.</span></label>

    {#if incoming}
      <fieldset class="box">
        <legend>Bring in {incoming.manifest.created} (version {incoming.manifest.appVersion})</legend>
        {#each categories as c (c)}
          <label class="check">
            <input
              type="checkbox"
              checked={incoming.picked.includes(c)}
              onchange={(e) => {
                if (incoming) incoming.picked = e.currentTarget.checked ? [...incoming.picked, c] : incoming.picked.filter((x) => x !== c);
              }}
            />
            {c}
            <span class="why">{incoming.manifest.entries.filter((x) => x.category === c).length} files</span>
          </label>
        {/each}
        <label class="check"><input type="radio" name="mode" value="merge" bind:group={incoming.mode} /> Merge: keep what I have, add what is missing <span class="why">Safe. A file that differs comes in renamed.</span></label>
        <label class="check"><input type="radio" name="mode" value="replace" bind:group={incoming.mode} /> Replace: use the backup's version <span class="why">A snapshot of your current settings is taken first.</span></label>
        <div class="row">
          <button type="button" disabled={busy || incoming.picked.length === 0} onclick={() => void doImport()}>Bring In</button>
          <button type="button" disabled={busy} onclick={() => (incoming = null)}>Cancel</button>
        </div>
      </fieldset>
    {/if}

    {#if report}
      <p class="note">Done: {countText(report)}. Restart the app to use them.</p>
      <div class="row"><button type="button" onclick={() => void exitApp()}>Quit Now</button></div>
    {/if}

    <h4>Automatic snapshots</h4>
    <p class="note">
      A snapshot is saved on start-up whenever something changed, and always before the first start of a new version. Backups live in <code>{info.backupsDir}</code>.
    </p>
    <div class="row">
      <label>Keep the newest <input class="num" type="number" min="1" max="100" value={info.options.keep} onchange={(e) => void setKeep(Number(e.currentTarget.value))} /></label>
      <span class="why">Backups you export by hand are never removed.</span>
    </div>
    <div class="row">
      <span>Also copy snapshots to: <code>{info.options.syncFolder ?? "nowhere"}</code></span>
      <button type="button" disabled={busy} onclick={() => void syncFolder(false)}>Choose Folder...</button>
      {#if info.options.syncFolder}<button type="button" disabled={busy} onclick={() => void syncFolder(true)}>Stop</button>{/if}
      <span class="why">A Dropbox, OneDrive or network folder protects you from a lost drive.</span>
    </div>
    {#if info.snapshots.length}
      <table>
        <tbody>
          {#each info.snapshots as s (s.path)}
            <tr>
              <td>{s.created}</td>
              <td>{reasonText[s.reason] ?? s.reason}</td>
              <td>v{s.appVersion}</td>
              <td class="num">{sizeText(s.bytes)}</td>
              <td><button type="button" disabled={busy} onclick={() => void choose(s.path)}>Restore...</button></td>
            </tr>
          {/each}
        </tbody>
      </table>
    {:else}
      <p class="note">No snapshots yet.</p>
    {/if}
  {:else}
    <p class="note">Reading your backups…</p>
  {/if}
  {#if message}<p class="msg">{message}</p>{/if}
</div>

<style>
  .page { display: flex; flex-direction: column; gap: 10px; }
  .note, .why { color: var(--muted, #9aa0aa); font-size: 12px; }
  .row { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; }
  .check { display: flex; gap: 6px; align-items: baseline; flex-wrap: wrap; }
  .box { border: 1px solid var(--border, #444); border-radius: 4px; padding: 8px 10px; display: flex; flex-direction: column; gap: 6px; }
  .msg { color: var(--accent, #6aa9ff); }
  h4 { margin: 8px 0 0; }
  code { word-break: break-all; }
  table { border-collapse: collapse; width: 100%; font-size: 12px; }
  td { padding: 3px 6px; border-bottom: 1px solid var(--border, #333); }
  .num { text-align: right; font-variant-numeric: tabular-nums; }
  input.num { width: 56px; }
</style>
