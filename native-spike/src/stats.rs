// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Frame timing and a small log written beside the program.

use std::io::Write;
use std::time::Instant;

#[derive(Default)]
pub struct FrameStats {
    last: Option<Instant>,
    times: Vec<f32>,
}

impl FrameStats {
    pub fn tick(&mut self) {
        let now = Instant::now();
        if let Some(prev) = self.last {
            self.times.push(now.duration_since(prev).as_secs_f32() * 1000.0);
            if self.times.len() > 300 {
                self.times.remove(0);
            }
        }
        self.last = Some(now);
    }

    /// Average and 99th percentile of the last 300 frames.
    pub fn summary(&self) -> String {
        if self.times.is_empty() {
            return "no frames yet".into();
        }
        let avg = self.times.iter().sum::<f32>() / self.times.len() as f32;
        let mut sorted = self.times.clone();
        sorted.sort_by(|a, b| a.total_cmp(b));
        let p99 = sorted[((sorted.len() - 1) as f32 * 0.99) as usize];
        format!("frame {avg:.1} ms avg, {p99:.1} ms worst 1%  ({:.0} fps)", 1000.0 / avg)
    }
}

/// Prints a timestamped line and appends it to `spike-timing.log` in the working folder.
pub fn log(line: &str) {
    eprintln!("{line}");
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("spike-timing.log")
    {
        let _ = writeln!(f, "{line}");
    }
}
