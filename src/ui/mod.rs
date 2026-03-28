mod panels;

use std::path::PathBuf;
use std::time::{Duration, Instant};

use eframe::egui::{self, Align2, RichText, Ui};

use crate::bot::BotRuntime;
use crate::config::AppConfig;
use crate::memory::{MemoryReader, MemorySnapshot};

pub struct CycleStats {
    pub interval_ms: u64,
    pub cycles_last_minute: u64,
    pub total_cycles: u64,
    pub last_cycle_ms: u64,
}

pub struct GliderApp {
    config: AppConfig,
    bot: BotRuntime,
    memory_reader: MemoryReader,
    snapshot: Option<MemorySnapshot>,
    pid_input: String,
    profile_path_input: String,
    last_profile_dir: Option<PathBuf>,
    status_message: String,
    last_cycle_at: Option<Instant>,
    cycle_window_started: Instant,
    cycles_last_minute: u64,
    total_cycles: u64,
    last_cycle_ms: u64,
    diagnostics_open: bool,
    diagnostics_verbosity: panels::diagnostics::DiagnosticsVerbosity,
}

impl Default for GliderApp {
    fn default() -> Self {
        Self {
            config: AppConfig::default(),
            bot: BotRuntime::default(),
            memory_reader: MemoryReader::default(),
            snapshot: None,
            pid_input: String::new(),
            profile_path_input: String::new(),
            last_profile_dir: None,
            status_message: String::new(),
            last_cycle_at: None,
            cycle_window_started: Instant::now(),
            cycles_last_minute: 0,
            total_cycles: 0,
            last_cycle_ms: 0,
            diagnostics_open: false,
            diagnostics_verbosity: panels::diagnostics::DiagnosticsVerbosity::ErrorsAndWarnings,
        }
    }
}

impl GliderApp {
    pub fn from_creation_context(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self::default();

        if let Some(render_state) = cc.wgpu_render_state.as_ref() {
            let info = render_state.adapter.get_info();
            app.status_message = format!(
                "Renderer online: wgpu {:?} ({})",
                info.backend, info.name
            );
        } else {
            app.status_message = "Renderer online: wgpu render state unavailable".to_string();
        }

        app
    }
}

impl eframe::App for GliderApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.run_scheduled_cycle();

        let cycle_stats = CycleStats {
            interval_ms: self.config.memory_poll_ms,
            cycles_last_minute: self.cycles_last_minute,
            total_cycles: self.total_cycles,
            last_cycle_ms: self.last_cycle_ms,
        };

        let ctx = ui.ctx().clone();

        self.draw_command_bar(ui);

        egui::Panel::right("diagnostics_drawer")
            .default_size(360.0)
            .resizable(true)
            .show_animated_inside(ui, self.diagnostics_open, |ui| {
                panels::diagnostics::draw(
                    ui,
                    self.snapshot.as_ref(),
                    &self.status_message,
                    &mut self.diagnostics_verbosity,
                );
            });

        self.draw_diagnostics_tab(&ctx);

        egui::CentralPanel::default().show_inside(ui, |ui| {
            panels::status::draw(
                ui,
                &mut self.bot,
                &mut self.memory_reader,
                &mut self.snapshot,
                &mut self.pid_input,
                &mut self.profile_path_input,
                &mut self.last_profile_dir,
                &cycle_stats,
                &mut self.status_message,
            );
        });

        let repaint_ms = if self.bot.state() == crate::bot::BotState::Running {
            self.config.memory_poll_ms.max(16)
        } else {
            250
        };
        ctx.request_repaint_after(Duration::from_millis(repaint_ms));
    }
}

impl GliderApp {
    fn draw_command_bar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("command_bar")
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new("GliderRust Tactical HUD").strong());
                    ui.separator();
                    ui.label(format!("State: {:?}", self.bot.state()));
                    ui.label(format!("Attached: {}", self.memory_reader.is_attached()));

                    if ui.button("Start").clicked() {
                        self.bot.start();
                        self.status_message = "Bot started".to_string();
                    }

                    if ui.button("Pause").clicked() {
                        self.bot.pause();
                        self.status_message = "Bot paused".to_string();
                    }

                    if ui.button("Stop").clicked() {
                        self.bot.stop();
                        self.status_message = "Bot stopped".to_string();
                    }

                    ui.separator();
                    let label = if self.diagnostics_open {
                        "Hide Diagnostics"
                    } else {
                        "Show Diagnostics"
                    };
                    if ui.button(label).clicked() {
                        self.diagnostics_open = !self.diagnostics_open;
                    }
                });

                if !self.status_message.is_empty() {
                    ui.label(RichText::new(self.status_message.as_str()).small());
                }
                ui.add_space(2.0);
            });
    }

    fn draw_diagnostics_tab(&mut self, ctx: &egui::Context) {
        egui::Area::new("diagnostics_edge_tab".into())
            .anchor(Align2::RIGHT_CENTER, egui::vec2(0.0, 0.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                let label = if self.diagnostics_open { ">" } else { "<" };
                let hover_text = if self.diagnostics_open {
                    "Hide diagnostics"
                } else {
                    "Show diagnostics"
                };

                let button = egui::Button::new(RichText::new(label).size(18.0))
                    .min_size(egui::vec2(24.0, 96.0));

                if ui.add(button).on_hover_text(hover_text).clicked() {
                    self.diagnostics_open = !self.diagnostics_open;
                }
            });
    }

    fn run_scheduled_cycle(&mut self) {
        if self.bot.state() != crate::bot::BotState::Running {
            return;
        }

        let now = Instant::now();
        let interval = Duration::from_millis(self.config.memory_poll_ms);
        let is_due = self
            .last_cycle_at
            .map(|last| now.duration_since(last) >= interval)
            .unwrap_or(true);

        if !is_due {
            return;
        }

        if now.duration_since(self.cycle_window_started) >= Duration::from_secs(60) {
            self.cycle_window_started = now;
            self.cycles_last_minute = 0;
        }

        let cycle_started = Instant::now();

        match self.memory_reader.read_snapshot() {
            Ok(latest) => {
                self.snapshot = Some(latest);
            }
            Err(err) => {
                self.snapshot = None;
                self.status_message = format!("Snapshot read failed: {err}");
            }
        }

        self.bot.tick(self.snapshot.as_ref());

        self.last_cycle_at = Some(now);
        self.cycles_last_minute += 1;
        self.total_cycles += 1;
        self.last_cycle_ms = cycle_started.elapsed().as_millis() as u64;
    }
}
