// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Saving and opening `.altaudio` sessions, autosave, and recovery.
//!
//! Media paths are stored twice: absolute, and relative to the session
//! file when the media is on the same drive. On open the relative path
//! is tried first, so a show folder can move between drives or machines
//! and still open.
//!
//! Autosaves go to `<app data>/autosave/<session id>.altaudio` with a
//! small `.json` note beside it saying where the real file lives. A clean
//! save or close removes them; any left at startup mean the app did not
//! close cleanly, and the user is offered recovery.

use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::document::{self, DocumentError};
use super::model::{Id, MediaRef, Session};

/// Session file extension, without the dot.
pub const EXTENSION: &str = "altaudio";

/// Saves `session` to `path`. `extra_media` adds files listed in the
/// Files panel that no clip uses yet, so they come back on open.
pub fn save(
    session: &Session,
    path: &Path,
    extra_media: &[MediaRef],
) -> Result<Session, DocumentError> {
    let mut s = session.clone();
    for m in extra_media {
        if !s.media.iter().any(|x| x.id == m.id) {
            s.media.push(m.clone());
        }
    }
    let base = path.parent().unwrap_or(Path::new(""));
    for m in &mut s.media {
        m.relative_path = relative_to(&m.path, base);
    }
    s.modified = Utc::now();
    document::save(&s, path)?;
    Ok(s)
}

/// Relative path from `base` to `target`, when both are absolute and on
/// the same drive. Paths are stored with `/` so they work on any system.
fn relative_to(target: &Path, base: &Path) -> Option<PathBuf> {
    if !target.is_absolute() || !base.is_absolute() {
        return None;
    }
    let rel = pathdiff::diff_paths(target, base)?;
    // Different drive letters on Windows give an absolute result.
    if rel.is_absolute() {
        return None;
    }
    Some(PathBuf::from(rel.to_string_lossy().replace('\\', "/")))
}

/// Result of opening a session.
#[derive(Debug, Clone, Serialize)]
pub struct Opened {
    pub session: Session,
    /// Media files not found at either stored path.
    pub missing: Vec<Id>,
}

/// Opens a session and finds each media file, preferring the path
/// relative to the session file.
pub fn open(path: &Path) -> Result<Opened, DocumentError> {
    let mut s: Session = document::load(path)?;
    let base = path.parent().unwrap_or(Path::new(""));
    let mut missing = Vec::new();
    for m in &mut s.media {
        let from_rel = m.relative_path.as_ref().map(|r| base.join(r));
        match from_rel {
            Some(p) if p.is_file() => m.path = normalize(&p),
            _ if m.path.is_file() => {}
            _ => missing.push(m.id),
        }
    }
    Ok(Opened {
        session: s,
        missing,
    })
}

/// Cleans `a/../b` style paths without touching the disk.
fn normalize(p: &Path) -> PathBuf {
    use std::path::Component;
    let mut out = PathBuf::new();
    for c in p.components() {
        match c {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// Note saved beside an autosave.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecoveryNote {
    pub session_id: Id,
    pub name: String,
    /// The session's real file, if it had been saved.
    pub original_path: Option<PathBuf>,
    pub saved_at: DateTime<Utc>,
}

/// An autosave found at startup.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Recoverable {
    pub autosave: PathBuf,
    pub note: RecoveryNote,
}

fn autosave_paths(dir: &Path, id: Id) -> (PathBuf, PathBuf) {
    (
        dir.join(format!("{id}.{EXTENSION}")),
        dir.join(format!("{id}.json")),
    )
}

pub fn autosave(
    dir: &Path,
    session: &Session,
    original: Option<&Path>,
    extra_media: &[MediaRef],
) -> Result<PathBuf, DocumentError> {
    fs::create_dir_all(dir).map_err(|source| DocumentError::Io {
        path: dir.to_path_buf(),
        source,
    })?;
    let (file, note_path) = autosave_paths(dir, session.id);
    // Media keeps absolute paths; relative ones would point inside the
    // autosave folder.
    let mut s = session.clone();
    for m in extra_media {
        if !s.media.iter().any(|x| x.id == m.id) {
            s.media.push(m.clone());
        }
    }
    document::save(&s, &file)?;
    let note = RecoveryNote {
        session_id: session.id,
        name: session.name.clone(),
        original_path: original.map(Path::to_path_buf),
        saved_at: Utc::now(),
    };
    let text = serde_json::to_string_pretty(&note)?;
    fs::write(&note_path, text).map_err(|source| DocumentError::Io {
        path: note_path,
        source,
    })?;
    Ok(file)
}

/// Removes the autosave for a session after a clean save or close.
pub fn clear_autosave(dir: &Path, id: Id) {
    let (file, note) = autosave_paths(dir, id);
    let _ = fs::remove_file(file);
    let _ = fs::remove_file(note);
}

/// Autosaves left behind by a crash, newest first.
pub fn find_recoverable(dir: &Path) -> Vec<Recoverable> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for e in entries.flatten() {
        let p = e.path();
        if p.extension().and_then(|x| x.to_str()) != Some("json") {
            continue;
        }
        let Ok(text) = fs::read_to_string(&p) else {
            continue;
        };
        let Ok(note) = serde_json::from_str::<RecoveryNote>(&text) else {
            continue;
        };
        let file = p.with_extension(EXTENSION);
        if file.is_file() {
            out.push(Recoverable {
                autosave: file,
                note,
            });
        }
    }
    out.sort_by_key(|r| std::cmp::Reverse(r.note.saved_at));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::model::new_id;

    fn media(id: Id, path: PathBuf) -> MediaRef {
        MediaRef {
            id,
            path,
            relative_path: None,
            sample_rate: 48_000,
            channels: 2,
            length: 48_000,
        }
    }

    #[test]
    fn save_and_open_follow_a_moved_show_folder() {
        let root = tempfile::tempdir().unwrap();
        let show = root.path().join("Show");
        fs::create_dir_all(show.join("Raw")).unwrap();
        let wav = show.join("Raw").join("host.wav");
        fs::write(&wav, b"x").unwrap();

        let mut s = Session::new("Ep 1", 48_000);
        let id = new_id();
        s.media.push(media(id, wav.clone()));
        let file = show.join("ep1.altaudio");
        let saved = save(&s, &file, &[]).unwrap();
        assert_eq!(
            saved.media[0].relative_path.as_deref(),
            Some(Path::new("Raw/host.wav"))
        );

        // Move the whole folder; the absolute path no longer exists.
        let moved = root.path().join("Moved");
        fs::rename(&show, &moved).unwrap();
        let opened = open(&moved.join("ep1.altaudio")).unwrap();
        assert!(opened.missing.is_empty());
        assert_eq!(
            opened.session.media[0].path,
            moved.join("Raw").join("host.wav")
        );
    }

    #[test]
    fn open_reports_missing_media() {
        let dir = tempfile::tempdir().unwrap();
        let mut s = Session::new("Ep 2", 48_000);
        let id = new_id();
        s.media.push(media(id, dir.path().join("gone.wav")));
        let file = dir.path().join("ep2.altaudio");
        save(&s, &file, &[]).unwrap();
        assert_eq!(open(&file).unwrap().missing, vec![id]);
    }

    #[test]
    fn extra_media_is_saved_once() {
        let dir = tempfile::tempdir().unwrap();
        let mut s = Session::new("Ep 3", 48_000);
        let a = media(new_id(), dir.path().join("a.wav"));
        s.media.push(a.clone());
        let b = media(new_id(), dir.path().join("b.wav"));
        let saved = save(&s, &dir.path().join("x.altaudio"), &[a, b]).unwrap();
        assert_eq!(saved.media.len(), 2);
    }

    #[test]
    fn autosave_round_trip_and_clear() {
        let dir = tempfile::tempdir().unwrap();
        let s = Session::new("Ep 4", 48_000);
        assert!(find_recoverable(dir.path()).is_empty());
        autosave(
            dir.path(),
            &s,
            Some(Path::new("C:/Shows/ep4.altaudio")),
            &[],
        )
        .unwrap();
        let found = find_recoverable(dir.path());
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].note.name, "Ep 4");
        assert_eq!(
            found[0].note.original_path.as_deref(),
            Some(Path::new("C:/Shows/ep4.altaudio"))
        );
        let back: Session = document::load(&found[0].autosave).unwrap();
        assert_eq!(back.id, s.id);
        clear_autosave(dir.path(), s.id);
        assert!(find_recoverable(dir.path()).is_empty());
    }
}
