// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! A synthetic three-hour waveform (as min/max peaks) and fake meters, to
//! measure drawing cost without needing any audio file.

use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Vec2};

/// Samples per peak block, and blocks: 256 x 2,000,000 is about three hours at 48 kHz.
const BLOCK: f64 = 256.0;
const BLOCKS: usize = 2_000_000;
const RATE: f64 = 48_000.0;

pub struct SyntheticWave {
    peaks: Vec<(f32, f32)>,
    /// Seconds at the left edge, and pixels per second.
    start: f64,
    zoom: f64,
}

impl SyntheticWave {
    pub fn new() -> Self {
        let mut peaks = Vec::with_capacity(BLOCKS);
        for i in 0..BLOCKS {
            let t = i as f64 * BLOCK / RATE;
            // Slow level changes and a speech-like flutter.
            let env = 0.35 + 0.35 * (t * 0.05).sin().abs() + 0.2 * (t * 1.7).sin().abs();
            let v = (env * (0.6 + 0.4 * (t * 37.0).sin().abs())) as f32;
            peaks.push((-v, v));
        }
        Self {
            peaks,
            start: 0.0,
            zoom: 40.0,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, playhead: f64) {
        let size = ui.available_size_before_wrap().max(Vec2::new(100.0, 120.0));
        let (resp, painter) = ui.allocate_painter(size, Sense::click_and_drag());
        let rect = resp.rect;
        painter.rect_filled(rect, 0.0, Color32::from_rgb(20, 24, 28));

        if resp.dragged() {
            self.start -= f64::from(resp.drag_delta().x) / self.zoom;
        }
        if resp.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll != 0.0 {
                let at = resp
                    .hover_pos()
                    .map_or(0.5, |p| f64::from((p.x - rect.left()) / rect.width()));
                let before = self.start + at * f64::from(rect.width()) / self.zoom;
                self.zoom = (self.zoom * (1.0 + f64::from(scroll) * 0.004)).clamp(0.05, 20_000.0);
                self.start = before - at * f64::from(rect.width()) / self.zoom;
            }
        }
        self.start = self.start.max(0.0);

        let mid = rect.center().y;
        let half = rect.height() * 0.45;
        let seconds_per_px = 1.0 / self.zoom;
        let color = Stroke::new(1.0, Color32::from_rgb(90, 190, 170));
        let mut shapes = Vec::with_capacity(rect.width() as usize);
        for x in 0..rect.width() as usize {
            let t0 = self.start + x as f64 * seconds_per_px;
            let t1 = t0 + seconds_per_px;
            let b0 = ((t0 * RATE / BLOCK) as usize).min(BLOCKS - 1);
            let b1 = ((t1 * RATE / BLOCK).ceil() as usize).clamp(b0 + 1, BLOCKS);
            let (mut lo, mut hi) = (0.0f32, 0.0f32);
            for p in &self.peaks[b0..b1] {
                lo = lo.min(p.0);
                hi = hi.max(p.1);
            }
            let px = rect.left() + x as f32 + 0.5;
            shapes.push(egui::Shape::line_segment(
                [Pos2::new(px, mid - hi * half), Pos2::new(px, mid - lo * half)],
                color,
            ));
        }
        painter.extend(shapes);

        let hx = rect.left() + ((playhead - self.start) * self.zoom) as f32;
        if hx >= rect.left() && hx <= rect.right() {
            painter.line_segment(
                [Pos2::new(hx, rect.top()), Pos2::new(hx, rect.bottom())],
                Stroke::new(1.5, Color32::from_rgb(240, 200, 60)),
            );
        }
        painter.text(
            rect.left_top() + Vec2::new(6.0, 4.0),
            egui::Align2::LEFT_TOP,
            "drag to scroll, wheel to zoom",
            egui::FontId::proportional(12.0),
            Color32::GRAY,
        );
    }
}

/// Sixteen stereo-looking meters with a fake moving level.
pub fn meters(ui: &mut egui::Ui, t: f64, moving: bool) {
    let size = ui.available_size_before_wrap().max(Vec2::new(100.0, 80.0));
    let (resp, painter) = ui.allocate_painter(size, Sense::hover());
    let rect = resp.rect;
    painter.rect_filled(rect, 0.0, Color32::from_rgb(20, 24, 28));
    let n = 32usize;
    let w = rect.width() / n as f32;
    for i in 0..n {
        let phase = i as f64 * 0.37;
        let level = if moving {
            (0.55 + 0.4 * (t * (1.3 + i as f64 * 0.11) + phase).sin()).clamp(0.0, 1.0) as f32
        } else {
            0.0
        };
        let h = rect.height() * level;
        let bar = Rect::from_min_max(
            Pos2::new(rect.left() + i as f32 * w + 2.0, rect.bottom() - h),
            Pos2::new(rect.left() + (i + 1) as f32 * w - 2.0, rect.bottom()),
        );
        let c = if level > 0.9 {
            Color32::from_rgb(230, 70, 60)
        } else if level > 0.7 {
            Color32::from_rgb(230, 190, 60)
        } else {
            Color32::from_rgb(80, 200, 120)
        };
        painter.rect_filled(bar, 1.0, c);
    }
}
