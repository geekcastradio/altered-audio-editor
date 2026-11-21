// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Audio probe: opens the sound hardware the way the editor does and prints
//! what happened, so a tester can send the output with a bug report. It
//! plays a quiet tone for one second on each output, nothing is saved, and
//! nothing leaves the computer.
//!
//! Run it from `src-tauri`:
//!
//! ```text
//! cargo run --example audio_probe
//! cargo run --example audio_probe -- --asio       (ASIO builds on Windows)
//! cargo run --example audio_probe -- --buffer 256 (ask for a buffer size)
//! ```

use std::time::Duration;

use altered_audio_editor_lib::engine::host;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SampleFormat, SizedSample};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let asio = args.iter().any(|a| a == "--asio");
    let buffer: u32 = args
        .iter()
        .position(|a| a == "--buffer")
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    host::set_asio(asio);
    host::set_buffer_frames(buffer);

    println!("Altered Audio Editor audio probe");
    println!(
        "ASIO in this build: {}",
        if host::ASIO_BUILT { "yes" } else { "no" }
    );
    println!("ASIO requested: {}", if asio { "yes" } else { "no" });
    println!("ASIO drivers installed: {:?}", host::asio_drivers());
    println!(
        "Buffer requested: {}",
        if buffer > 0 {
            format!("{buffer} frames")
        } else {
            "the driver's own".into()
        }
    );
    let h = host::host();
    println!("Host in use: {}", h.id().name());

    let outputs = match h.output_devices() {
        Ok(d) => d.collect::<Vec<_>>(),
        Err(e) => {
            println!("Could not list outputs: {e}");
            return;
        }
    };
    if outputs.is_empty() {
        println!("No output devices found.");
    }
    for (i, d) in outputs.iter().enumerate() {
        let name = d
            .description()
            .map_or_else(|e| format!("(unnamed: {e})"), |x| x.name().to_string());
        println!("\nOutput {i}: {name}");
        match d.default_output_config() {
            Ok(c) => {
                println!(
                    "  default: {} Hz, {} channels, {:?}, buffer {:?}",
                    c.sample_rate(),
                    c.channels(),
                    c.sample_format(),
                    c.buffer_size()
                );
                match play(d, &c) {
                    Ok(()) => println!("  result: opened and played for one second"),
                    Err(e) => println!("  result: FAILED: {e}"),
                }
            }
            Err(e) => println!("  no default configuration: {e}"),
        }
    }

    let inputs = h.input_devices().map(|d| d.count()).unwrap_or(0);
    println!("\nInput devices found: {inputs}");
}

fn play(device: &cpal::Device, supported: &cpal::SupportedStreamConfig) -> Result<(), String> {
    let config = host::with_buffer(supported.config(), supported.buffer_size());
    match supported.sample_format() {
        SampleFormat::F32 => run::<f32>(device, &config),
        SampleFormat::I16 => run::<i16>(device, &config),
        SampleFormat::I32 => run::<i32>(device, &config),
        other => Err(format!("sample format {other:?} is not used by the editor")),
    }
}

fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig) -> Result<(), String>
where
    T: SizedSample + FromSample<f32>,
{
    let channels = usize::from(config.channels);
    let rate = config.sample_rate as f32;
    let mut phase = 0f32;
    let stream = device
        .build_output_stream(
            *config,
            move |out: &mut [T], _| {
                for frame in out.chunks_mut(channels) {
                    let v = (phase * std::f32::consts::TAU).sin() * 0.1;
                    phase = (phase + 440.0 / rate).fract();
                    for s in frame {
                        *s = T::from_sample(v);
                    }
                }
            },
            |e| eprintln!("  stream error: {e}"),
            None,
        )
        .map_err(|e| e.to_string())?;
    stream.play().map_err(|e| e.to_string())?;
    std::thread::sleep(Duration::from_secs(1));
    Ok(())
}
