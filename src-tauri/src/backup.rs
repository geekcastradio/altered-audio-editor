// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Backing up and restoring the app's own settings: preferences and
//! presets, shortcuts, workspaces, templates and the rest of the config
//! folder. A backup is a `.aaebackup` file, a zip with a `manifest.json`
//! first and the files under their relative names.
//!
//! The rules, so a new version never costs anyone their settings:
//! - restoring in Merge mode keeps what is already there and adds what is
//!   missing; a file that differs under the same name comes in renamed, never
//!   overwritten;
//! - Replace mode is only used after the caller has taken a snapshot;
//! - keys and files this version does not know are carried along untouched;
//! - tokens and keys (names containing `token`, `secret`, `password`,
//!   `apikey`) are left out unless asked for;
//! - snapshots are taken on a launch where something changed, and always
//!   before the first launch of a new version; the newest few are kept.

use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

pub const FORMAT: &str = "altaudio.backup";
pub const VERSION: u32 = 1;
pub const EXT: &str = "aaebackup";
const MANIFEST: &str = "manifest.json";
const MAX_FILE: u64 = 64 << 20;
/// Folders that are never backed up: working caches and logs, and the backups themselves.
const SKIP_DIRS: [&str; 6] = ["Backups", "EBWebView", "cache", "Cache", "logs", "Logs"];
/// Files that only make sense on this computer.
const SKIP_FILES: [&str; 2] = ["cache-folder.txt", "last-version.txt"];
const SECRET_WORDS: [&str; 5] = ["token", "secret", "password", "apikey", "api-key"];
/// How many automatic snapshots to keep unless the user says otherwise.
pub const DEFAULT_KEEP: u32 = 10;
/// Options kept in the config folder, so they travel with a backup.
pub const OPTIONS_FILE: &str = "backup.json";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub path: String,
    pub category: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub format: String,
    pub version: u32,
    pub app_version: String,
    pub created: String,
    pub reason: String,
    pub signature: String,
    pub includes_secrets: bool,
    pub entries: Vec<Item>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Merge,
    Replace,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Report {
    pub added: Vec<String>,
    pub merged: Vec<String>,
    pub replaced: Vec<String>,
    pub renamed: Vec<String>,
    pub unchanged: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotInfo {
    pub path: PathBuf,
    pub name: String,
    pub bytes: u64,
    pub created: String,
    pub reason: String,
    pub app_version: String,
}

/// The user's backup choices.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Options {
    pub sync_folder: Option<String>,
    pub keep: u32,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            sync_folder: None,
            keep: DEFAULT_KEEP,
        }
    }
}

pub fn load_options(config: &Path) -> Options {
    fs::read_to_string(config.join(OPTIONS_FILE))
        .ok()
        .and_then(|t| serde_json::from_str::<Options>(&t).ok())
        .map(|o| Options {
            keep: o.keep.clamp(1, 100),
            ..o
        })
        .unwrap_or_default()
}

pub fn save_options(config: &Path, options: &Options) -> Result<(), String> {
    fs::create_dir_all(config).map_err(|e| e.to_string())?;
    let text = serde_json::to_string_pretty(options).map_err(|e| e.to_string())?;
    write_atomic(&config.join(OPTIONS_FILE), text.as_bytes())
}

/// What a file is, for the checklist.
pub fn category(rel: &str) -> &'static str {
    let name = rel.rsplit('/').next().unwrap_or(rel);
    if rel == "preferences.alprefs" {
        "Settings and presets"
    } else if rel == "keymap.alkeys" {
        "Keyboard shortcuts"
    } else if rel.starts_with("workspaces/") || rel == "workspace.alworkspace" {
        "Workspaces and layouts"
    } else if rel.starts_with("templates/") {
        "Session templates"
    } else if name.ends_with(".alhost") || rel.starts_with("hosts/") {
        "Host profiles"
    } else {
        "Other settings"
    }
}

/// A relative path from a backup that stays inside the folder it is restored to.
pub fn safe_rel(rel: &str) -> Option<PathBuf> {
    if rel.is_empty() || rel.contains('\\') || rel.contains(':') {
        return None;
    }
    let p = Path::new(rel);
    p.components()
        .all(|c| matches!(c, Component::Normal(_)))
        .then(|| p.to_path_buf())
}

fn is_secret_key(key: &str) -> bool {
    let k = key.to_ascii_lowercase();
    SECRET_WORDS.iter().any(|w| k.contains(w))
}

fn strip_secrets(bytes: &[u8]) -> Vec<u8> {
    let Ok(mut v) = serde_json::from_slice::<Value>(bytes) else {
        return bytes.to_vec();
    };
    if let Some(values) = v.get_mut("values").and_then(Value::as_object_mut) {
        values.retain(|k, _| !is_secret_key(k));
        return serde_json::to_vec_pretty(&v).unwrap_or_else(|_| bytes.to_vec());
    }
    bytes.to_vec()
}

/// A stable fingerprint (FNV-1a) of names and contents, to tell whether anything changed.
fn signature(files: &[(String, Vec<u8>)]) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    let mut eat = |b: &[u8]| {
        for x in b {
            h ^= *x as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
    };
    for (name, bytes) in files {
        eat(name.as_bytes());
        eat(&[0]);
        eat(&(bytes.len() as u64).to_le_bytes());
        eat(bytes);
    }
    format!("{h:016x}")
}

fn walk(dir: &Path, base: &Path, out: &mut Vec<(String, PathBuf)>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let path = e.path();
        let name = e.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') || name.ends_with(".tmp") {
            continue;
        }
        if path.is_dir() {
            if !SKIP_DIRS.contains(&name.as_str()) {
                walk(&path, base, out);
            }
        } else if let Ok(rel) = path.strip_prefix(base) {
            let rel = rel.to_string_lossy().replace('\\', "/");
            if SKIP_FILES.contains(&name.as_str()) {
                continue;
            }
            if fs::metadata(&path)
                .map(|m| m.len() <= MAX_FILE)
                .unwrap_or(false)
            {
                out.push((rel, path));
            }
        }
    }
}

/// Every backed-up file in `config` with its contents, sorted by name.
pub fn collect(config: &Path, include_secrets: bool) -> Vec<(String, Vec<u8>)> {
    let mut found = Vec::new();
    walk(config, config, &mut found);
    found.sort_by(|a, b| a.0.cmp(&b.0));
    found
        .into_iter()
        .filter_map(|(rel, path)| {
            let bytes = fs::read(&path).ok()?;
            let bytes = if !include_secrets && rel == "preferences.alprefs" {
                strip_secrets(&bytes)
            } else {
                bytes
            };
            Some((rel, bytes))
        })
        .collect()
}

fn write_atomic(dest: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let tmp = dest.with_extension("tmp");
    fs::write(&tmp, bytes).map_err(|e| format!("Could not write {}: {e}", dest.display()))?;
    fs::rename(&tmp, dest).map_err(|e| format!("Could not write {}: {e}", dest.display()))
}

/// Writes a backup of `config` to `dest`.
pub fn create(
    config: &Path,
    dest: &Path,
    app_version: &str,
    reason: &str,
    include_secrets: bool,
) -> Result<Manifest, String> {
    let files = collect(config, include_secrets);
    write_backup(&files, dest, app_version, reason, include_secrets)
}

fn write_backup(
    files: &[(String, Vec<u8>)],
    dest: &Path,
    app_version: &str,
    reason: &str,
    include_secrets: bool,
) -> Result<Manifest, String> {
    let manifest = Manifest {
        format: FORMAT.into(),
        version: VERSION,
        app_version: app_version.into(),
        created: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        reason: reason.into(),
        signature: signature(files),
        includes_secrets: include_secrets,
        entries: files
            .iter()
            .map(|(p, b)| Item {
                path: p.clone(),
                category: category(p).into(),
                bytes: b.len() as u64,
            })
            .collect(),
    };
    let mut buf = Vec::new();
    {
        let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buf));
        let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        zip.start_file(MANIFEST, opts).map_err(|e| e.to_string())?;
        zip.write_all(&serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        for (rel, bytes) in files {
            zip.start_file(rel, opts).map_err(|e| e.to_string())?;
            zip.write_all(bytes).map_err(|e| e.to_string())?;
        }
        zip.finish().map_err(|e| e.to_string())?;
    }
    write_atomic(dest, &buf)?;
    Ok(manifest)
}

/// The manifest of a backup file, checked to be one of ours and not newer than this version reads.
pub fn read_manifest(path: &Path) -> Result<Manifest, String> {
    let file =
        fs::File::open(path).map_err(|e| format!("Could not open {}: {e}", path.display()))?;
    let mut zip = ZipArchive::new(std::io::BufReader::new(file))
        .map_err(|_| "That isn't a backup file.".to_string())?;
    let mut text = String::new();
    zip.by_name(MANIFEST)
        .map_err(|_| "That isn't a backup file.".to_string())?
        .read_to_string(&mut text)
        .map_err(|e| e.to_string())?;
    let m: Manifest = serde_json::from_str(&text)
        .map_err(|_| "That backup's list of files is damaged.".to_string())?;
    if m.format != FORMAT {
        return Err("That isn't a backup file.".into());
    }
    if m.version > VERSION {
        return Err(
            "That backup was made by a newer version. Update the app, then try again.".into(),
        );
    }
    Ok(m)
}

fn merge_json(mine: &[u8], theirs: &[u8]) -> Option<Vec<u8>> {
    let mut a: Value = serde_json::from_slice(mine).ok()?;
    let b: Value = serde_json::from_slice(theirs).ok()?;
    let field = ["values", "bindings"].into_iter().find(|f| {
        a.get(*f).is_some_and(Value::is_object) && b.get(*f).is_some_and(Value::is_object)
    })?;
    let theirs_map = b.get(field)?.as_object()?.clone();
    let mine_map = a.get_mut(field)?.as_object_mut()?;
    for (k, v) in theirs_map {
        mine_map.entry(k).or_insert(v);
    }
    serde_json::to_vec_pretty(&a).ok()
}

fn renamed_path(dest: &Path) -> PathBuf {
    let stem = dest
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let ext = dest
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    let mut n = 0;
    loop {
        let label = if n == 0 {
            " (restored)".to_string()
        } else {
            format!(" (restored {})", n + 1)
        };
        let p = dest.with_file_name(format!("{stem}{label}{ext}"));
        if !p.exists() {
            return p;
        }
        n += 1;
    }
}

/// Restores `src` into `config`. `categories` limits it to some of the manifest's categories.
pub fn restore(
    src: &Path,
    config: &Path,
    mode: Mode,
    categories: Option<&[String]>,
) -> Result<Report, String> {
    let manifest = read_manifest(src)?;
    let file = fs::File::open(src).map_err(|e| e.to_string())?;
    let mut zip = ZipArchive::new(std::io::BufReader::new(file)).map_err(|e| e.to_string())?;
    let mut report = Report::default();
    for item in &manifest.entries {
        if categories.is_some_and(|c| !c.iter().any(|x| x == &item.category)) {
            continue;
        }
        let Some(rel) = safe_rel(&item.path) else {
            continue;
        };
        let mut bytes = Vec::new();
        zip.by_name(&item.path)
            .map_err(|e| format!("{} is missing from the backup: {e}", item.path))?
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        let dest = config.join(&rel);
        let name = item.path.clone();
        if !dest.exists() {
            write_atomic(&dest, &bytes)?;
            report.added.push(name);
            continue;
        }
        let current = fs::read(&dest).unwrap_or_default();
        if current == bytes {
            report.unchanged.push(name);
        } else if mode == Mode::Replace {
            write_atomic(&dest, &bytes)?;
            report.replaced.push(name);
        } else if let Some(merged) = merge_json(&current, &bytes) {
            if merged == current {
                report.unchanged.push(name);
            } else {
                write_atomic(&dest, &merged)?;
                report.merged.push(name);
            }
        } else {
            write_atomic(&renamed_path(&dest), &bytes)?;
            report.renamed.push(name);
        }
    }
    Ok(report)
}

fn stamp() -> String {
    chrono::Local::now().format("%Y%m%d-%H%M%S").to_string()
}

/// Every backup file in `dir`, newest first.
pub fn list(dir: &Path) -> Vec<SnapshotInfo> {
    let Ok(rd) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out: Vec<SnapshotInfo> = rd
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == EXT))
        .filter_map(|e| {
            let path = e.path();
            let m = read_manifest(&path).ok()?;
            Some(SnapshotInfo {
                name: path.file_name()?.to_string_lossy().into_owned(),
                bytes: e.metadata().ok()?.len(),
                created: m.created,
                reason: m.reason,
                app_version: m.app_version,
                path,
            })
        })
        .collect();
    out.sort_by(|a, b| b.name.cmp(&a.name));
    out
}

/// Takes a snapshot of `config` into `dir`. Without `force`, nothing is written when the
/// newest snapshot already holds exactly the same settings. Old automatic snapshots beyond
/// `keep` are removed. Copies go to `sync` too. Returns the new file, if one was made.
pub fn snapshot(
    config: &Path,
    dir: &Path,
    app_version: &str,
    reason: &str,
    force: bool,
    options: &Options,
) -> Result<Option<PathBuf>, String> {
    let files = collect(config, false);
    if files.is_empty() {
        return Ok(None);
    }
    if !force {
        let sig = signature(&files);
        if list(dir)
            .first()
            .and_then(|s| read_manifest(&s.path).ok())
            .is_some_and(|m| m.signature == sig)
        {
            return Ok(None);
        }
    }
    let name = format!("auto-{}-{}.{EXT}", stamp(), reason);
    let dest = dir.join(&name);
    write_backup(&files, &dest, app_version, reason, false)?;
    if let Some(sync) = options
        .sync_folder
        .as_deref()
        .filter(|s| !s.trim().is_empty())
    {
        // A sync folder that is missing or read-only must never stop the app.
        let _ = fs::create_dir_all(sync)
            .and_then(|_| fs::copy(&dest, Path::new(sync).join(&name)).map(|_| ()));
    }
    prune(dir, options.keep);
    Ok(Some(dest))
}

/// Keeps the newest `keep` automatic snapshots. Backups the user exported by hand are never removed.
pub fn prune(dir: &Path, keep: u32) {
    let autos: Vec<SnapshotInfo> = list(dir)
        .into_iter()
        .filter(|s| s.name.starts_with("auto-"))
        .collect();
    for old in autos.into_iter().skip(keep as usize) {
        let _ = fs::remove_file(old.path);
    }
}

/// Runs at every launch: a snapshot before the first launch of a new version, else one if
/// anything changed since the last. Never fails the launch.
pub fn on_launch(config: &Path, dir: &Path, app_version: &str) {
    let marker = config.join("last-version.txt");
    let previous = fs::read_to_string(&marker)
        .ok()
        .map(|s| s.trim().to_string());
    let upgrading = previous.as_deref().is_some_and(|p| p != app_version);
    let options = load_options(config);
    let _ = if upgrading {
        snapshot(
            config,
            dir,
            previous.as_deref().unwrap_or(app_version),
            "before-upgrade",
            true,
            &options,
        )
    } else {
        snapshot(config, dir, app_version, "launch", false, &options)
    };
    let _ = fs::create_dir_all(config).and_then(|_| fs::write(&marker, app_version));
}

/// Counts used by the tests and the settings page.
pub fn by_category(m: &Manifest) -> BTreeMap<String, usize> {
    let mut out = BTreeMap::new();
    for e in &m.entries {
        *out.entry(e.category.clone()).or_insert(0) += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> tempfile::TempDir {
        let d = tempfile::tempdir().unwrap();
        let p = d.path();
        fs::write(p.join("preferences.alprefs"), r#"{"format":"altaudio.preferences","schema_version":1,"values":{"a":1,"rss.token":"SECRET","voiceChain.presets":[1]}}"#).unwrap();
        fs::write(p.join("keymap.alkeys"), r#"{"format":"altaudio.keymap","schema_version":1,"bindings":{"edit.undo":["Ctrl+Z"]}}"#).unwrap();
        fs::create_dir_all(p.join("workspaces")).unwrap();
        fs::write(p.join("workspaces/default.alworkspace"), b"{}").unwrap();
        fs::create_dir_all(p.join("Cache")).unwrap();
        fs::write(p.join("Cache/big.bin"), b"skip me").unwrap();
        fs::write(p.join("cache-folder.txt"), b"D:\\Cache").unwrap();
        d
    }

    #[test]
    fn a_backup_holds_settings_not_caches_and_leaves_secrets_out() {
        let c = config();
        let out = tempfile::tempdir().unwrap();
        let dest = out.path().join("x.aaebackup");
        let m = create(c.path(), &dest, "0.9.5", "manual", false).unwrap();
        let names: Vec<_> = m.entries.iter().map(|e| e.path.as_str()).collect();
        assert_eq!(
            names,
            [
                "keymap.alkeys",
                "preferences.alprefs",
                "workspaces/default.alworkspace"
            ]
        );
        let back = tempfile::tempdir().unwrap();
        restore(&dest, back.path(), Mode::Merge, None).unwrap();
        let prefs = fs::read_to_string(back.path().join("preferences.alprefs")).unwrap();
        assert!(prefs.contains("voiceChain.presets") && !prefs.contains("SECRET"));
        let with = create(
            c.path(),
            &out.path().join("y.aaebackup"),
            "0.9.5",
            "manual",
            true,
        )
        .unwrap();
        assert!(with.includes_secrets);
        let back2 = tempfile::tempdir().unwrap();
        restore(
            &out.path().join("y.aaebackup"),
            back2.path(),
            Mode::Merge,
            None,
        )
        .unwrap();
        assert!(fs::read_to_string(back2.path().join("preferences.alprefs"))
            .unwrap()
            .contains("SECRET"));
    }

    #[test]
    fn merge_keeps_mine_adds_theirs_and_never_overwrites() {
        let c = config();
        let out = tempfile::tempdir().unwrap();
        let dest = out.path().join("x.aaebackup");
        create(c.path(), &dest, "0.9.5", "manual", false).unwrap();
        let target = tempfile::tempdir().unwrap();
        fs::write(target.path().join("preferences.alprefs"), r#"{"format":"altaudio.preferences","schema_version":1,"values":{"a":2,"newer.key":true}}"#).unwrap();
        fs::write(target.path().join("keymap.alkeys"), b"not json at all").unwrap();
        let r = restore(&dest, target.path(), Mode::Merge, None).unwrap();
        let prefs: Value = serde_json::from_str(
            &fs::read_to_string(target.path().join("preferences.alprefs")).unwrap(),
        )
        .unwrap();
        assert_eq!(prefs["values"]["a"], 2, "mine wins");
        assert_eq!(prefs["values"]["newer.key"], true, "unknown keys survive");
        assert_eq!(
            prefs["values"]["voiceChain.presets"],
            serde_json::json!([1]),
            "theirs added"
        );
        assert_eq!(r.merged, ["preferences.alprefs"]);
        assert_eq!(r.renamed, ["keymap.alkeys"]);
        assert_eq!(
            fs::read(target.path().join("keymap.alkeys")).unwrap(),
            b"not json at all"
        );
        assert!(target.path().join("keymap (restored).alkeys").exists());
        assert_eq!(r.added, ["workspaces/default.alworkspace"]);
    }

    #[test]
    fn replace_overwrites_and_categories_limit_what_comes_back() {
        let c = config();
        let out = tempfile::tempdir().unwrap();
        let dest = out.path().join("x.aaebackup");
        create(c.path(), &dest, "0.9.5", "manual", false).unwrap();
        let target = tempfile::tempdir().unwrap();
        fs::write(target.path().join("keymap.alkeys"), b"old").unwrap();
        let r = restore(
            &dest,
            target.path(),
            Mode::Replace,
            Some(&["Keyboard shortcuts".to_string()]),
        )
        .unwrap();
        assert_eq!(r.replaced, ["keymap.alkeys"]);
        assert!(!target.path().join("preferences.alprefs").exists());
    }

    #[test]
    fn unsafe_paths_and_newer_or_foreign_files_are_refused() {
        assert!(
            safe_rel("../x").is_none()
                && safe_rel("/x").is_none()
                && safe_rel("a\\b").is_none()
                && safe_rel("C:x").is_none()
        );
        assert!(safe_rel("workspaces/a.alworkspace").is_some());
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("odd.aaebackup");
        fs::write(&p, b"nope").unwrap();
        assert!(read_manifest(&p).is_err());
        // A manifest from a newer format version.
        let mut buf = Vec::new();
        {
            let mut z = ZipWriter::new(std::io::Cursor::new(&mut buf));
            z.start_file(MANIFEST, SimpleFileOptions::default())
                .unwrap();
            z.write_all(br#"{"format":"altaudio.backup","version":99,"appVersion":"9","created":"","reason":"","signature":"","includesSecrets":false,"entries":[]}"#).unwrap();
            z.finish().unwrap();
        }
        fs::write(&p, buf).unwrap();
        assert!(read_manifest(&p).unwrap_err().contains("newer"));
    }

    #[test]
    fn snapshots_skip_when_nothing_changed_keep_the_newest_and_copy_to_sync() {
        let c = config();
        let dir = tempfile::tempdir().unwrap();
        let sync = tempfile::tempdir().unwrap();
        let opts = Options {
            sync_folder: Some(sync.path().to_string_lossy().into_owned()),
            keep: 2,
        };
        assert!(
            snapshot(c.path(), dir.path(), "0.9.5", "launch", false, &opts)
                .unwrap()
                .is_some()
        );
        assert!(
            snapshot(c.path(), dir.path(), "0.9.5", "launch", false, &opts)
                .unwrap()
                .is_none(),
            "unchanged"
        );
        assert_eq!(fs::read_dir(sync.path()).unwrap().count(), 1);
        for i in 0..3 {
            fs::write(
                c.path().join("keymap.alkeys"),
                format!(
                    r#"{{"format":"altaudio.keymap","schema_version":1,"bindings":{{"x{i}":[]}}}}"#
                ),
            )
            .unwrap();
            std::thread::sleep(std::time::Duration::from_millis(1100));
            snapshot(c.path(), dir.path(), "0.9.5", "launch", false, &opts).unwrap();
        }
        assert_eq!(list(dir.path()).len(), 2, "only the newest two are kept");
        let manual = dir.path().join("mine.aaebackup");
        create(c.path(), &manual, "0.9.5", "manual", false).unwrap();
        prune(dir.path(), 1);
        assert!(manual.exists(), "a backup made by hand is never pruned");
    }

    #[test]
    fn a_new_version_gets_a_snapshot_first() {
        let c = config();
        let dir = tempfile::tempdir().unwrap();
        on_launch(c.path(), dir.path(), "0.9.1");
        assert_eq!(list(dir.path()).len(), 1);
        on_launch(c.path(), dir.path(), "0.9.1");
        assert_eq!(list(dir.path()).len(), 1, "same version, nothing changed");
        std::thread::sleep(std::time::Duration::from_millis(1100));
        on_launch(c.path(), dir.path(), "0.9.5");
        let l = list(dir.path());
        assert_eq!(l.len(), 2);
        assert_eq!(l[0].reason, "before-upgrade");
        assert_eq!(
            l[0].app_version, "0.9.1",
            "labelled with the version it holds"
        );
    }
}
