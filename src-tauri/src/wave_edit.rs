// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Waveform editor edits in Rust: delete, crop, silence, amplify, fades, copy,
//! paste, mix paste, crossfading joins, replace, stretch and insert. The file
//! is a one-track session whose clips are the edit list, so each edit is a
//! session op built from the document and a range in samples. Mirrors
//! `src/panels/editor/wave-edit.ts`, tested against
//! `shared-fixtures/wave-edit.json`. Sessions and ops are `serde_json::Value`.
//! A document with MIDI clips is not a Waveform editor file, so the length of
//! the file counts audio clips only.

use crate::clip_edit::{
    end_of, from_points, isolate, length_of, media_by_id, piece_of, points_of, ripple_delete_range,
    with_crossfades, LockedError,
};
use crate::envelope::{level_at, DB_MIN};
use crate::js_compat::js_round;
use serde_json::{json, Value};
use std::collections::HashSet;

fn num(v: &Value, key: &str) -> f64 {
    v.get(key).and_then(Value::as_f64).unwrap_or(0.0)
}

fn text<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or("")
}

fn arr<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v.get(key)
        .and_then(Value::as_array)
        .map_or(&[], Vec::as_slice)
}

fn truthy(v: &Value, key: &str) -> bool {
    match v.get(key) {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => n.as_f64().is_some_and(|x| x != 0.0),
        Some(Value::String(s)) => !s.is_empty(),
        Some(_) => true,
    }
}

fn track_ids(doc: &Value) -> Vec<String> {
    arr(doc, "tracks")
        .iter()
        .map(|t| text(t, "id").to_string())
        .collect()
}

fn first_track_id(doc: &Value) -> Option<String> {
    arr(doc, "tracks")
        .first()
        .map(|t| text(t, "id").to_string())
}

/// End of the last audio clip, in session samples.
pub fn session_end(doc: &Value) -> f64 {
    let mut end = 0.0_f64;
    for t in arr(doc, "tracks") {
        for c in arr(t, "clips") {
            end = end.max(end_of(doc, c));
        }
    }
    end
}

/// Clamps a range to the file and orders it. None when nothing is left.
pub fn clamp_range(doc: &Value, a: f64, b: f64) -> Option<(f64, f64)> {
    let end = session_end(doc);
    let lo = 0.0_f64.max(a.min(b));
    let hi = end.min(a.max(b));
    (hi > lo).then_some((lo, hi))
}

/// Adds marker moves to an edit, so markers stay on the same audio.
fn with_markers(op: Option<Value>, extra: Vec<Value>) -> Option<Value> {
    let op = op?;
    if extra.is_empty() {
        return Some(op);
    }
    if text(&op, "op") == "batch" {
        let mut ops: Vec<Value> = arr(&op, "ops").to_vec();
        ops.extend(extra);
        Some(json!({ "op": "batch", "ops": ops }))
    } else {
        let mut ops = vec![op];
        ops.extend(extra);
        Some(json!({ "op": "batch", "ops": ops }))
    }
}

/// Marker moves for removing a..b: later markers move left, markers inside the
/// cut land at its start, and range markers shrink by the part that went.
pub fn markers_after_cut(doc: &Value, a: f64, b: f64) -> Vec<Value> {
    let len = b - a;
    let at = |p: f64| {
        if p >= b {
            p - len
        } else if p >= a {
            a
        } else {
            p
        }
    };
    let mut changes = vec![];
    for m in arr(doc, "markers") {
        let (pos, dur) = (num(m, "position"), num(m, "duration"));
        let start = at(pos);
        let end = if dur > 0.0 { at(pos + dur) } else { start };
        let duration = 0.0_f64.max(end - start);
        if start != pos || duration != dur {
            changes.push(json!({ "id": text(m, "id"), "position": start, "duration": duration }));
        }
    }
    if changes.is_empty() {
        vec![]
    } else {
        vec![json!({ "op": "set_markers", "changes": changes })]
    }
}

/// Marker moves for opening `len` samples at `at`: later markers move right,
/// ranges across it grow.
pub fn markers_after_insert(doc: &Value, at: f64, len: f64) -> Vec<Value> {
    let mut changes = vec![];
    for m in arr(doc, "markers") {
        let (pos, dur) = (num(m, "position"), num(m, "duration"));
        if pos >= at {
            changes.push(json!({ "id": text(m, "id"), "position": pos + len }));
        } else if dur > 0.0 && pos + dur > at {
            changes.push(json!({ "id": text(m, "id"), "duration": dur + len }));
        }
    }
    if changes.is_empty() {
        vec![]
    } else {
        vec![json!({ "op": "set_markers", "changes": changes })]
    }
}

/// Marker changes for Crop to a..b: markers outside go, the rest move to the
/// new start.
pub fn markers_after_crop(doc: &Value, a: f64, b: f64) -> Vec<Value> {
    let markers = arr(doc, "markers");
    let gone: Vec<&Value> = markers
        .iter()
        .filter(|m| {
            let (p, d) = (num(m, "position"), num(m, "duration"));
            p + d < a || p > b || (d == 0.0 && p == b)
        })
        .collect();
    let gone_ids: HashSet<&str> = gone.iter().map(|m| text(m, "id")).collect();
    let mut changes = vec![];
    for m in markers.iter().filter(|m| !gone_ids.contains(text(m, "id"))) {
        let (p, d) = (num(m, "position"), num(m, "duration"));
        let start = p.max(a);
        let end = (p + d).min(b);
        let (np, nd) = (start - a, 0.0_f64.max(end - start));
        if np != p || nd != d {
            changes.push(json!({ "id": text(m, "id"), "position": np, "duration": nd }));
        }
    }
    let mut out = vec![];
    if !gone.is_empty() {
        let ids: Vec<&str> = gone.iter().map(|m| text(m, "id")).collect();
        out.push(json!({ "op": "remove_markers", "ids": ids }));
    }
    if !changes.is_empty() {
        out.push(json!({ "op": "set_markers", "changes": changes }));
    }
    out
}

/// Delete: removes the range and closes the gap.
pub fn delete_range(
    doc: &Value,
    a: f64,
    b: f64,
    new_id: &mut dyn FnMut() -> String,
) -> Result<Option<Value>, LockedError> {
    let op = ripple_delete_range(doc, &track_ids(doc), a, b, new_id, None)?;
    Ok(with_markers(op, markers_after_cut(doc, a, b)))
}

/// Every clip after `ops` (an isolate result) runs, by id, in the order the
/// document lists them.
fn after_split(doc: &Value, ops: &[Value]) -> Vec<(String, Value)> {
    let mut clips: Vec<(String, Value)> = vec![];
    for t in arr(doc, "tracks") {
        for c in arr(t, "clips") {
            put(&mut clips, c);
        }
    }
    for op in ops {
        match text(op, "op") {
            "remove_clips" => {
                let gone: HashSet<&str> = arr(op, "ids").iter().filter_map(Value::as_str).collect();
                clips.retain(|(id, _)| !gone.contains(id.as_str()));
            }
            "restore_clips" => {
                for pair in arr(op, "clips") {
                    if let Some(c) = pair.get(1) {
                        put(&mut clips, c);
                    }
                }
            }
            _ => {}
        }
    }
    clips
}

/// Sets a clip by id, keeping the place of one already there.
fn put(list: &mut Vec<(String, Value)>, c: &Value) {
    let id = text(c, "id").to_string();
    match list.iter_mut().find(|(k, _)| *k == id) {
        Some(slot) => slot.1 = c.clone(),
        None => list.push((id, c.clone())),
    }
}

fn get<'a>(list: &'a [(String, Value)], id: &str) -> Option<&'a Value> {
    list.iter().find(|(k, _)| k == id).map(|(_, v)| v)
}

/// Crop: keeps only the range, which then starts at 0.
pub fn crop_to(
    doc: &Value,
    a: f64,
    b: f64,
    new_id: &mut dyn FnMut() -> String,
) -> Result<Option<Value>, LockedError> {
    if a <= 0.0 && b >= session_end(doc) {
        return Ok(None);
    }
    let found = isolate(doc, &track_ids(doc), a, b, new_id)?;
    let keep: HashSet<&str> = found.inside.iter().map(String::as_str).collect();
    let clips = after_split(doc, &found.ops);
    let outside: Vec<&str> = clips
        .iter()
        .map(|(id, _)| id.as_str())
        .filter(|id| !keep.contains(id))
        .collect();
    let mut out = found.ops.clone();
    if !outside.is_empty() {
        out.push(json!({ "op": "remove_clips", "ids": outside }));
    }
    if a > 0.0 {
        let changes: Vec<Value> = found
            .inside
            .iter()
            .map(|id| {
                let start = get(&clips, id).map_or(0.0, |c| num(c, "start"));
                json!({ "id": id, "start": start - a })
            })
            .collect();
        out.push(json!({ "op": "set_clips", "changes": changes }));
    }
    out.extend(markers_after_crop(doc, a, b));
    Ok(Some(json!({ "op": "batch", "ops": out })))
}

/// Splits around the range, then applies `change` to every piece inside it.
fn on_pieces(
    doc: &Value,
    a: f64,
    b: f64,
    new_id: &mut dyn FnMut() -> String,
    mut change: impl FnMut(&Value) -> Option<Value>,
) -> Result<Option<Value>, LockedError> {
    let found = isolate(doc, &track_ids(doc), a, b, new_id)?;
    let pieces = after_split(doc, &found.ops);
    let changes: Vec<Value> = found
        .inside
        .iter()
        .filter_map(|id| get(&pieces, id).and_then(&mut change))
        .collect();
    if changes.is_empty() {
        return Ok(None);
    }
    let mut ops = found.ops;
    ops.push(json!({ "op": "set_clips", "changes": changes }));
    Ok(Some(json!({ "op": "batch", "ops": ops })))
}

/// Silence: the range plays nothing; its length stays.
pub fn silence_range(
    doc: &Value,
    a: f64,
    b: f64,
    new_id: &mut dyn FnMut() -> String,
) -> Result<Option<Value>, LockedError> {
    on_pieces(doc, a, b, new_id, |c| {
        (!truthy(c, "muted")).then(|| json!({ "id": text(c, "id"), "muted": true }))
    })
}

/// Amplify: raises or lowers the range by `db`.
pub fn amplify_range(
    doc: &Value,
    a: f64,
    b: f64,
    db: f64,
    new_id: &mut dyn FnMut() -> String,
) -> Result<Option<Value>, LockedError> {
    if db == 0.0 {
        return Ok(None);
    }
    on_pieces(doc, a, b, new_id, |c| {
        let gain = (-96.0_f64).max(24.0_f64.min(num(c, "gain_db") + db));
        Some(json!({ "id": text(c, "id"), "gain_db": gain }))
    })
}

/// Gain in dB for a fade at `t` from 0 to 1, silent at 0. "linear" ramps the
/// gain itself in a straight line; anything else is an equal-power curve.
pub fn fade_db(t: f64, shape: &str) -> f64 {
    let x = 0.0_f64.max(1.0_f64.min(t));
    let g = if shape == "linear" {
        x
    } else {
        (x * std::f64::consts::PI / 2.0).sin()
    };
    if g <= 0.0 {
        DB_MIN
    } else {
        DB_MIN.max(20.0 * g.log10())
    }
}

const FADE_POINTS: usize = 24;

/// Fade In or Fade Out over the range: written into each piece's volume
/// envelope, on top of any envelope it already has.
pub fn fade_range(
    doc: &Value,
    a: f64,
    b: f64,
    dir: &str,
    new_id: &mut dyn FnMut() -> String,
    shape: &str,
) -> Result<Option<Value>, LockedError> {
    let span = b - a;
    if span <= 0.0 {
        return Ok(None);
    }
    let rate = num(doc, "sample_rate");
    on_pieces(doc, a, b, new_id, |c| {
        let m = media_by_id(doc, text(c, "media_id"));
        let len = length_of(c, m, rate);
        let old = points_of(c, "volume");
        let pts: Vec<crate::envelope::Point> = (0..=FADE_POINTS)
            .map(|i| {
                let pos = js_round(len * i as f64 / FADE_POINTS as f64);
                let t = (num(c, "start") + pos - a) / span;
                let f = fade_db(if dir == "in" { t } else { 1.0 - t }, shape);
                crate::envelope::Point {
                    position: pos,
                    value: DB_MIN.max(level_at(&old, pos) + f),
                    curve: "linear".to_string(),
                }
            })
            .collect();
        Some(json!({ "id": text(c, "id"), "volume": from_points(&pts) }))
    })
}

/// Copy: the range as pieces starting at 0, with the files they play.
pub fn copy_range(doc: &Value, a: f64, b: f64) -> Option<Value> {
    let rate = num(doc, "sample_rate");
    let mut clips = vec![];
    for t in arr(doc, "tracks") {
        for c in arr(t, "clips") {
            let end = end_of(doc, c);
            let start = num(c, "start");
            if end <= a || start >= b {
                continue;
            }
            let from = a.max(start) - start;
            let to = b.min(end) - start;
            let m = media_by_id(doc, text(c, "media_id"));
            clips.push(piece_of(
                c,
                m,
                rate,
                from,
                to,
                start + from - a,
                text(c, "id"),
            ));
        }
    }
    if clips.is_empty() {
        return None;
    }
    let used: HashSet<&str> = clips.iter().map(|c| text(c, "media_id")).collect();
    let media: Vec<Value> = arr(doc, "media")
        .iter()
        .filter(|m| used.contains(text(m, "id")))
        .cloned()
        .collect();
    Some(json!({ "rate": rate, "length": b - a, "clips": clips, "media": media }))
}

fn has_media(doc: &Value, id: &str) -> bool {
    media_by_id(doc, id).is_some()
}

/// Mix Paste: lays the copied audio over what is already there from `at`,
/// `db` louder or quieter, without moving anything. Returns the edit and the
/// length pasted.
pub fn mix_paste_at(
    doc: &Value,
    board: &Value,
    at: f64,
    db: f64,
    new_id: &mut dyn FnMut() -> String,
) -> Option<(Value, f64)> {
    let track = first_track_id(doc)?;
    if arr(board, "clips").is_empty() {
        return None;
    }
    let ratio = num(doc, "sample_rate") / num(board, "rate");
    let mut ops = vec![];
    for m in arr(board, "media") {
        if !has_media(doc, text(m, "id")) {
            ops.push(json!({ "op": "add_media", "media": m }));
        }
    }
    let clips: Vec<Value> = arr(board, "clips")
        .iter()
        .map(|c| {
            let mut n = c.clone();
            n["id"] = json!(new_id());
            n["start"] = json!(at + js_round(num(c, "start") * ratio));
            n["gain_db"] = json!((-96.0_f64).max(24.0_f64.min(num(c, "gain_db") + db)));
            json!([track, n])
        })
        .collect();
    ops.push(json!({ "op": "restore_clips", "clips": clips }));
    Some((
        json!({ "op": "batch", "ops": ops }),
        js_round(num(board, "length") * ratio),
    ))
}

/// What opening a gap at `at` does: the ops that remove what was split, the
/// clips put back (the ones from `at` on, moved right by `length`) and the
/// moves for every untouched clip from `at` on.
fn open_gap(
    doc: &Value,
    at: f64,
    length: f64,
    new_id: &mut dyn FnMut() -> String,
    ops: &mut Vec<Value>,
) -> Result<Vec<Value>, LockedError> {
    let found = isolate(doc, &track_ids(doc), at, 9007199254740991.0, new_id)?;
    let mut moved: Vec<(String, Value)> = vec![];
    for t in arr(doc, "tracks") {
        for c in arr(t, "clips") {
            put(&mut moved, c);
        }
    }
    let mut restored: Vec<Value> = vec![];
    for op in &found.ops {
        match text(op, "op") {
            "remove_clips" => ops.push(op.clone()),
            "restore_clips" => {
                for pair in arr(op, "clips") {
                    let (tid, c) = (&pair[0], &pair[1]);
                    moved.retain(|(k, _)| k != text(c, "id"));
                    let mut c = c.clone();
                    if num(&c, "start") >= at {
                        c["start"] = json!(num(&c, "start") + length);
                    }
                    restored.push(json!([tid, c]));
                }
            }
            _ => {}
        }
    }
    let shift: Vec<Value> = moved
        .iter()
        .filter(|(_, c)| num(c, "start") >= at)
        .map(|(id, c)| json!({ "id": id, "start": num(c, "start") + length }))
        .collect();
    if !shift.is_empty() {
        ops.push(json!({ "op": "set_clips", "changes": shift }));
    }
    Ok(restored)
}

/// Paste: opens a gap at `at` as long as the copied audio and puts it there.
/// Audio copied from a file at another rate keeps its timing.
pub fn paste_at(
    doc: &Value,
    board: &Value,
    at: f64,
    new_id: &mut dyn FnMut() -> String,
) -> Result<Option<(Value, f64)>, LockedError> {
    let Some(track) = first_track_id(doc) else {
        return Ok(None);
    };
    if arr(board, "clips").is_empty() {
        return Ok(None);
    }
    let ratio = num(doc, "sample_rate") / num(board, "rate");
    let length = js_round(num(board, "length") * ratio);
    let mut ops = vec![];
    for m in arr(board, "media") {
        if !has_media(doc, text(m, "id")) {
            ops.push(json!({ "op": "add_media", "media": m }));
        }
    }
    let mut restored = open_gap(doc, at, length, new_id, &mut ops)?;
    for c in arr(board, "clips") {
        let mut n = c.clone();
        n["id"] = json!(new_id());
        n["start"] = json!(at + js_round(num(c, "start") * ratio));
        restored.push(json!([track, n]));
    }
    ops.push(json!({ "op": "restore_clips", "clips": restored }));
    ops.extend(markers_after_insert(doc, at, length));
    Ok(Some((json!({ "op": "batch", "ops": ops }), length)))
}

/// Crossfades the joins at `seams`: where one piece ends and the next begins
/// at the same frame, the first piece runs on past the join by `length` frames
/// and the two fade across that overlap. A join where the first piece has no
/// more audio, or that is not a plain join, is skipped. Returns the edit and
/// how many joins it covers.
pub fn crossfade_seams(
    doc: &Value,
    seams: &[f64],
    length: f64,
    default_shape: &str,
) -> Option<(Value, usize)> {
    let rate = num(doc, "sample_rate");
    let mut changes: Vec<Value> = vec![];
    let mut taken: HashSet<String> = HashSet::new();
    for t in arr(doc, "tracks") {
        let clips = arr(t, "clips");
        for &seam in seams {
            let left = clips.iter().find(|c| {
                !truthy(c, "looped")
                    && !truthy(c, "stretch")
                    && !taken.contains(text(c, "id"))
                    && end_of(doc, c) == seam
            });
            let right = clips.iter().find(|c| {
                num(c, "start") == seam && Some(text(c, "id")) != left.map(|l| text(l, "id"))
            });
            let (Some(left), Some(right)) = (left, right) else {
                continue;
            };
            let Some(m) = media_by_id(doc, text(left, "media_id")) else {
                continue;
            };
            let sr = if truthy(m, "sample_rate") {
                num(m, "sample_rate")
            } else {
                rate
            };
            let right_len = end_of(doc, right) - num(right, "start");
            let half = ((num(left, "source_out") - num(left, "source_in"))
                .min(right_len * sr / rate)
                / 2.0)
                .floor();
            let extra = js_round(length * sr / rate)
                .min(half)
                .min(num(m, "length") - num(left, "source_out"));
            if extra < 1.0 {
                continue;
            }
            taken.insert(text(left, "id").to_string());
            changes.push(
                json!({ "id": text(left, "id"), "source_out": num(left, "source_out") + extra }),
            );
        }
    }
    if changes.is_empty() {
        return None;
    }
    let joins = changes.len();
    let merged = with_crossfades(doc, &changes, default_shape);
    Some((json!({ "op": "set_clips", "changes": merged }), joins))
}

fn new_clip(id: String, name: &str, media: &Value, start: f64) -> Value {
    json!({
        "id": id,
        "name": name,
        "media_id": text(media, "id"),
        "start": start,
        "source_in": 0,
        "source_out": num(media, "length"),
    })
}

/// Replace: swaps the audio in a..b for `media`, a file of exactly that
/// length. The rest of the file stays put.
pub fn replace_range(
    doc: &Value,
    a: f64,
    b: f64,
    media: &Value,
    name: &str,
    new_id: &mut dyn FnMut() -> String,
) -> Result<Option<Value>, LockedError> {
    let Some(track) = first_track_id(doc) else {
        return Ok(None);
    };
    if b <= a {
        return Ok(None);
    }
    let found = isolate(doc, &track_ids(doc), a, b, new_id)?;
    let mut out = found.ops.clone();
    if !found.inside.is_empty() {
        out.push(json!({ "op": "remove_clips", "ids": found.inside }));
    }
    out.push(json!({ "op": "add_media", "media": media }));
    out.push(json!({ "op": "add_clips", "track_id": track, "clips": [new_clip(new_id(), name, media, a)] }));
    Ok(Some(json!({ "op": "batch", "ops": out })))
}

/// Stretch: puts `media` (the range re-timed) in place of a..b and moves
/// everything after b by the change in length. Markers after the range move
/// with it; markers inside are spaced out (or in) to match.
pub fn stretch_range(
    doc: &Value,
    a: f64,
    b: f64,
    media: &Value,
    name: &str,
    new_id: &mut dyn FnMut() -> String,
) -> Result<Option<Value>, LockedError> {
    let Some(track) = first_track_id(doc) else {
        return Ok(None);
    };
    if b <= a || num(media, "length") <= 0.0 {
        return Ok(None);
    }
    let rate = num(doc, "sample_rate");
    let length = js_round(num(media, "length") * rate / num(media, "sample_rate"));
    let delta = length - (b - a);
    let found = isolate(doc, &track_ids(doc), a, b, new_id)?;
    let mut out = found.ops.clone();
    if !found.inside.is_empty() {
        out.push(json!({ "op": "remove_clips", "ids": found.inside }));
    }
    let gone: HashSet<&str> = found.inside.iter().map(String::as_str).collect();
    let clips = after_split(doc, &found.ops);
    let mut shift: Vec<(String, f64)> = clips
        .iter()
        .filter(|(id, c)| !gone.contains(id.as_str()) && num(c, "start") >= b && delta != 0.0)
        .map(|(id, c)| (id.clone(), num(c, "start") + delta))
        .collect();
    // Moving right, the furthest clip goes first; moving left, the nearest.
    shift.sort_by(|x, y| {
        let o = if delta > 0.0 {
            y.1.partial_cmp(&x.1)
        } else {
            x.1.partial_cmp(&y.1)
        };
        o.unwrap_or(std::cmp::Ordering::Equal)
    });
    if !shift.is_empty() {
        let changes: Vec<Value> = shift
            .iter()
            .map(|(id, s)| json!({ "id": id, "start": s }))
            .collect();
        out.push(json!({ "op": "set_clips", "changes": changes }));
    }
    out.push(json!({ "op": "add_media", "media": media }));
    out.push(json!({ "op": "add_clips", "track_id": track, "clips": [new_clip(new_id(), name, media, a)] }));
    let ratio = length / (b - a);
    let mut markers = vec![];
    for m in arr(doc, "markers") {
        let pos = num(m, "position");
        if pos >= b {
            if delta != 0.0 {
                markers.push(json!({ "id": text(m, "id"), "position": pos + delta }));
            }
        } else if pos >= a {
            markers.push(json!({
                "id": text(m, "id"),
                "position": a + js_round((pos - a) * ratio),
                "duration": js_round(num(m, "duration") * ratio),
            }));
        }
    }
    if !markers.is_empty() {
        out.push(json!({ "op": "set_markers", "changes": markers }));
    }
    Ok(Some(json!({ "op": "batch", "ops": out })))
}

/// Insert: puts a whole file (a recorded take) in at `at`, moving what follows
/// to the right. Returns the edit and the length inserted.
pub fn insert_file(
    doc: &Value,
    at: f64,
    media: &Value,
    name: &str,
    new_id: &mut dyn FnMut() -> String,
) -> Result<Option<(Value, f64)>, LockedError> {
    let Some(track) = first_track_id(doc) else {
        return Ok(None);
    };
    if num(media, "length") <= 0.0 {
        return Ok(None);
    }
    let length =
        js_round(num(media, "length") * num(doc, "sample_rate") / num(media, "sample_rate"));
    let mut ops = vec![json!({ "op": "add_media", "media": media })];
    let restored = open_gap(doc, at, length, new_id, &mut ops)?;
    if !restored.is_empty() {
        ops.push(json!({ "op": "restore_clips", "clips": restored }));
    }
    ops.push(json!({ "op": "add_clips", "track_id": track, "clips": [new_clip(new_id(), name, media, at)] }));
    ops.extend(markers_after_insert(doc, at, length));
    Ok(Some((json!({ "op": "batch", "ops": ops }), length)))
}
