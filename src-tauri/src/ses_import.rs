// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Reads older `.ses` multitrack sessions (the `COOLNESS` file layout).
//!
//! Only what was checked against real files is read. One layout is read so
//! far: the one with a `TRKS` chunk and a `bk25` clip list (a 3.0-era
//! session, UTF-16 names). The older layout with `trks` and `bk20` is
//! recognised and refused with a plain message, because its clip records do
//! not say which track a clip is on in any way that has been proven.
//!
//! Layout, little-endian throughout:
//! - 8 bytes `COOLNESS`, a u32, then chunks of: 4-character tag, u32 length,
//!   data. The `LISTFILE` chunk has an 8-character tag.
//! - `hdr `: first u32 is the session sample rate.
//! - `TRKS`: one record per track, each marked by the UTF-16 text
//!   `AudioTrack`. A u32 track id sits 26 bytes after the marker and the
//!   NUL-ended UTF-16 track name 34 bytes after it. The first record is the
//!   master bus.
//! - `WLST`: entries of tag `wav `, u32 size, then 16 bytes (a handle, two
//!   flags, and the file length in samples at byte 12) and the NUL-ended
//!   UTF-16 path.
//! - `bk25`: u32 count, u32 record size, then records. In a record: start in
//!   session samples at byte 24, length at 28, the id of the track it is on
//!   at 40, and the 1-based number of its file in `WLST` at 100.
//!
//! Nothing here touches the interface or the app handle, so it moves into a
//! native shell unchanged.

use std::path::{Path, PathBuf};

use crate::codecs::probe;
use crate::session::model::{new_id, ChannelLayout, Clip, Fade, Id, MediaRef, Session, Track};
use crate::sesx::{ImportReport, Imported, SesxError};

const MAGIC: &[u8; 8] = b"COOLNESS";
const MARKER: &str = "AudioTrack";

struct Chunk<'a> {
    tag: String,
    data: &'a [u8],
}

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    let s = b.get(at..at.checked_add(4)?)?;
    Some(u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
}

/// Splits the file into chunks. Stops quietly at damaged data.
fn chunks(d: &[u8]) -> Result<Vec<Chunk<'_>>, SesxError> {
    if d.len() < 12 || &d[..8] != MAGIC {
        return Err(SesxError::Ses(
            "This is not a .ses session file.".to_string(),
        ));
    }
    let mut out = Vec::new();
    let mut pos = 12usize;
    while pos + 8 <= d.len() {
        let (tag, len_at, data_at) =
            if &d[pos..pos + 4] == b"LIST" && &d[pos + 4..pos + 8] == b"FILE" {
                ("LISTFILE".to_string(), pos + 8, pos + 12)
            } else {
                (
                    String::from_utf8_lossy(&d[pos..pos + 4]).into_owned(),
                    pos + 4,
                    pos + 8,
                )
            };
        let Some(len) = u32_at(d, len_at) else { break };
        let end = data_at.saturating_add(len as usize);
        if end > d.len() {
            break;
        }
        out.push(Chunk {
            tag,
            data: &d[data_at..end],
        });
        pos = end;
    }
    Ok(out)
}

/// UTF-16 text up to a NUL, from `at`.
fn utf16_z(b: &[u8], at: usize) -> String {
    let mut units = Vec::new();
    let mut i = at;
    while let Some(p) = b.get(i..i + 2) {
        let u = u16::from_le_bytes([p[0], p[1]]);
        if u == 0 {
            break;
        }
        units.push(u);
        i += 2;
        if units.len() > 4096 {
            break;
        }
    }
    String::from_utf16_lossy(&units)
}

struct TrackRec {
    id: u32,
    name: String,
}

/// The tracks in file order, master bus first.
fn track_records(trks: &[u8]) -> Vec<TrackRec> {
    let needle: Vec<u8> = MARKER.encode_utf16().flat_map(u16::to_le_bytes).collect();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i + needle.len() <= trks.len() {
        if trks[i..i + needle.len()] == needle[..] {
            if let Some(id) = u32_at(trks, i + 26) {
                out.push(TrackRec {
                    id,
                    name: utf16_z(trks, i + 34),
                });
            }
            i += needle.len();
        } else {
            i += 1;
        }
    }
    out
}

struct FileRec {
    path: String,
    /// The file's length in its own samples, as the file list notes it.
    length: u64,
}

fn file_list(wlst: &[u8]) -> Vec<FileRec> {
    let mut out = Vec::new();
    let mut o = 0usize;
    while o + 8 <= wlst.len() {
        let Some(len) = u32_at(wlst, o + 4) else {
            break;
        };
        let (s, e) = (o + 8, (o + 8).saturating_add(len as usize));
        let Some(entry) = wlst.get(s..e) else { break };
        if &wlst[o..o + 4] == b"wav " && entry.len() >= 16 {
            out.push(FileRec {
                length: u64::from(u32_at(entry, 12).unwrap_or(0)),
                path: utf16_z(entry, 16),
            });
        } else {
            out.push(FileRec {
                length: 0,
                path: String::new(),
            });
        }
        o = e;
    }
    out
}

struct ClipRec {
    start: u64,
    length: u64,
    track_id: u32,
    file: usize,
}

fn clip_records(bk: &[u8]) -> Vec<ClipRec> {
    let (Some(n), Some(size)) = (u32_at(bk, 0), u32_at(bk, 4)) else {
        return Vec::new();
    };
    let size = size as usize;
    if size < 104 {
        return Vec::new();
    }
    let mut out = Vec::new();
    for i in 0..(n as usize).min(100_000) {
        let Some(r) = bk.get(8 + i * size..8 + (i + 1) * size) else {
            break;
        };
        out.push(ClipRec {
            start: u64::from(u32_at(r, 24).unwrap_or(0)),
            length: u64::from(u32_at(r, 28).unwrap_or(0)),
            track_id: u32_at(r, 40).unwrap_or(0),
            file: u32_at(r, 100).unwrap_or(0) as usize,
        });
    }
    out
}

/// Finds a file next to the session, or where the session says it was.
fn find_file(stored: &str, base: &Path) -> Option<PathBuf> {
    let name = stored.rsplit(['\\', '/']).next().unwrap_or(stored);
    let mut tries = vec![PathBuf::from(stored)];
    if !name.is_empty() {
        tries.push(base.join(name));
    }
    tries.into_iter().find(|p| p.is_file())
}

/// Reads a `.ses` session and builds a session from it.
pub fn import(path: &Path) -> Result<Imported, SesxError> {
    let bytes = std::fs::read(path)?;
    let name = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Imported Session".into());
    import_bytes(&bytes, path.parent().unwrap_or(Path::new("")), name)
}

pub(crate) fn import_bytes(d: &[u8], base: &Path, name: String) -> Result<Imported, SesxError> {
    let chunks = chunks(d)?;
    let find = |tag: &str| chunks.iter().find(|c| c.tag == tag).map(|c| c.data);
    let (Some(hdr), Some(trks), Some(bk)) = (find("hdr "), find("TRKS"), find("bk25")) else {
        if find("trks").is_some() && find("bk20").is_some() {
            return Err(SesxError::Ses(
                "This is an older Cool Edit Pro 2 session. Which track each clip is on is not readable in that layout yet, so it is not imported. If you can export the session as .sesx or save it from Audition, that opens.".to_string(),
            ));
        }
        return Err(SesxError::Ses(
            "This .ses layout is not one this version can read.".to_string(),
        ));
    };
    let rate = u32_at(hdr, 0).unwrap_or(48_000).clamp(8_000, 384_000);
    let mut session = Session::new(name, rate);
    let mut report = ImportReport::default();

    let files = find("WLST").map(file_list).unwrap_or_default();
    // Skip the first record, the master bus.
    let tracks: Vec<TrackRec> = track_records(trks).into_iter().skip(1).collect();
    let clips = clip_records(bk);

    // One media entry per file in the list, made when a clip first uses it.
    let mut media: Vec<Option<Id>> = vec![None; files.len()];
    let mut missing = Vec::new();
    let mut trimmed = 0usize;

    for t in &tracks {
        let mut track = Track::new(t.name.clone(), ChannelLayout::Stereo);
        let mut mine: Vec<&ClipRec> = clips.iter().filter(|c| c.track_id == t.id).collect();
        mine.sort_by_key(|c| c.start);
        for c in mine {
            let Some(file) = c
                .file
                .checked_sub(1)
                .and_then(|i| files.get(i).map(|f| (i, f)))
            else {
                continue;
            };
            let (idx, rec) = file;
            let media_id = match media[idx] {
                Some(id) => id,
                None => {
                    let found = find_file(&rec.path, base);
                    let m = match found.as_ref().and_then(|p| probe(p).ok().map(|i| (p, i))) {
                        Some((p, info)) => MediaRef {
                            id: new_id(),
                            path: p.clone(),
                            relative_path: None,
                            sample_rate: info.sample_rate,
                            channels: info.channels,
                            length: info.length,
                        },
                        None => {
                            let m = MediaRef {
                                id: new_id(),
                                path: PathBuf::from(&rec.path),
                                relative_path: None,
                                sample_rate: rate,
                                channels: 2,
                                length: rec.length.max(c.length),
                            };
                            missing.push(m.id);
                            report.missing_files.push(m.path.clone());
                            m
                        }
                    };
                    let id = m.id;
                    session.media.push(m);
                    media[idx] = Some(id);
                    id
                }
            };
            let file_rate = session
                .media
                .iter()
                .find(|m| m.id == media_id)
                .map_or(rate, |m| m.sample_rate.max(1));
            // A clip as long as its file is the whole file. A shorter one is
            // trimmed, and where it starts inside the file is not read yet,
            // so it starts at the beginning of the file.
            let whole = rec.length > 0 && c.length == rec.length;
            let source_out = if whole {
                rec.length
            } else {
                trimmed += 1;
                (c.length as f64 * f64::from(file_rate) / f64::from(rate)).round() as u64
            };
            if source_out == 0 {
                continue;
            }
            let stem = Path::new(&rec.path.replace('\\', "/"))
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "Clip".into());
            track.clips.push(Clip {
                extra: Default::default(),
                id: new_id(),
                name: stem,
                media_id,
                start: c.start,
                source_in: 0,
                source_out,
                gain_db: 0.0,
                muted: false,
                locked: false,
                color: None,
                fade_in: Fade::default(),
                fade_out: Fade::default(),
                envelopes: Vec::new(),
                effects: Default::default(),
                group: None,
                looped: None,
                stretch: None,
                audio_type: None,
                channel_mix: None,
                reversed: false,
                inverted: false,
                take: None,
            });
            report.clips += 1;
        }
        session.tracks.push(track);
        report.tracks += 1;
    }

    // Said once in the notice, so nothing looks like it came across when it did not.
    report
        .skipped_names
        .push("track volume, pan, effects, EQ and envelopes".to_string());
    report.skipped_effects = 1;
    if trimmed > 0 {
        report.skipped_names.push(format!(
            "where {trimmed} trimmed {} start inside the file (they start at its beginning)",
            if trimmed == 1 { "clip" } else { "clips" }
        ));
    }
    Ok(Imported {
        session,
        report,
        missing,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utf16(s: &str) -> Vec<u8> {
        s.encode_utf16()
            .chain(std::iter::once(0))
            .flat_map(u16::to_le_bytes)
            .collect()
    }

    fn chunk(tag: &[u8; 4], data: &[u8]) -> Vec<u8> {
        let mut v = tag.to_vec();
        v.extend((data.len() as u32).to_le_bytes());
        v.extend(data);
        v
    }

    fn track(id: u32, name: &str) -> Vec<u8> {
        let mut v = vec![0u8; 16];
        v.extend(utf16(MARKER)[..20].iter());
        v.extend([0u8; 6]);
        // The marker is 20 bytes, so the id sits at marker + 26.
        let mut rec = v;
        rec.extend(id.to_le_bytes());
        rec.extend(utf16(name).len().to_le_bytes()[..4].iter());
        rec.extend(utf16(name));
        rec.extend([0u8; 12]);
        rec
    }

    fn clip(start: u32, len: u32, track_id: u32, file: u32) -> Vec<u8> {
        let mut r = vec![0u8; 112];
        r[24..28].copy_from_slice(&start.to_le_bytes());
        r[28..32].copy_from_slice(&len.to_le_bytes());
        r[40..44].copy_from_slice(&track_id.to_le_bytes());
        r[100..104].copy_from_slice(&file.to_le_bytes());
        r
    }

    fn wlst_entry(path: &str, length: u32) -> Vec<u8> {
        let mut data = vec![0u8; 16];
        data[12..16].copy_from_slice(&length.to_le_bytes());
        data.extend(utf16(path));
        chunk(b"wav ", &data)
    }

    fn sample(clips: &[Vec<u8>]) -> Vec<u8> {
        let mut hdr = vec![0u8; 16];
        hdr[..4].copy_from_slice(&48_000u32.to_le_bytes());
        let mut trks = Vec::new();
        for (id, name) in [
            (10000, "Master"),
            (10001, "Track 1"),
            (10002, "Track 2"),
            (10003, "Track 3"),
        ] {
            trks.extend(track(id, name));
        }
        let mut bk = Vec::new();
        bk.extend((clips.len() as u32).to_le_bytes());
        bk.extend(112u32.to_le_bytes());
        for c in clips {
            bk.extend(c);
        }
        let mut body = Vec::new();
        body.extend(chunk(b"hdr ", &hdr));
        body.extend(chunk(b"TRKS", &trks));
        body.extend(chunk(
            b"WLST",
            &wlst_entry("C:\\Music\\Closing Theme.mp3", 2_340_864),
        ));
        body.extend(chunk(b"bk25", &bk));
        let mut f = MAGIC.to_vec();
        f.extend((body.len() as u32).to_le_bytes());
        f.extend(body);
        f
    }

    fn run(bytes: &[u8]) -> Result<Imported, SesxError> {
        import_bytes(bytes, Path::new("/nonexistent"), "t".into())
    }

    #[test]
    fn two_clips_land_on_their_own_tracks() {
        let f = sample(&[
            clip(0, 2_340_864, 10001, 1),
            clip(306_948, 2_340_864, 10002, 1),
        ]);
        let i = run(&f).unwrap();
        assert_eq!(i.session.sample_rate, 48_000);
        let names: Vec<_> = i.session.tracks.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(names, ["Track 1", "Track 2", "Track 3"]);
        assert_eq!(i.session.tracks[0].clips.len(), 1);
        assert_eq!(i.session.tracks[1].clips.len(), 1);
        assert!(i.session.tracks[2].clips.is_empty());
        assert_eq!(i.session.tracks[0].clips[0].start, 0);
        assert_eq!(i.session.tracks[1].clips[0].start, 306_948);
        assert_eq!(i.session.tracks[1].clips[0].source_out, 2_340_864);
        assert_eq!(i.session.media.len(), 1);
        assert_eq!(i.report.tracks, 3);
        assert_eq!(i.report.clips, 2);
        assert_eq!(i.report.missing_files.len(), 1);
        assert_eq!(i.session.tracks[0].clips[0].name, "Closing Theme");
    }

    #[test]
    fn a_clip_on_track_two_with_track_one_empty_stays_on_track_two() {
        let f = sample(&[clip(1000, 2_340_864, 10002, 1)]);
        let i = run(&f).unwrap();
        assert!(i.session.tracks[0].clips.is_empty());
        assert_eq!(i.session.tracks[1].clips[0].start, 1000);
    }

    #[test]
    fn a_trimmed_clip_is_reported() {
        let f = sample(&[clip(0, 1_000_000, 10001, 1)]);
        let i = run(&f).unwrap();
        assert_eq!(i.session.tracks[0].clips[0].source_out, 1_000_000);
        assert!(i
            .report
            .skipped_names
            .iter()
            .any(|n| n.contains("1 trimmed clip")));
    }

    #[test]
    fn a_clip_for_an_unknown_file_or_track_is_left_out() {
        let f = sample(&[clip(0, 10, 10001, 9), clip(0, 10, 99999, 1)]);
        let i = run(&f).unwrap();
        assert_eq!(i.report.clips, 0);
    }

    #[test]
    fn the_older_cool_edit_layout_is_refused_plainly() {
        let mut body = Vec::new();
        body.extend(chunk(b"hdr ", &[0u8; 16]));
        body.extend(chunk(b"trks", &[0u8; 8]));
        body.extend(chunk(b"bk20", &[0u8; 8]));
        let mut f = MAGIC.to_vec();
        f.extend((body.len() as u32).to_le_bytes());
        f.extend(body);
        let e = run(&f).unwrap_err().to_string();
        assert!(e.contains("older Cool Edit Pro 2"), "{e}");
    }

    #[test]
    fn other_files_and_damaged_files_never_panic() {
        assert!(run(b"not a session").is_err());
        assert!(run(&[]).is_err());
        let f = sample(&[clip(0, 2_340_864, 10001, 1)]);
        for cut in (0..f.len()).step_by(7) {
            let _ = run(&f[..cut]);
        }
        let mut bad = f.clone();
        for b in bad.iter_mut().skip(12) {
            *b = 0xff;
        }
        let _ = run(&bad);
    }
}
