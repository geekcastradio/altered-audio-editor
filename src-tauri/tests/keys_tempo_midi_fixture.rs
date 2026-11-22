// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks the Rust key chords, library list, tempo map and MIDI edits against
//! `shared-fixtures/keys-tempo-midi.json`, which the TypeScript test reads too.

use altered_audio_editor_lib::chord::*;
use altered_audio_editor_lib::library_list::*;
use altered_audio_editor_lib::midi_edit as m;
use altered_audio_editor_lib::tempo_map as t;
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use std::collections::BTreeSet;

fn fixture() -> Value {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/keys-tempo-midi.json");
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

fn f(v: &Value) -> f64 {
    v.as_f64().unwrap()
}

fn s(v: &Value) -> &str {
    v.as_str().unwrap()
}

fn get<T: DeserializeOwned>(v: &Value) -> T {
    serde_json::from_value(v.clone()).unwrap()
}

fn val<T: serde::Serialize>(v: &T) -> Value {
    serde_json::to_value(v).unwrap()
}

/// Equal as JSON, with every number compared as a float (0 and 0.0 are the same).
fn same(a: &Value, b: &Value, what: &str) {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => {
            let (x, y) = (x.as_f64().unwrap(), y.as_f64().unwrap());
            assert!(
                (x - y).abs() <= 1e-9 * y.abs().max(1.0),
                "{what}: {x} vs {y}"
            );
        }
        (Value::Array(x), Value::Array(y)) => {
            assert_eq!(x.len(), y.len(), "{what}: length");
            for (p, q) in x.iter().zip(y) {
                same(p, q, what);
            }
        }
        (Value::Object(x), Value::Object(y)) => {
            assert_eq!(
                x.keys().collect::<Vec<_>>(),
                y.keys().collect::<Vec<_>>(),
                "{what}: keys"
            );
            for (k, p) in x {
                same(p, &y[k], what);
            }
        }
        _ => assert_eq!(a, b, "{what}"),
    }
}

fn opt(v: &Value) -> Option<String> {
    v.as_str().map(str::to_string)
}

#[test]
fn chords() {
    let fx = fixture();
    for c in fx["chords"].as_array().unwrap() {
        assert_eq!(
            parse_chord(s(&c["text"])),
            opt(&c["out"]),
            "chord {:?}",
            c["text"]
        );
    }
    for c in fx["events"].as_array().unwrap() {
        let e = &c["ev"];
        let ev = KeyEvent {
            key: s(&e["key"]).to_string(),
            ctrl: e["ctrlKey"].as_bool().unwrap(),
            alt: e["altKey"].as_bool().unwrap(),
            shift: e["shiftKey"].as_bool().unwrap(),
            meta: e["metaKey"].as_bool().unwrap(),
            code: opt(&e["code"]),
        };
        assert_eq!(chord_from_event(&ev), opt(&c["out"]), "event {e}");
    }
}

#[test]
fn library_lists() {
    let fx = fixture();
    for c in fx["libSan"].as_array().unwrap() {
        let raw = if c["raw"].get("__undefined").is_some() {
            Value::Null
        } else {
            c["raw"].clone()
        };
        same(&val(&sanitize_known(&raw)), &c["out"], "sanitize");
    }
    let big: Vec<Value> = (0..520)
        .map(|i| json!({ "dir": format!("/lib/{i}"), "name": format!("L{i}") }))
        .collect();
    assert_eq!(sanitize_known(&Value::Array(big)).len(), MAX_KNOWN);
    for c in fx["packs"].as_array().unwrap() {
        let known: Vec<KnownLibrary> = c["known"]
            .as_array()
            .unwrap()
            .iter()
            .map(|k| KnownLibrary {
                dir: s(&k["dir"]).to_string(),
                name: s(&k["name"]).to_string(),
                color: opt(&k["color"]),
            })
            .collect();
        assert_eq!(pack_library_list(&known), s(&c["text"]));
    }
    for c in fx["reads"].as_array().unwrap() {
        match read_library_list(s(&c["text"])) {
            Ok(list) => same(&val(&list), &c["out"], "read"),
            Err(e) => assert_eq!(e, s(&c["out"]["error"]), "read {:?}", c["text"]),
        }
    }
    for c in fx["segs"].as_array().unwrap() {
        assert_eq!(last_segment(s(&c["dir"])), s(&c["out"]));
    }
    for c in fx["sames"].as_array().unwrap() {
        assert_eq!(
            same_dir(s(&c["a"]), s(&c["b"])),
            c["out"].as_bool().unwrap()
        );
    }
}

fn tempo_of(v: &Value) -> t::Tempo {
    get(v)
}

fn seg_json(seg: &t::Segment) -> Value {
    val(seg)
}

#[test]
fn tempo_map() {
    let fx = fixture();
    for c in fx["tempo"].as_array().unwrap() {
        let d = tempo_of(&c["doc"]);
        same(&val(&t::segments(&d)), &c["segments"], "segments");
        for p in c["beats"].as_array().unwrap() {
            let b = f(&p["b"]);
            same(&json!(t::beats_to_seconds(&d, b)), &p["seconds"], "seconds");
            let at = t::bar_at(&d, b);
            same(
                &json!([at.bar, at.beat, at.seg.bar]),
                &json!([p["bar"], p["beat"], p["segBar"]]),
                "bar at",
            );
            same(&json!(t::nearest_bar(&d, b)), &p["nearest"], "nearest");
            same(
                &seg_json(&t::segment_at_beat(&d, b)),
                &p["segAt"],
                "segment at beat",
            );
        }
        for p in c["secs"].as_array().unwrap() {
            let x = f(&p["s"]);
            same(&json!(t::seconds_to_beats(&d, x)), &p["beats"], "beats");
            same(
                &seg_json(&t::segment_at_seconds(&d, x)),
                &p["segAt"],
                "segment at seconds",
            );
        }
        for p in c["bars"].as_array().unwrap() {
            let b = f(&p["b"]);
            same(&json!(t::bar_start(&d, b)), &p["start"], "bar start");
            same(
                &seg_json(&t::segment_at_bar(&d, b)),
                &p["segAt"],
                "segment at bar",
            );
        }
    }
    for c in fx["tempoEdits"].as_array().unwrap() {
        let mut d = tempo_of(&c["start"]);
        for step in c["steps"].as_array().unwrap() {
            match s(&step["kind"]) {
                "edit" => {
                    let v = &step["values"];
                    let values = t::Values {
                        bpm: v.get("bpm").map(f),
                        beats_per_bar: v.get("beats_per_bar").map(f),
                        beat_unit: v.get("beat_unit").map(f),
                    };
                    d = t::edit_segment(&d, step["id"].as_str(), &values);
                }
                "add" => {
                    let (next, id) = t::add_change(&d, f(&step["bar"]), s(&step["id"]));
                    assert_eq!(id, s(&step["outId"]));
                    d = next;
                }
                "move" => d = t::move_change(&d, s(&step["id"]), f(&step["bar"])),
                _ => d = t::remove_change(&d, s(&step["id"])),
            }
            same(&val(&d), &step["out"], "edit step");
        }
    }
    for c in fx["bpmTexts"].as_array().unwrap() {
        assert_eq!(t::bpm_text(f(&c["b"])), s(&c["out"]));
    }
    for c in fx["meters"].as_array().unwrap() {
        assert_eq!(
            t::meter_text(f(&c["bpb"]), c.get("unit").map(f)),
            s(&c["out"])
        );
    }
    for c in fx["meterTexts"].as_array().unwrap() {
        let got = t::parse_meter(s(&c["text"]));
        match got {
            None => assert!(c["out"].is_null(), "meter {:?}", c["text"]),
            Some((top, bottom)) => same(
                &json!({ "beats_per_bar": top, "beat_unit": bottom }),
                &c["out"],
                "meter",
            ),
        }
    }
}

fn ids(v: &Value) -> BTreeSet<String> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|x| s(x).to_string())
        .collect()
}

fn idx(v: &Value) -> BTreeSet<usize> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_u64().unwrap() as usize)
        .collect()
}

fn clips(v: &Value) -> Vec<m::Clip> {
    get(v)
}

fn notes(v: &Value) -> Vec<m::Note> {
    get(v)
}

#[test]
fn midi_math() {
    let fx = fixture();
    for (i, g) in fx["grids"].as_array().unwrap().iter().enumerate() {
        let (label, size) = m::GRIDS[i];
        assert_eq!(label, s(&g["label"]));
        same(&json!(size), &g["beats"], "grid size");
    }
    for c in fx["conv"].as_array().unwrap() {
        let d = tempo_of(&c["doc"]);
        let rate = f(&c["rate"]);
        let samples = m::beats_to_samples(f(&c["beats"]), &d, rate);
        same(&json!(samples), &c["samples"], "samples");
        same(
            &json!(m::samples_to_beats(samples, &d, rate)),
            &c["back"],
            "back",
        );
        same(
            &json!(m::samples_to_beats(
                (f(&c["beats"]) * 1000.0).round(),
                &d,
                rate
            )),
            &c["fromSamples"],
            "from samples",
        );
    }
    for c in fx["snaps"].as_array().unwrap() {
        same(&json!(m::snap(f(&c["b"]), f(&c["g"]))), &c["snap"], "snap");
        same(
            &json!(m::snap_down(f(&c["b"]), f(&c["g"]))),
            &c["down"],
            "snap down",
        );
    }
    for c in fx["gridCases"].as_array().unwrap() {
        same(
            &json!(m::grid_beats(f(&c["g"]), &tempo_of(&c["doc"]), f(&c["at"]))),
            &c["out"],
            "grid beats",
        );
    }
    for c in fx["names"].as_array().unwrap() {
        let p = c["p"].as_i64().unwrap();
        assert_eq!(m::note_name(p), s(&c["name"]), "name {p}");
        assert_eq!(m::is_black_key(p), c["black"].as_bool().unwrap());
    }
}

#[test]
fn midi_clips() {
    let fx = fixture();
    for c in fx["clipCases"].as_array().unwrap() {
        let d = tempo_of(&c["doc"]);
        let cl = clips(&c["clips"]);
        let set = ids(&c["ids"]);
        let name = c["name"].as_str().unwrap_or("MIDI");
        same(
            &val(&m::new_clip("nc", f(&c["at"]), &d, name)),
            &c["newClip"],
            "new clip",
        );
        assert_eq!(
            m::clip_at_beat(&cl, f(&c["beat"])).map(|x| x.id.clone()),
            opt(&c["clipAt"])
        );
        same(&val(&m::sorted(&cl)), &c["sorted"], "sorted");
        same(
            &val(&m::move_clips(&cl, &set, f(&c["delta"]))),
            &c["moved"],
            "moved",
        );
        let mut n = 0;
        let (from, to) = m::transfer_clips(
            &cl,
            &clips(&c["to"]),
            &set,
            f(&c["delta"]),
            c["copy"].as_bool().unwrap(),
            || {
                n += 1;
                format!("new{}", n - 1)
            },
        );
        same(
            &json!({ "from": from, "to": to }),
            &c["transferred"],
            "transferred",
        );
        same(
            &val(&m::resize_clip(&cl, s(&c["rid"]), f(&c["len"]), 0.25)),
            &c["resized"],
            "resized",
        );
        same(
            &val(&m::resize_clip(&cl, s(&c["rid"]), f(&c["len"]), 0.5)),
            &c["resizedMin"],
            "resized min",
        );
        same(&val(&m::remove_clips(&cl, &set)), &c["removed"], "removed");
        let mut n = 0;
        let (all, added) = m::duplicate_clips(&cl, &set, || {
            n += 1;
            format!("dup{}", n - 1)
        });
        same(
            &json!({ "clips": all, "added": added }),
            &c["duplicated"],
            "duplicated",
        );
        if !c["rep"].is_null() {
            let rep: m::Clip = get(&c["rep"]);
            same(
                &val(&m::replace_clip(&cl, &rep)),
                &c["replaced"],
                "replaced",
            );
        }
    }
}

#[test]
fn midi_notes() {
    let fx = fixture();
    for c in fx["noteCases"].as_array().unwrap() {
        let ns = notes(&c["notes"]);
        let set = idx(&c["idx"]);
        same(&val(&m::sort_notes(&ns)), &c["sorted"], "sorted");
        same(
            &val(&m::add_note(&ns, &get(&c["add"]))),
            &c["added"],
            "added",
        );
        same(
            &val(&m::move_notes(&ns, &set, f(&c["beats"]), f(&c["semis"]))),
            &c["moved"],
            "moved",
        );
        same(
            &val(&m::resize_notes(&ns, &set, f(&c["beats"]))),
            &c["resized"],
            "resized",
        );
        same(&val(&m::remove_notes(&ns, &set)), &c["removed"], "removed");
        same(
            &val(&m::set_velocity(&ns, &set, f(&c["vel"]))),
            &c["velocity"],
            "velocity",
        );
        let (lo, hi) = m::pitch_range(&ns);
        same(&json!([lo, hi]), &c["range"], "range");
        same(
            &val(&m::resize_notes_start(&ns, &set, f(&c["beats"]))),
            &c["startResized"],
            "start resized",
        );
        let copied = m::copy_notes(&ns, &set);
        same(&val(&copied), &c["copied"], "copied");
        same(&json!(m::span_of(&copied)), &c["span"], "span");
        let (all, pasted) = m::paste_notes(&ns, &copied, f(&c["at"]));
        same(
            &json!({ "notes": all, "pasted": pasted }),
            &c["pasted"],
            "pasted",
        );
    }
}
