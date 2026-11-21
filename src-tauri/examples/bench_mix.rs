// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Speed benchmark: mixes a dense session as fast as possible and reports how
//! many times faster than real time it ran. The session has many mono tracks,
//! each with a Voice Chain, built from generated noise, so the result does not
//! depend on any file. Nothing leaves the computer.
//!
//! Run it from `src-tauri`, in release mode (debug builds are far slower):
//!
//! ```text
//! cargo run --release --example bench_mix
//! cargo run --release --example bench_mix -- --tracks 20 --seconds 120
//! cargo run --release --example bench_mix -- --plain   (no effects, mixing only)
//! ```

use std::sync::atomic::AtomicBool;
use std::time::Instant;

use altered_audio_editor_lib::export::mixdown::{mixdown, MixdownRequest};
use altered_audio_editor_lib::session::model::{
    new_id, ChannelLayout, EffectSlot, MediaRef, Session, Track,
};
use altered_audio_editor_lib::session::ops::{apply, ClipSpec, Op};

const RATE: u32 = 48_000;
const CHAIN: [&str; 4] = [
    "altaudio.parametric_eq",
    "altaudio.dynamics",
    "altaudio.limiter",
    "altaudio.breath_gate",
];

fn arg(args: &[String], name: &str, default: u64) -> u64 {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let tracks = arg(&args, "--tracks", 48).max(1) as usize;
    let seconds = arg(&args, "--seconds", 60).max(1);
    let runs = arg(&args, "--runs", 3).max(1);
    let plain = args.iter().any(|a| a == "--plain");

    let dir = tempfile::tempdir().expect("temp folder");
    let frames = seconds * u64::from(RATE);
    let media = dir.path().join("noise.wav");
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(&media, spec).expect("write noise");
    let mut seed = 0x1234_5678_u32;
    for _ in 0..frames {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        w.write_sample(((seed >> 16) as i16) / 4).expect("sample");
    }
    w.finalize().expect("finish noise");

    let mut s = Session::new("bench", RATE);
    let m = MediaRef {
        id: new_id(),
        path: media,
        relative_path: None,
        sample_rate: RATE,
        channels: 1,
        length: frames,
    };
    let mid = m.id;
    s.media.push(m);
    for i in 0..tracks {
        let mut t = Track::new(format!("Track {}", i + 1), ChannelLayout::Mono);
        if !plain {
            t.effects.slots = CHAIN
                .iter()
                .map(|id| EffectSlot {
                    id: new_id(),
                    effect: (*id).into(),
                    enabled: true,
                    params: Default::default(),
                    preset: None,
                    sidechain: None,
                    plugin_state: None,
                    data: None,
                    position: None,
                })
                .collect();
        }
        let tid = t.id;
        s.tracks.push(t);
        apply(
            &mut s,
            &Op::AddClips {
                track_id: tid,
                clips: vec![ClipSpec {
                    id: new_id(),
                    name: "noise".into(),
                    media_id: mid,
                    start: 0,
                    source_in: 0,
                    source_out: frames,
                }],
            },
        )
        .expect("add clip");
    }

    println!("Altered Audio Editor speed benchmark");
    println!(
        "{tracks} mono tracks, {seconds} s each, {}",
        if plain {
            "no effects"
        } else {
            "Voice Chain: EQ, Dynamics, Limiter, Breath Gate"
        }
    );
    println!(
        "Cores available: {}",
        std::thread::available_parallelism().map_or(1, std::num::NonZero::get)
    );

    let out = dir.path().join("mix.wav");
    let req = MixdownRequest::default();
    let mut best = f64::MAX;
    for run in 1..=runs {
        let t0 = Instant::now();
        let r = mixdown(&s, &req, &out, &AtomicBool::new(false), |_, _| {}).expect("mixdown");
        let took = t0.elapsed().as_secs_f64();
        let audio = r.frames as f64 / f64::from(r.sample_rate);
        println!(
            "Run {run}: {took:.2} s for {audio:.0} s of audio: {:.1}x real time",
            audio / took
        );
        best = best.min(took);
    }
    println!("Best: {:.1}x real time", seconds as f64 / best);
}
