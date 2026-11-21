// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Phase 2 spike: a native interface sketch with a dock layout, a GPU-drawn
//! waveform, animated meters, a transport bar and text fields. It measures
//! start-up and frame times so the toolkit choice rests on numbers from a real
//! machine. It is not shipped and plays no sound.

use std::time::Instant;

use eframe::egui;
use egui_dock::{DockArea, DockState, NodeIndex, Style, TabViewer};

mod stats;
mod wave;

use stats::FrameStats;
use wave::SyntheticWave;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Files,
    Editor,
    Levels,
    Settings,
}

impl Tab {
    fn title(self) -> &'static str {
        match self {
            Tab::Files => "Files",
            Tab::Editor => "Editor",
            Tab::Levels => "Levels",
            Tab::Settings => "Settings",
        }
    }
}

struct Panels {
    wave: SyntheticWave,
    search: String,
    notes: String,
    gain_db: f32,
    time: f64,
    playing: bool,
}

struct Viewer<'a> {
    p: &'a mut Panels,
}

impl TabViewer for Viewer<'_> {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Tab) -> egui::WidgetText {
        tab.title().into()
    }

    fn id(&mut self, tab: &mut Tab) -> egui::Id {
        egui::Id::NULL.with(tab.title())
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Tab) {
        match *tab {
            Tab::Files => {
                ui.label("Search");
                ui.text_edit_singleline(&mut self.p.search);
                ui.separator();
                for i in 0..40 {
                    ui.label(format!("Episode {i:03} host mic.wav"));
                }
            }
            Tab::Editor => self.p.wave.show(ui, self.p.time),
            Tab::Levels => {
                let t = self.p.time;
                wave::meters(ui, t, self.p.playing);
            }
            Tab::Settings => {
                ui.label("Notes (type here, try an input method, paste, select):");
                ui.text_edit_multiline(&mut self.p.notes);
                ui.add(egui::Slider::new(&mut self.p.gain_db, -60.0..=12.0).text("Gain dB"));
                ui.add(egui::DragValue::new(&mut self.p.gain_db).suffix(" dB"));
            }
        }
    }

    fn closeable(&mut self, _tab: &mut Tab) -> bool {
        false
    }
}

struct App {
    dock: DockState<Tab>,
    panels: Panels,
    stats: FrameStats,
    started: Instant,
    first_frame_logged: bool,
}

impl App {
    fn new() -> Self {
        let mut dock = DockState::new(vec![Tab::Editor, Tab::Settings]);
        let surface = dock.main_surface_mut();
        let [main, _files] = surface.split_left(NodeIndex::root(), 0.2, vec![Tab::Files]);
        surface.split_below(main, 0.7, vec![Tab::Levels]);
        Self {
            dock,
            panels: Panels {
                wave: SyntheticWave::new(),
                search: String::new(),
                notes: String::new(),
                gain_db: 0.0,
                time: 0.0,
                playing: true,
            },
            stats: FrameStats::default(),
            started: Instant::now(),
            first_frame_logged: false,
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.stats.tick();
        if !self.first_frame_logged {
            self.first_frame_logged = true;
            stats::log(&format!(
                "first frame {:.0} ms after the app object was made",
                self.started.elapsed().as_secs_f64() * 1000.0
            ));
        }
        if self.panels.playing {
            self.panels.time += f64::from(ctx.input(|i| i.stable_dt));
        }

        egui::Panel::top("transport").show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.button(if self.panels.playing { "Pause" } else { "Play" }).clicked() {
                    self.panels.playing = !self.panels.playing;
                }
                if ui.button("Stop").clicked() {
                    self.panels.playing = false;
                    self.panels.time = 0.0;
                }
                let t = self.panels.time;
                ui.monospace(format!(
                    "{}:{:02}:{:02}.{:03}",
                    (t / 3600.0) as u32,
                    ((t / 60.0) as u32) % 60,
                    (t as u32) % 60,
                    ((t.fract()) * 1000.0) as u32
                ));
                ui.separator();
                ui.label(self.stats.summary());
            });
        });

        egui::CentralPanel::default().show(ui, |ui| {
            DockArea::new(&mut self.dock)
                .style(Style::from_egui(ui.style()))
                .show_inside(ui, &mut Viewer { p: &mut self.panels });
        });

        // The meters and playhead move, so ask for the next frame at once.
        ctx.request_repaint();
    }
}

fn main() -> eframe::Result<()> {
    let begin = Instant::now();
    stats::log("start");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 760.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Altered Audio Editor, native spike",
        options,
        Box::new(move |_cc| {
            stats::log(&format!(
                "window and renderer ready {:.0} ms after start",
                begin.elapsed().as_secs_f64() * 1000.0
            ));
            Ok(Box::new(App::new()))
        }),
    )
}
