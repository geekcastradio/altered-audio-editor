// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks the Waveform editor edits in Rust against `shared-fixtures/wave-edit.json`,
//! which the TypeScript test reads too.

use altered_audio_editor_lib::clip_edit::LockedError;
use altered_audio_editor_lib::wave_edit::*;
use serde_json::{json, Value};

fn fixture() -> Value {
    let p =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../shared-fixtures/wave-edit.json");
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

fn same(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => {
            let (x, y) = (x.as_f64().unwrap(), y.as_f64().unwrap());
            (x - y).abs() <= 1e-9 * (1.0 + x.abs().max(y.abs()))
        }
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(p, q)| same(p, q))
        }
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len() && x.iter().all(|(k, v)| y.get(k).is_some_and(|w| same(v, w)))
        }
        _ => a == b,
    }
}

fn ok(a: &Value, b: &Value, what: &str) {
    assert!(same(a, b), "{what}\n got {a}\nwant {b}");
}

fn f(v: &Value) -> f64 {
    v.as_f64().unwrap()
}

fn counter() -> impl FnMut() -> String {
    let mut n = 0;
    move || {
        n += 1;
        format!("n{n}0")
    }
}

fn locked(r: Result<Option<Value>, LockedError>) -> Value {
    match r {
        Ok(v) => v.unwrap_or(Value::Null),
        Err(LockedError) => json!({ "locked": true }),
    }
}

fn locked_pair(r: Result<Option<(Value, f64)>, LockedError>) -> Value {
    match r {
        Ok(Some((op, length))) => json!({ "op": op, "length": length }),
        Ok(None) => Value::Null,
        Err(LockedError) => json!({ "locked": true }),
    }
}

fn doc<'a>(fx: &'a Value, c: &Value) -> &'a Value {
    &fx["docs"][c["i"].as_u64().unwrap() as usize]
}

#[test]
fn ranges_and_markers() {
    let fx = fixture();
    for c in fx["clamp"].as_array().unwrap() {
        let got = clamp_range(doc(&fx, c), f(&c["a"]), f(&c["b"]))
            .map_or(Value::Null, |(a, b)| json!([a, b]));
        ok(&got, &c["out"], &format!("clamp {c}"));
    }
    let base = &fx["docs"][0];
    for c in fx["marks"].as_array().unwrap() {
        let (a, b) = (f(&c["a"]), f(&c["b"]));
        ok(
            &json!(markers_after_cut(base, a, b)),
            &c["cut"],
            &format!("cut {a} {b}"),
        );
        ok(
            &json!(markers_after_crop(base, a, b)),
            &c["crop"],
            &format!("crop markers {a} {b}"),
        );
        ok(
            &json!(markers_after_insert(base, a, b - a)),
            &c["ins"],
            &format!("insert markers {a} {b}"),
        );
    }
    for c in fx["fadeDb"].as_array().unwrap() {
        let got = fade_db(f(&c["t"]), c["s"].as_str().unwrap());
        ok(&json!(got), &c["out"], &format!("fade db {c}"));
    }
}

#[test]
fn range_edits() {
    let fx = fixture();
    for c in fx["del"].as_array().unwrap() {
        let got = locked(delete_range(
            doc(&fx, c),
            f(&c["a"]),
            f(&c["b"]),
            &mut counter(),
        ));
        ok(&got, &c["out"], &format!("delete {c}"));
    }
    for c in fx["crop"].as_array().unwrap() {
        let got = locked(crop_to(doc(&fx, c), f(&c["a"]), f(&c["b"]), &mut counter()));
        ok(&got, &c["out"], &format!("crop {}", c["a"]));
    }
    for c in fx["sil"].as_array().unwrap() {
        let got = locked(silence_range(
            doc(&fx, c),
            f(&c["a"]),
            f(&c["b"]),
            &mut counter(),
        ));
        ok(&got, &c["out"], &format!("silence {}", c["a"]));
    }
    for c in fx["amp"].as_array().unwrap() {
        let got = locked(amplify_range(
            doc(&fx, c),
            f(&c["a"]),
            f(&c["b"]),
            f(&c["db"]),
            &mut counter(),
        ));
        ok(&got, &c["out"], &format!("amplify {c}"));
    }
    for c in fx["fade"].as_array().unwrap() {
        let got = locked(fade_range(
            doc(&fx, c),
            f(&c["a"]),
            f(&c["b"]),
            c["d"].as_str().unwrap(),
            &mut counter(),
            c["s"].as_str().unwrap(),
        ));
        ok(&got, &c["out"], &format!("fade {} {}", c["a"], c["d"]));
    }
}

#[test]
fn copy_and_paste() {
    let fx = fixture();
    // The first cases copy from the base file, the last 3 from the two track file.
    let base = &fx["docs"][0];
    let two = &fx["docs"][1];
    let list = fx["copy"].as_array().unwrap();
    for (n, c) in list.iter().enumerate() {
        let src = if n < list.len() - 3 { base } else { two };
        let got = copy_range(src, f(&c["a"]), f(&c["b"])).unwrap_or(Value::Null);
        ok(&got, &c["out"], &format!("copy {n}"));
    }
    let boards = fx["boards"].as_array().unwrap();
    for c in fx["mix"].as_array().unwrap() {
        let board = &boards[c["bi"].as_u64().unwrap() as usize];
        let got = locked_pair(Ok(mix_paste_at(
            doc(&fx, c),
            board,
            f(&c["at"]),
            f(&c["db"]),
            &mut counter(),
        )));
        ok(
            &got,
            &c["out"],
            &format!("mix paste {} {}", c["bi"], c["at"]),
        );
    }
    for c in fx["paste"].as_array().unwrap() {
        let board = &boards[c["bi"].as_u64().unwrap() as usize];
        let got = locked_pair(paste_at(doc(&fx, c), board, f(&c["at"]), &mut counter()));
        ok(&got, &c["out"], &format!("paste {} {}", c["bi"], c["at"]));
    }
}

#[test]
fn joins_replace_stretch_insert() {
    let fx = fixture();
    let shape = fx["shape"].as_str().unwrap();
    for c in fx["seams"].as_array().unwrap() {
        let seams: Vec<f64> = c["s"].as_array().unwrap().iter().map(f).collect();
        let got = crossfade_seams(doc(&fx, c), &seams, f(&c["len"]), shape).map_or(
            Value::Null,
            |(op, joins)| json!({ "op": op, "joins": joins }),
        );
        ok(&got, &c["out"], &format!("seams {c}"));
    }
    for c in fx["repl"].as_array().unwrap() {
        let got = locked(replace_range(
            doc(&fx, c),
            f(&c["a"]),
            f(&c["b"]),
            &c["m"],
            "Fixed",
            &mut counter(),
        ));
        ok(&got, &c["out"], &format!("replace {} {}", c["a"], c["b"]));
    }
    for c in fx["str"].as_array().unwrap() {
        let got = locked(stretch_range(
            doc(&fx, c),
            f(&c["a"]),
            f(&c["b"]),
            &c["m"],
            "Slow",
            &mut counter(),
        ));
        ok(&got, &c["out"], &format!("stretch {} {}", c["a"], c["b"]));
    }
    let media = [&fx["media"]["mediaA"], &fx["media"]["mediaB"]];
    let zero = json!({ "id": "mn", "sample_rate": 48000, "length": 0 });
    for c in fx["ins"].as_array().unwrap() {
        let mi = c["mi"].as_u64().unwrap() as usize;
        let m = if mi < 2 { media[mi] } else { &zero };
        let got = locked_pair(insert_file(
            doc(&fx, c),
            f(&c["at"]),
            m,
            "Take",
            &mut counter(),
        ));
        ok(&got, &c["out"], &format!("insert {} {}", c["at"], mi));
    }
    let empty = fx["empty"].as_array().unwrap();
    let no_tracks = json!({ "sample_rate": 48000, "media": [], "markers": [], "tracks": [] });
    ok(
        &locked(delete_range(&no_tracks, 0.0, 10.0, &mut counter())),
        &empty[0],
        "empty delete",
    );
    ok(
        &locked_pair(insert_file(&no_tracks, 0.0, media[0], "x", &mut counter())),
        &empty[1],
        "empty insert",
    );
    let none = crossfade_seams(&no_tracks, &[96000.0], 10.0, shape).map_or(
        Value::Null,
        |(op, joins)| json!({ "op": op, "joins": joins }),
    );
    ok(&none, &empty[2], "empty seams");
}
