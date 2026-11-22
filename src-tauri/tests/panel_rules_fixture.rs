// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks the Rust EQ graph, phase curve, band split, channel mix, Music Fit
//! text and Files format rules against `shared-fixtures/panel-rules.json`,
//! which the TypeScript tests read too.

use altered_audio_editor_lib::clip_props::*;
use altered_audio_editor_lib::file_format::*;
use altered_audio_editor_lib::voice_chain_math::*;
use serde_json::Value;

fn fixture() -> Value {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/panel-rules.json");
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

fn f(v: &Value) -> f64 {
    v.as_f64().unwrap()
}

fn near(a: f64, b: f64, what: &Value) {
    let tol = 1e-9 * b.abs().max(1.0);
    assert!((a - b).abs() <= tol, "{what}: {a} vs {b}");
}

fn params(v: &Value) -> Params {
    v.as_object()
        .unwrap()
        .iter()
        .map(|(k, x)| (k.clone(), f(x)))
        .collect()
}

fn opt_params(v: &Value) -> Option<Params> {
    (!v.is_null()).then(|| params(v))
}

fn sends(v: &Value) -> Vec<Send> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|s| Send {
            left: f(&s["left"]),
            right: f(&s["right"]),
        })
        .collect()
}

fn strs(v: &Value) -> Vec<String> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect()
}

#[test]
fn eq_graph() {
    for c in fixture()["eq"].as_array().unwrap() {
        if c.get("dir").is_some() {
            if c["dir"] == "octaves" {
                near(octaves_of(f(&c["q"])), f(&c["oct"]), c);
            } else {
                near(q_of(f(&c["oct"])), f(&c["q"]), c);
            }
        } else if c.get("edges").is_some() {
            let (a, b) = width_edges(f(&c["f"]), f(&c["q"]));
            near(a, f(&c["edges"][0]), c);
            near(b, f(&c["edges"][1]), c);
        } else if c.get("qe").is_some() {
            near(q_from_edge(f(&c["f"]), f(&c["at"])), f(&c["qe"]), c);
        } else {
            assert_eq!(format_hz(f(&c["hz"])), c["text"].as_str().unwrap(), "{c}");
        }
    }
}

#[test]
fn phase_curve() {
    let fx = fixture();
    for c in fx["phase"].as_array().unwrap() {
        let p = params(&c["params"]);
        assert_eq!(point_count(&p) as f64, f(&c["count"]), "{c}");
        assert_eq!(range_of(&p), f(&c["range"]), "{c}");
        let got = curve_points(&p);
        let want = c["pts"].as_array().unwrap();
        assert_eq!(got.len(), want.len(), "{c}");
        for (g, w) in got.iter().zip(want) {
            assert_eq!(g.index as f64, f(&w["index"]));
            near(g.hz, f(&w["hz"]), c);
            near(g.deg, f(&w["deg"]), c);
        }
    }
    for c in fx["curve"].as_array().unwrap() {
        let pts = curve_points(&params(&c["params"]));
        near(
            curve_at(&pts, c["spline"].as_bool().unwrap(), f(&c["hz"])),
            f(&c["out"]),
            c,
        );
    }
    for c in fx["edit"].as_array().unwrap() {
        let p = params(&c["params"]);
        assert_eq!(add_point(&p, 1234.0, -33.0), opt_params(&c["add"]), "{c}");
        assert_eq!(add_point(&p, 0.0, 0.0), opt_params(&c["adds"]), "{c}");
        for (i, idx) in [0usize, 1, 2, 3, 7, 9].iter().enumerate() {
            assert_eq!(remove_point(&p, *idx), opt_params(&c["rm"][i]), "{c} {idx}");
        }
    }
    for c in fx["groups"].as_array().unwrap() {
        let p = params(&c["params"]);
        let g = c["g"].as_str().unwrap();
        assert_eq!(group_shown(true, &p, g), c["on"].as_bool().unwrap(), "{c}");
        assert_eq!(
            group_shown(false, &p, g),
            c["other"].as_bool().unwrap(),
            "{c}"
        );
    }
}

#[test]
fn band_split() {
    let fx = fixture();
    for c in fx["edges"].as_array().unwrap() {
        let got = parse_edges(c["t"].as_str().unwrap());
        match (&c["out"], got) {
            (Value::Null, None) => {}
            (w, Some(g)) => {
                let w = w.as_array().unwrap();
                assert_eq!(g.len(), w.len(), "{c}");
                for (a, b) in g.iter().zip(w) {
                    near(*a, f(b), c);
                }
            }
            other => panic!("{c}: {other:?}"),
        }
    }
    for c in fx["checks"].as_array().unwrap() {
        let e: Vec<f64> = c["e"].as_array().unwrap().iter().map(f).collect();
        assert_eq!(check_edges(&e).as_deref(), c["out"].as_str(), "{c}");
    }
    for c in fx["bands"].as_array().unwrap() {
        let e: Vec<f64> = c["e"].as_array().unwrap().iter().map(f).collect();
        let got = band_slot_specs(c["k"].as_u64().unwrap() as usize, &e);
        let want = c["out"].as_array().unwrap();
        assert_eq!(got.len(), want.len(), "{c}");
        for (g, w) in got.iter().zip(want) {
            assert_eq!(g.0 as f64, f(&w[0]));
            assert_eq!(g.1, f(&w[1]));
        }
    }
}

#[test]
fn channel_mix() {
    let fx = fixture();
    for c in fx["labels"].as_array().unwrap() {
        assert_eq!(
            channel_labels(c["n"].as_u64().unwrap() as usize),
            strs(&c["out"]),
            "{c}"
        );
    }
    for c in fx["mixes"].as_array().unwrap() {
        let n = c["n"].as_u64().unwrap() as usize;
        let near_mix = |got: Vec<Send>, want: &Value| {
            let want = sends(want);
            assert_eq!(got.len(), want.len(), "{c}");
            for (g, w) in got.iter().zip(&want) {
                near(g.left, w.left, &Value::Null);
                near(g.right, w.right, &Value::Null);
            }
        };
        near_mix(default_mix(n), &c["def"]);
        near_mix(
            mix_of(
                Some(&[Send {
                    left: 0.3,
                    right: 0.7,
                }]),
                n,
            ),
            &c["filled"],
        );
        near_mix(mix_of(None, n), &c["none"]);
    }
    for c in fx["presets"].as_array().unwrap() {
        let n = c["n"].as_u64().unwrap() as usize;
        let got = presets(n);
        let want = c["out"].as_array().unwrap();
        assert_eq!(got.len(), want.len(), "{c}");
        for (g, w) in got.iter().zip(want) {
            assert_eq!(g.id, w["id"].as_str().unwrap());
            assert_eq!(g.label, w["label"].as_str().unwrap());
            let ws = sends(&w["mix"]["sends"]);
            assert_eq!(g.mix.len(), ws.len());
            for (a, b) in g.mix.iter().zip(&ws) {
                near(a.left, b.left, w);
                near(a.right, b.right, w);
            }
        }
        for (p, m) in got.iter().zip(c["match"].as_array().unwrap()) {
            assert_eq!(preset_for(&p.mix, n).as_deref(), m.as_str(), "{c}");
        }
        assert_eq!(
            preset_for(
                &[Send {
                    left: 0.9,
                    right: 0.1
                }],
                n
            )
            .as_deref(),
            c["odd"].as_str(),
            "{c}"
        );
    }
}

#[test]
fn music_fit_text() {
    for c in fixture()["fits"].as_array().unwrap() {
        if let Some(s) = c.get("same") {
            assert_eq!(
                same_length(f(&s[0]), f(&s[1])),
                c["out"].as_bool().unwrap(),
                "{c}"
            );
            continue;
        }
        let p = &c["p"];
        let plan = FitPlan {
            segments: p["segments"]
                .as_array()
                .unwrap()
                .iter()
                .map(|s| (f(&s["start"]), f(&s["end"])))
                .collect(),
            joins: p["joins"].as_u64().unwrap() as u32,
            quality: f(&p["quality"]),
            stretch: f(&p["stretch"]),
            tempo: f(&p["tempo"]),
        };
        assert_eq!(describe_plan(&plan), c["plan"].as_str().unwrap(), "{c}");
        assert_eq!(
            describe_joins(&plan, f(&c["origin"])),
            c["joins"].as_str().unwrap(),
            "{c}"
        );
    }
}

#[test]
fn files_formats() {
    for c in fixture()["files"].as_array().unwrap() {
        let want = c["out"].as_str().unwrap();
        if let Some(d) = c.get("dur") {
            assert_eq!(format_duration(f(&d[0]), f(&d[1])), want, "{c}");
        } else if let Some(h) = c.get("hz") {
            assert_eq!(format_rate(f(h)), want, "{c}");
        } else if let Some(n) = c.get("ch") {
            assert_eq!(format_channels(n.as_u64().unwrap() as u32), want, "{c}");
        } else if let Some(b) = c.get("bytes") {
            assert_eq!(format_size(f(b)), want, "{c}");
        } else {
            assert_eq!(status_text(c["st"].as_str().unwrap()), Some(want), "{c}");
        }
    }
}
