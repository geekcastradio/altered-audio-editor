// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Rows for the audio files open in the Waveform editor, shown in the Files
//! panel. Mirrors `src/panels/files/wave-rows.ts`; tested against
//! `shared-fixtures/wave-space.json`.

#[derive(Clone, Debug, PartialEq)]
pub struct WaveRow {
    pub path: String,
    pub name: String,
    pub format: String,
    pub untitled: bool,
    pub dirty: bool,
    /// The file the Waveform editor shows now. False for files set aside.
    pub front: bool,
}

/// A file set aside while another one is open.
#[derive(Clone, Debug)]
pub struct ParkedFile {
    pub path: String,
    pub format: String,
    pub untitled: bool,
    pub dirty: bool,
}

/// The file the Waveform editor shows now.
#[derive(Clone, Debug)]
pub struct CurrentFile {
    pub path: String,
    pub format: String,
    pub untitled: bool,
}

/// The last part of a path, split at `\` or `/`; the whole path when that is empty.
fn base(p: &str) -> String {
    let last = p.rsplit(['\\', '/']).next().unwrap_or(p);
    if last.is_empty() { p } else { last }.to_string()
}

fn same(a: &str, b: &str) -> bool {
    a.to_lowercase() == b.to_lowercase()
}

/// The file in front, then the ones set aside. Files already listed as pool
/// items are left out, so nothing shows twice.
pub fn wave_rows(
    current: Option<&CurrentFile>,
    current_dirty: bool,
    parked: &[ParkedFile],
    listed_paths: &[String],
) -> Vec<WaveRow> {
    let listed = |p: &str| listed_paths.iter().any(|x| same(x, p));
    let mut rows = vec![];
    if let Some(c) = current.filter(|c| !listed(&c.path)) {
        rows.push(WaveRow {
            path: c.path.clone(),
            name: base(&c.path),
            format: c.format.clone(),
            untitled: c.untitled,
            dirty: current_dirty,
            front: true,
        });
    }
    for p in parked.iter().filter(|p| !listed(&p.path)) {
        rows.push(WaveRow {
            path: p.path.clone(),
            name: base(&p.path),
            format: p.format.clone(),
            untitled: p.untitled,
            dirty: p.dirty,
            front: false,
        });
    }
    rows
}
