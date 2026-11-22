// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Mix routing, Duck Music Under Voice and Sync Tracks as rules over a session
//! document read as JSON. Matches `src/panels/mixer/routing.ts`,
//! `src/app/duck.ts` and `src/app/sync-tracks.ts`; all are checked against
//! `shared-fixtures/routing-duck-sync.json`.

use crate::js_compat::{js_round, js_to_fixed};
use serde_json::{json, Map, Value};
use std::collections::HashSet;

/// What JavaScript treats as true.
pub(crate) fn truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|x| x != 0.0 && !x.is_nan()),
        Value::String(s) => !s.is_empty(),
        _ => true,
    }
}

fn arr<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v.get(key)
        .and_then(Value::as_array)
        .map_or(&[], Vec::as_slice)
}

fn s<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or("")
}

/// Id the Mixer and Voice Chain use for the master.
pub const MASTER_ID: &str = "master";
/// The output picker's value for "plays nowhere".
pub const NONE_ID: &str = "none";

/// The output a picker value stands for.
pub fn output_of(value: &str) -> Value {
    match value {
        MASTER_ID => json!({ "type": "master" }),
        NONE_ID => json!({ "type": "none" }),
        id => json!({ "type": "bus", "bus_id": id }),
    }
}

/// The picker value for an output.
pub fn output_value(o: Option<&Value>) -> String {
    match o {
        Some(o) if s(o, "type") == "bus" => s(o, "bus_id").to_string(),
        Some(o) if s(o, "type") == "none" => NONE_ID.to_string(),
        _ => MASTER_ID.to_string(),
    }
}

/// The tracks and buses a chain's effects listen to.
pub fn key_sources(chain: Option<&Value>) -> Vec<String> {
    let Some(chain) = chain else {
        return Vec::new();
    };
    arr(chain, "slots")
        .iter()
        .filter(|sl| sl.get("sidechain").is_some_and(truthy))
        .map(|sl| sl["sidechain"].as_str().unwrap_or("").to_string())
        .collect()
}

fn owners(doc: &Value) -> Vec<&Value> {
    arr(doc, "tracks").iter().chain(arr(doc, "buses")).collect()
}

/// Every edge as (from, to): outputs and sends to buses, and keys (source to listener).
fn edges(doc: &Value) -> Vec<(String, String)> {
    let buses: HashSet<&str> = arr(doc, "buses").iter().map(|b| s(b, "id")).collect();
    let all: HashSet<&str> = owners(doc).iter().map(|o| s(o, "id")).collect();
    let mut out = Vec::new();
    for o in owners(doc) {
        let id = s(o, "id");
        if let Some(op) = o.get("output") {
            if s(op, "type") == "bus" && buses.contains(s(op, "bus_id")) {
                out.push((id.to_string(), s(op, "bus_id").to_string()));
            }
        }
        for sd in arr(o, "sends") {
            if buses.contains(s(sd, "bus_id")) {
                out.push((id.to_string(), s(sd, "bus_id").to_string()));
            }
        }
        for k in key_sources(o.get("effects")) {
            if all.contains(k.as_str()) {
                out.push((k, id.to_string()));
            }
        }
    }
    out
}

/// True when sound from `from` reaches `to`.
pub fn reaches(doc: &Value, from: &str, to: &str) -> bool {
    let e = edges(doc);
    let mut stack = vec![from.to_string()];
    let mut seen = HashSet::new();
    while let Some(n) = stack.pop() {
        if n == to {
            return true;
        }
        if !seen.insert(n.clone()) {
            continue;
        }
        for (a, b) in &e {
            if *a == n {
                stack.push(b.clone());
            }
        }
    }
    false
}

/// Buses `owner` can output or send to without a loop.
pub fn bus_choices(doc: &Value, owner: &str) -> Vec<Value> {
    arr(doc, "buses")
        .iter()
        .filter(|b| s(b, "id") != owner && !reaches(doc, s(b, "id"), owner))
        .cloned()
        .collect()
}

/// Tracks and buses an effect on `owner`'s chain can listen to without a loop.
pub fn key_choices(doc: &Value, owner: &str) -> Vec<Value> {
    let ok = |id: &str| owner == MASTER_ID || (id != owner && !reaches(doc, owner, id));
    let mut out = Vec::new();
    for t in arr(doc, "tracks").iter().filter(|t| ok(s(t, "id"))) {
        out.push(json!({ "id": t["id"], "name": t["name"], "bus": false }));
    }
    for b in arr(doc, "buses").iter().filter(|b| ok(s(b, "id"))) {
        out.push(json!({ "id": b["id"], "name": b["name"], "bus": true }));
    }
    out
}

fn chain_active(c: Option<&Value>) -> bool {
    let Some(c) = c else {
        return false;
    };
    if !c.get("enabled").is_some_and(truthy) {
        return false;
    }
    let zero = |k: &str| c.get(k).and_then(Value::as_f64) == Some(0.0);
    !zero("input_gain_db")
        || !zero("output_gain_db")
        || arr(c, "slots")
            .iter()
            .any(|sl| sl.get("enabled").is_some_and(truthy))
}

/// True when the mix runs on the render thread (a track routed to a bus, a send, a master chain or a sidechain).
pub fn is_routed(doc: Option<&Value>) -> bool {
    let Some(doc) = doc else {
        return false;
    };
    chain_active(doc.get("master").and_then(|m| m.get("effects")))
        || arr(doc, "tracks").iter().any(|t| {
            let ty = t.get("output").map_or("", |o| s(o, "type"));
            ty == "bus"
                || ty == "none"
                || arr(t, "sends")
                    .iter()
                    .any(|sd| sd.get("enabled").is_some_and(truthy))
                || !key_sources(t.get("effects")).is_empty()
        })
}

/// Where `output` goes, as a label.
pub fn output_name(doc: &Value, output: Option<&Value>) -> String {
    if let Some(o) = output {
        if s(o, "type") == "none" {
            return "None".into();
        }
    }
    let Some(o) = output.filter(|o| s(o, "type") == "bus") else {
        return "Master".into();
    };
    arr(doc, "buses")
        .iter()
        .find(|b| s(b, "id") == s(o, "bus_id"))
        .and_then(|b| b.get("name").and_then(Value::as_str))
        .unwrap_or("Master")
        .to_string()
}

/// A new post-fader send to `bus_id` at 0 dB.
pub fn new_send(bus_id: &str, id: &str) -> Value {
    json!({ "id": id, "bus_id": bus_id, "level_db": 0, "pan": 0, "pre_fader": false, "enabled": true })
}

pub fn set_send(sends: &[Value], id: &str, patch: &Map<String, Value>) -> Vec<Value> {
    sends
        .iter()
        .map(|sd| {
            if s(sd, "id") == id {
                let mut m = sd.as_object().cloned().unwrap_or_default();
                for (k, v) in patch {
                    m.insert(k.clone(), v.clone());
                }
                Value::Object(m)
            } else {
                sd.clone()
            }
        })
        .collect()
}

pub fn remove_send(sends: &[Value], id: &str) -> Vec<Value> {
    sends
        .iter()
        .filter(|sd| s(sd, "id") != id)
        .cloned()
        .collect()
}

/// Bus changes as one op. With exclusive solo on, soloing a bus unsolos every other track and bus.
pub fn bus_op(doc: Option<&Value>, id: &str, changes: &Value, exclusive: bool) -> Value {
    let op = json!({ "op": "set_bus", "id": id, "changes": changes });
    let Some(doc) = doc else {
        return op;
    };
    if !exclusive || changes.get("solo") != Some(&Value::Bool(true)) {
        return op;
    }
    let mut ops = vec![op.clone()];
    for t in arr(doc, "tracks")
        .iter()
        .filter(|t| t.get("solo").is_some_and(truthy))
    {
        ops.push(json!({ "op": "set_track", "id": t["id"], "changes": { "solo": false } }));
    }
    for b in arr(doc, "buses")
        .iter()
        .filter(|b| s(b, "id") != id && b.get("solo").is_some_and(truthy))
    {
        ops.push(json!({ "op": "set_bus", "id": b["id"], "changes": { "solo": false } }));
    }
    if ops.len() == 1 {
        op
    } else {
        json!({ "op": "batch", "ops": ops })
    }
}

// ---- Duck Music Under Voice ----

/// The name stored on the slots this feature makes, so running it again replaces them.
pub const DUCK_PRESET: &str = "Duck under voice";

pub const DUCK_DEPTHS: [(&str, &str, f64); 3] = [
    ("light", "Light (6 dB)", 6.0),
    ("medium", "Medium (10 dB)", 10.0),
    ("deep", "Deep (15 dB)", 15.0),
];

/// Where the detector starts to react.
pub const DUCK_THRESHOLD_DB: f64 = -45.0;
/// A normal speaking level over the threshold, used to turn depth into a ratio.
const NOMINAL_OVER_DB: f64 = 25.0;

/// The Dynamics settings that drop the music by about `db` while a voice speaks at a normal level.
pub fn duck_params(db: f64) -> Value {
    let d = 1.0_f64.max(db.min(18.0));
    let ratio = 20.0_f64.min(1.0_f64.max(1.0 / (1.0 - d / NOMINAL_OVER_DB)));
    json!({
        "threshold": DUCK_THRESHOLD_DB,
        "ratio": js_round(ratio * 10.0) / 10.0,
        "attack": 30,
        "release": 500,
        "knee": 6,
        "makeup": 0,
        "linked": 1,
    })
}

/// One keyed Dynamics slot for one voice track.
pub fn duck_slot(id: &str, voice_id: &str, db: f64) -> Value {
    json!({
        "id": id,
        "effect": "altaudio.dynamics",
        "enabled": true,
        "params": duck_params(db),
        "preset": DUCK_PRESET,
        "sidechain": voice_id,
    })
}

/// True for a slot a previous run made.
pub fn is_duck_slot(sl: &Value) -> bool {
    s(sl, "preset") == DUCK_PRESET
        && s(sl, "effect") == "altaudio.dynamics"
        && sl.get("sidechain").is_some_and(truthy)
}

/// A voice track: has clips, is not muted, and is not all music, effects or ambience.
pub fn is_voice_track(t: &Value) -> bool {
    if t.get("muted").is_some_and(truthy) || arr(t, "clips").is_empty() {
        return false;
    }
    arr(t, "clips").iter().any(|c| {
        let ty = c.get("audio_type").unwrap_or(&Value::Null);
        !truthy(ty) || ty.as_str() == Some("dialogue")
    })
}

/// Voice tracks that can key a duck on `music`: not the music itself.
pub fn duck_keys(doc: &Value, music: &Value) -> Vec<Value> {
    arr(doc, "tracks")
        .iter()
        .filter(|t| s(t, "id") != s(music, "id") && is_voice_track(t))
        .cloned()
        .collect()
}

#[derive(Debug, Clone, PartialEq)]
pub struct DuckPlan {
    pub targets: Vec<Value>,
    pub keys: Vec<Value>,
    pub by_role: bool,
}

fn role_of(t: &Value) -> &str {
    t.get("mix_role")
        .and_then(Value::as_str)
        .unwrap_or("normal")
}

/// What Duck Music works on. None when there is nothing to duck.
pub fn duck_plan(doc: &Value, selected: Option<&Value>) -> Option<DuckPlan> {
    let targets: Vec<Value> = match selected {
        Some(sel) if role_of(sel) != "ducked" => vec![sel.clone()],
        _ => arr(doc, "tracks")
            .iter()
            .filter(|t| role_of(t) == "ducked")
            .cloned()
            .collect(),
    };
    if targets.is_empty() {
        return None;
    }
    let own: HashSet<&str> = targets.iter().map(|t| s(t, "id")).collect();
    let foreground: Vec<Value> = arr(doc, "tracks")
        .iter()
        .filter(|t| {
            role_of(t) == "foreground"
                && !t.get("muted").is_some_and(truthy)
                && !arr(t, "clips").is_empty()
                && !own.contains(s(t, "id"))
        })
        .cloned()
        .collect();
    let by_role = !foreground.is_empty();
    let keys = if by_role {
        foreground
    } else {
        arr(doc, "tracks")
            .iter()
            .filter(|t| !own.contains(s(t, "id")) && is_voice_track(t))
            .cloned()
            .collect()
    };
    Some(DuckPlan {
        targets,
        keys,
        by_role,
    })
}

/// True when the track already carries duck slots.
pub fn has_duck(music: &Value) -> bool {
    music
        .get("effects")
        .is_some_and(|e| arr(e, "slots").iter().any(is_duck_slot))
}

/// The music track's chain with its old duck slots replaced by new ones (one per voice), or None past `max` effects.
pub fn with_duck(
    chain: &Value,
    voice_ids: &[String],
    db: f64,
    new_id: &mut dyn FnMut() -> String,
    max: usize,
) -> Option<Value> {
    let kept: Vec<Value> = arr(chain, "slots")
        .iter()
        .filter(|sl| !is_duck_slot(sl))
        .cloned()
        .collect();
    let added: Vec<Value> = voice_ids
        .iter()
        .map(|v| duck_slot(&new_id(), v, db))
        .collect();
    if kept.len() + added.len() > max {
        return None;
    }
    let mut out = chain.as_object().cloned().unwrap_or_default();
    out.insert(
        "slots".into(),
        Value::Array(kept.into_iter().chain(added).collect()),
    );
    Some(Value::Object(out))
}

/// The chain with every duck slot taken out.
pub fn without_duck(chain: &Value) -> Value {
    let mut out = chain.as_object().cloned().unwrap_or_default();
    out.insert(
        "slots".into(),
        Value::Array(
            arr(chain, "slots")
                .iter()
                .filter(|sl| !is_duck_slot(sl))
                .cloned()
                .collect(),
        ),
    );
    Value::Object(out)
}

// ---- Sync Tracks ----

/// Drift below this is left unmentioned: under 20 ms over an hour.
pub const DRIFT_QUIET_PPM: f64 = 5.0;

/// What moving a track can come to.
#[derive(Debug, Clone, PartialEq)]
pub enum Move {
    Op(Value),
    BeforeStart,
    Blocked,
    Nothing,
}

/// A clip that has to move is locked.
#[derive(Debug, Clone, PartialEq)]
pub struct Locked;

/// Moves every clip on the track so the offset closes.
pub fn move_track(doc: &Value, track_id: &str, offset: f64) -> Result<Move, Locked> {
    let Some(track) = arr(doc, "tracks").iter().find(|t| s(t, "id") == track_id) else {
        return Ok(Move::Nothing);
    };
    if !arr(track, "midi_clips").is_empty() {
        return Ok(Move::Blocked);
    }
    let shift = js_round(offset);
    if shift == 0.0 || arr(track, "clips").is_empty() {
        return Ok(Move::Nothing);
    }
    let mut changes = Vec::new();
    for c in arr(track, "clips") {
        let start = c["start"].as_f64().unwrap_or(0.0) - shift;
        if start < 0.0 {
            return Ok(Move::BeforeStart);
        }
        if c.get("locked").is_some_and(truthy) {
            return Err(Locked);
        }
        changes.push(json!({ "id": c["id"], "start": start }));
    }
    Ok(Move::Op(json!({ "op": "set_clips", "changes": changes })))
}

/// The result of measuring two tracks.
#[derive(Debug, Clone, Copy)]
pub struct TrackSync {
    pub offset: f64,
    pub drift_ppm: f64,
    pub confidence: f64,
    pub inverted: bool,
}

/// The result in plain words, one line each.
pub fn describe_sync(r: &TrackSync, rate: f64) -> Vec<String> {
    let mut out = Vec::new();
    let secs = r.offset.abs() / rate;
    if js_round(r.offset) == 0.0 {
        out.push("The tracks already line up.".to_string());
    } else if r.offset > 0.0 {
        out.push(format!(
            "The second track is {} s late. Moving it earlier by that much lines it up.",
            js_to_fixed(secs, 3)
        ));
    } else {
        out.push(format!(
            "The second track starts {} s early. Moving it later by that much lines it up.",
            js_to_fixed(secs, 3)
        ));
    }
    let per_hour = r.drift_ppm.abs() * 3.6;
    if r.drift_ppm.abs() >= DRIFT_QUIET_PPM {
        out.push(format!(
            "The gap {} by about {} ms every hour ({} ppm). Moving the track fixes the start; the two will slip apart over a long recording.",
            if r.drift_ppm > 0.0 { "grows" } else { "shrinks" },
            js_to_fixed(per_hour, 0),
            js_to_fixed(r.drift_ppm.abs(), 0)
        ));
    } else {
        out.push(
            "The two clocks stay together, so a single move holds for the whole recording."
                .to_string(),
        );
    }
    if r.inverted {
        out.push("The second track is also upside down against the first. Flip its polarity if they sound thin together.".to_string());
    }
    out
}

/// How much to trust it, in a word.
pub fn trust(r: &TrackSync) -> &'static str {
    if r.confidence >= 0.5 {
        "strong"
    } else if r.confidence >= 0.25 {
        "fair"
    } else {
        "weak"
    }
}
