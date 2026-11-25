// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Copying a session's audio into its own folder, so the folder holds
//! everything the session needs. Files already anywhere inside the
//! session folder stay where they are. A file whose copy is already
//! there (same name, size and content) is not copied twice; a different
//! file with the same name gets a numbered name.

use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;

use serde::Serialize;

/// One copy job at a time. Two drops of the same file at once used to race
/// for the same half-written file name, and Windows refused the second
/// (os error 32). Now the second waits, then finds the first one's copy.
static COPY_LOCK: Mutex<()> = Mutex::new(());
static PART_COUNTER: AtomicU32 = AtomicU32::new(0);

/// What copying did, path for path.
#[derive(Debug, Clone, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CopyResult {
    /// Where each file is now, in the order given: the copy, or the
    /// original when it was already inside or could not be copied.
    pub paths: Vec<PathBuf>,
    pub copied: u32,
    /// Copies already in the folder from before.
    pub reused: u32,
    /// Files that could not be copied, with the reason.
    pub failed: Vec<String>,
}

/// The folder copies go into: `folder` (a plain name, "Audio" when empty)
/// beside the session file.
pub fn media_dir(session_file: &Path, folder: &str) -> PathBuf {
    let base = session_file.parent().unwrap_or(Path::new(""));
    let clean: String = folder
        .trim()
        .chars()
        .filter(|c| !matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|'))
        .collect();
    let clean = clean.trim_matches('.').trim();
    base.join(if clean.is_empty() { "Audio" } else { clean })
}

/// Whether `path` is inside `dir` (at any depth).
pub fn is_inside(path: &Path, dir: &Path) -> bool {
    let p = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let d = fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
    p.starts_with(&d)
}

/// Copies each of `paths` into `dir` unless it is already inside `keep`
/// (the session folder).
pub fn copy_in(paths: &[PathBuf], dir: &Path, keep: &Path) -> CopyResult {
    let _turn = COPY_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut out = CopyResult::default();
    for src in paths {
        if is_inside(src, keep) {
            out.paths.push(src.clone());
            continue;
        }
        match copy_one(src, dir) {
            Ok((to, fresh)) => {
                if fresh {
                    out.copied += 1;
                } else {
                    out.reused += 1;
                }
                out.paths.push(to);
            }
            Err(e) => {
                let name = src
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                // Say which file is missing and where it was expected, not just the system's words.
                if !src.exists() {
                    out.failed
                        .push(format!("{name}: the original is not at {}", src.display()));
                } else {
                    out.failed.push(format!("{name}: {e}"));
                }
                out.paths.push(src.clone());
            }
        }
    }
    out
}

/// Copies one file, or finds its copy already there. True when copied.
fn copy_one(src: &Path, dir: &Path) -> std::io::Result<(PathBuf, bool)> {
    let name = src
        .file_name()
        .ok_or_else(|| std::io::Error::other("no file name"))?;
    fs::create_dir_all(dir)?;
    let stem = Path::new(name)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let ext = Path::new(name)
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    for n in 1..10_000 {
        let to = if n == 1 {
            dir.join(name)
        } else {
            dir.join(format!("{stem} ({n}){ext}"))
        };
        if !to.exists() {
            // Copy beside it first, so a cut-short copy never looks finished.
            let part = to.with_extension(format!(
                "altaudio-part{}",
                PART_COUNTER.fetch_add(1, Ordering::Relaxed)
            ));
            fs::copy(src, &part)?;
            fs::rename(&part, &to)?;
            return Ok((to, true));
        }
        if same_file(src, &to)? {
            return Ok((to, false));
        }
    }
    Err(std::io::Error::other("too many files with that name"))
}

/// Same size, and the same first and last 64 KB: taken as the same file.
fn same_file(a: &Path, b: &Path) -> std::io::Result<bool> {
    let (ma, mb) = (fs::metadata(a)?, fs::metadata(b)?);
    if ma.len() != mb.len() {
        return Ok(false);
    }
    const SPAN: u64 = 64 * 1024;
    let read = |p: &Path, at: u64| -> std::io::Result<Vec<u8>> {
        let mut f = File::open(p)?;
        f.seek(SeekFrom::Start(at))?;
        let mut buf = Vec::with_capacity(SPAN as usize);
        f.take(SPAN).read_to_end(&mut buf)?;
        Ok(buf)
    };
    let tail = ma.len().saturating_sub(SPAN);
    Ok(read(a, 0)? == read(b, 0)? && read(a, tail)? == read(b, tail)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copies_once_keeps_inside_files_and_numbers_clashes() {
        let root = tempfile::tempdir().unwrap();
        let session = root.path().join("Show").join("Show.altaudio");
        fs::create_dir_all(session.parent().unwrap()).unwrap();
        let outside = root.path().join("elsewhere");
        fs::create_dir_all(&outside).unwrap();
        let a = outside.join("vox.wav");
        fs::write(&a, b"first").unwrap();
        let other = root.path().join("other");
        fs::create_dir_all(&other).unwrap();
        let b = other.join("vox.wav");
        fs::write(&b, b"second take").unwrap();
        let inside = session
            .parent()
            .unwrap()
            .join("Recordings")
            .join("take.wav");
        fs::create_dir_all(inside.parent().unwrap()).unwrap();
        fs::write(&inside, b"take").unwrap();

        let dir = media_dir(&session, " Audio ");
        let keep = session.parent().unwrap();
        let r = copy_in(&[a.clone(), inside.clone(), b.clone()], &dir, keep);
        assert_eq!(r.copied, 2);
        assert_eq!(r.paths[0], dir.join("vox.wav"));
        assert_eq!(r.paths[1], inside);
        assert_eq!(r.paths[2], dir.join("vox (2).wav"));
        assert_eq!(fs::read(&r.paths[2]).unwrap(), b"second take");

        // Importing the same file again finds its copy.
        let again = copy_in(&[a, b], &dir, keep);
        assert_eq!((again.copied, again.reused), (0, 2));
        assert_eq!(
            again.paths,
            vec![dir.join("vox.wav"), dir.join("vox (2).wav")]
        );
        assert!(r.failed.is_empty());
    }

    #[test]
    fn two_drops_of_the_same_file_at_once_make_one_copy() {
        let root = tempfile::tempdir().unwrap();
        let session = root.path().join("S").join("S.altaudio");
        let src = root.path().join("outside").join("big.flac");
        fs::create_dir_all(src.parent().unwrap()).unwrap();
        fs::write(&src, vec![7u8; 4 * 1024 * 1024]).unwrap();
        let dir = media_dir(&session, "");
        let keep = session.parent().unwrap().to_path_buf();
        let run = || {
            let (s, d, k) = (src.clone(), dir.clone(), keep.clone());
            std::thread::spawn(move || copy_in(&[s], &d, &k))
        };
        let (a, b) = (run(), run());
        let (a, b) = (a.join().unwrap(), b.join().unwrap());
        assert!(a.failed.is_empty() && b.failed.is_empty());
        assert_eq!(a.paths, b.paths);
        assert_eq!(a.copied + b.copied, 1);
        let names = fs::read_dir(&dir).unwrap().count();
        assert_eq!(names, 1);
    }

    #[test]
    fn a_missing_file_stays_put_and_is_reported() {
        let root = tempfile::tempdir().unwrap();
        let session = root.path().join("S").join("S.altaudio");
        let gone = root.path().join("gone.wav");
        let r = copy_in(
            std::slice::from_ref(&gone),
            &media_dir(&session, ""),
            session.parent().unwrap(),
        );
        assert_eq!(r.paths, vec![gone]);
        assert_eq!(r.failed.len(), 1);
        assert!(r.failed[0].contains("the original is not at"));
        assert!(media_dir(&session, "../x").ends_with("x"));
    }
}
