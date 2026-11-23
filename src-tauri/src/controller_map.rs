// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Hardware controller bindings in Rust: what a message from a fader box is
//! bound to, the names shown in the list, the standard eight-strip preset,
//! and what the screen tells the surface back (fader positions, button
//! lights, pan rings). Mirrors `src/app/controller/map.ts` and
//! `feedback.ts`, tested against `shared-fixtures/controller-map.json`.

use crate::fader::db_to_fader;
use crate::js_compat::js_round;
use serde_json::{json, Value};

/// A menu command a controller can run: (id, label).
pub const ACTIONS: [(&str, &str); 10] = [
    ("multitrack.play-pause", "Play / Stop"),
    ("multitrack.stop", "Stop"),
    ("multitrack.record", "Record"),
    ("transport.go-start", "Go to Start"),
    ("transport.go-end", "Go to End"),
    ("transport.prev-marker", "Previous Marker"),
    ("transport.next-marker", "Next Marker"),
    ("edit.undo", "Undo"),
    ("edit.redo", "Redo"),
    ("file.save", "Save"),
];

fn text<'a>(v: &'a Value, k: &str) -> &'a str {
    v.get(k).and_then(Value::as_str).unwrap_or("")
}

fn num(v: &Value, k: &str) -> f64 {
    v.get(k).and_then(Value::as_f64).unwrap_or(0.0)
}

fn int(v: &Value, k: &str) -> i64 {
    num(v, k) as i64
}

/// One string per source, so a source bound twice can be found.
pub fn source_key(s: &Value) -> String {
    format!(
        "{}:{}:{}",
        text(s, "kind"),
        int(s, "channel"),
        int(s, "number")
    )
}

/// Whether a binding listens to a message.
pub fn matches(b: &Value, m: &Value) -> bool {
    let s = &b["source"];
    s["kind"] == m["kind"] && s["channel"] == m["channel"] && s["number"] == m["number"]
}

/// A message's position: 0 to 1.
pub fn position(kind: &str, value: f64) -> f64 {
    if kind == "bend" {
        value / 16383.0
    } else {
        value / 127.0
    }
}

/// How far a relative encoder turned: 1 to 63 is up, 65 to 127 is down.
pub fn turn(value: f64) -> f64 {
    if value == 0.0 || value == 64.0 {
        0.0
    } else if value < 64.0 {
        value
    } else {
        -(value - 64.0)
    }
}

/// A pan from a 0 to 1 position, with a small dead zone so center is easy to find.
pub fn pan_at(pos: f64) -> f64 {
    let p = js_round(1.0_f64.min(0.0_f64.max(pos)) * 200.0 - 100.0);
    if p.abs() <= 3.0 {
        0.0
    } else {
        p
    }
}

/// The name of a source for the list: "Fader, channel 3".
pub fn source_name(s: &Value) -> String {
    let ch = format!("channel {}", int(s, "channel") + 1);
    match text(s, "kind") {
        "bend" => format!("Fader, {ch}"),
        "note" => format!("Button {}, {ch}", int(s, "number")),
        _ => format!("Knob {}, {ch}", int(s, "number")),
    }
}

fn track_label(t: &str) -> &'static str {
    match t {
        "volume" => "Volume",
        "pan" => "Pan",
        "mute" => "Mute",
        "solo" => "Solo",
        "arm" => "Arm",
        _ => "Fader Touch",
    }
}

/// The name of a target for the list.
pub fn target_name(t: &Value) -> String {
    match text(t, "type") {
        "master" => "Master Volume".to_string(),
        "bank" => {
            let by = int(t, "by");
            match by {
                8 => "Next 8 Tracks".into(),
                -8 => "Previous 8 Tracks".into(),
                1 => "Next Track".into(),
                -1 => "Previous Track".into(),
                _ => format!("Move {by} Tracks"),
            }
        }
        "action" => {
            let id = text(t, "action");
            ACTIONS
                .iter()
                .find(|a| a.0 == id)
                .map_or_else(|| id.to_string(), |a| a.1.to_string())
        }
        other => format!("Track {} {}", int(t, "track") + 1, track_label(other)),
    }
}

/// Every target a person can pick in the list, for tracks 1 to `tracks`.
pub fn target_choices(tracks: usize) -> Vec<Value> {
    let mut out = vec![json!({ "type": "master" })];
    for i in 0..tracks {
        for ty in ["volume", "pan", "mute", "solo", "arm"] {
            out.push(json!({ "type": ty, "track": i }));
        }
    }
    for (id, _) in ACTIONS {
        out.push(json!({ "type": "action", "action": id }));
    }
    for by in [-8, 8, -1, 1] {
        out.push(json!({ "type": "bank", "by": by }));
    }
    out
}

/// The standard eight-strip layout: what a fader box sends in its Mackie mode.
pub fn mackie_preset() -> Vec<Value> {
    let mut out = Vec::new();
    for i in 0..8 {
        out.push(json!({ "source": { "kind": "bend", "channel": i, "number": 0 }, "target": { "type": "volume", "track": i } }));
        out.push(json!({ "source": { "kind": "note", "channel": 0, "number": 104 + i }, "target": { "type": "touch", "track": i } }));
        out.push(json!({ "source": { "kind": "cc", "channel": 0, "number": 16 + i }, "target": { "type": "pan", "track": i }, "relative": true }));
        out.push(json!({ "source": { "kind": "note", "channel": 0, "number": i }, "target": { "type": "arm", "track": i } }));
        out.push(json!({ "source": { "kind": "note", "channel": 0, "number": 8 + i }, "target": { "type": "solo", "track": i } }));
        out.push(json!({ "source": { "kind": "note", "channel": 0, "number": 16 + i }, "target": { "type": "mute", "track": i } }));
    }
    out.push(json!({ "source": { "kind": "bend", "channel": 8, "number": 0 }, "target": { "type": "master" } }));
    let b = |number: i64, target: Value| json!({ "source": { "kind": "note", "channel": 0, "number": number }, "target": target });
    out.push(b(46, json!({ "type": "bank", "by": -8 })));
    out.push(b(47, json!({ "type": "bank", "by": 8 })));
    out.push(b(48, json!({ "type": "bank", "by": -1 })));
    out.push(b(49, json!({ "type": "bank", "by": 1 })));
    for (n, a) in [
        (91, "transport.go-start"),
        (92, "transport.go-end"),
        (93, "multitrack.stop"),
        (94, "multitrack.play-pause"),
        (95, "multitrack.record"),
    ] {
        out.push(b(n, json!({ "type": "action", "action": a })));
    }
    out
}

/// A fader position, 0 to 1, as a 14-bit pitch bend value.
pub fn fader14(pos: f64) -> f64 {
    js_round(1.0_f64.min(0.0_f64.max(pos)) * 16383.0)
}

/// The pan ring value for a pan of -100 to 100: a lit point from 1 (left) to 11 (right).
pub fn ring_byte(pan: f64) -> f64 {
    let p = 100.0_f64.min((-100.0_f64).max(pan));
    16.0 + js_round(((p + 100.0) / 200.0) * 10.0) + 1.0
}

fn bend(channel: i64, value: f64) -> Vec<i64> {
    let v = value as i64;
    vec![0xe0 | channel, v & 0x7f, v >> 7]
}

/// Everything the surface should show for these bindings. `strips[i]` is the
/// strip on the surface's channel `i` (already offset by the bank); a channel
/// with no strip gets its fader at the bottom and its lights off. Each frame
/// is `{ key, bytes, hold? }`.
pub fn frames(bindings: &[Value], strips: &[Option<Value>], master_db: f64) -> Vec<Value> {
    let mut out = Vec::new();
    for b in bindings {
        let t = &b["target"];
        let ty = text(t, "type");
        let s = t
            .get("track")
            .and_then(Value::as_u64)
            .and_then(|i| strips.get(i as usize))
            .and_then(Option::as_ref);
        let (kind, channel, number) = (
            text(&b["source"], "kind"),
            int(&b["source"], "channel"),
            int(&b["source"], "number"),
        );
        let relative = b.get("relative") == Some(&json!(true));
        if ty == "volume" && kind == "bend" && !relative {
            let pos = s.map_or(0.0, |s| db_to_fader(num(s, "volume_db")));
            let mut f =
                json!({ "key": format!("v:{channel}"), "bytes": bend(channel, fader14(pos)) });
            if let Some(s) = s {
                f["hold"] = json!(format!("{}:volume", text(s, "id")));
            }
            out.push(f);
        } else if ty == "master" && kind == "bend" && !relative {
            out.push(json!({ "key": format!("m:{channel}"), "bytes": bend(channel, fader14(db_to_fader(master_db))) }));
        } else if matches!(ty, "mute" | "solo" | "arm") && kind == "note" {
            let flag = match ty {
                "mute" => "muted",
                "solo" => "solo",
                _ => "record_armed",
            };
            let on = s.is_some_and(|s| s.get(flag) == Some(&json!(true)));
            out.push(json!({ "key": format!("n:{channel}:{number}"), "bytes": [0x90 | channel, number, if on { 127 } else { 0 }] }));
        } else if ty == "pan" && kind == "cc" && relative && (16..=23).contains(&number) {
            let ring = s.map_or(0.0, |s| ring_byte(num(s, "pan")));
            out.push(json!({ "key": format!("r:{channel}:{number}"), "bytes": [0xb0 | channel, number + 32, ring as i64] }));
        }
    }
    out
}
