mod panels;
pub mod theme;

use std::path::PathBuf;
use std::time::{Duration, Instant};

use eframe::egui::{self, CornerRadius, Frame, Margin, RichText, Stroke, Ui};

use crate::bot::BotRuntime;
use crate::config::AppConfig;
use crate::memory::{MemoryReader, MemorySnapshot};

pub use panels::debug::DiagnosticsVerbosity;

// ── Cycle statistics ──────────────────────────────────────────────────────────
pub struct CycleStats {
    pub interval_ms: u64,
    pub cycles_last_minute: u64,
    pub total_cycles: u64,
    pub last_cycle_ms: u64,
}

// ── Tab enum ──────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Home,
    Profiles,
    Settings,
    Debug,
}

impl ActiveTab {
    const ALL: &'static [ActiveTab] = &[
        ActiveTab::Home,
        ActiveTab::Profiles,
        ActiveTab::Settings,
        ActiveTab::Debug,
    ];

    fn label(self) -> &'static str {
        match self {
            ActiveTab::Home => "Home",
            ActiveTab::Profiles => "Profiles",
            ActiveTab::Settings => "Settings",
            ActiveTab::Debug => "Debug",
        }
    }
}

// ── App ───────────────────────────────────────────────────────────────────────
pub struct GliderApp {
    config: AppConfig,
    bot: BotRuntime,
    memory_reader: MemoryReader,
    snapshot: Option<MemorySnapshot>,
    active_tab: ActiveTab,
    pid_input: String,
    profile_path_input: String,
    last_profile_dir: Option<PathBuf>,
    status_message: String,
    last_cycle_at: Option<Instant>,
    cycle_window_started: Instant,
    cycles_last_minute: u64,
    total_cycles: u64,
    last_cycle_ms: u64,
    diagnostics_verbosity: DiagnosticsVerbosity,
}

impl Default for GliderApp {
    fn default() -> Self {
        Self {
            config: AppConfig::default(),
            bot: BotRuntime::default(),
            memory_reader: MemoryReader::default(),
            snapshot: None,
            active_tab: ActiveTab::Home,
            pid_input: String::new(),
            profile_path_input: String::new(),
            last_profile_dir: None,
            status_message: String::new(),
            last_cycle_at: None,
            cycle_window_started: Instant::now(),
            cycles_last_minute: 0,
            total_cycles: 0,
            last_cycle_ms: 0,
            diagnostics_verbosity: DiagnosticsVerbosity::ErrorsAndWarnings,
        }
    }
}

impl GliderApp {
    pub fn from_creation_context(cc: &eframe::CreationContext<'_>) -> Self {
        theme::apply(&cc.egui_ctx);
        let mut app = Self::default();
        if let Some(rs) = cc.wgpu_render_state.as_ref() {
            let info = rs.adapter.get_info();
            app.status_message =
                format!("Renderer: wgpu {:?} ({})", info.backend, info.name);
        }
        app
    }
}

impl eframe::App for GliderApp {
    fn clear_color(&self, _vis: &egui::Visuals) -> [f32; 4] {
        // #07050F — matches BG_BASE
        [0.027, 0.020, 0.059, 1.0]
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.run_scheduled_cycle();

        let cycle_stats = CycleStats {
            interval_ms: self.config.memory_poll_ms,
            cycles_last_minute: self.cycles_last_minute,
            total_cycles: self.total_cycles,
            last_cycle_ms: self.last_cycle_ms,
        };

        let ctx = ui.ctx().clone();
        let full_rect = ui.max_rect();

        // ── 1. Paint ambient background first ─────────────────────────────────
        theme::paint_background(ui, full_rect);

        // ── 2. Title / Tab bar ────────────────────────────────────────────────
        egui::Panel::top("title_tab_bar")
            .resizable(false)
            .frame(
                Frame::new()
                    .fill(theme::TAB_BAR_FILL)
                    .inner_margin(Margin::symmetric(14, 0))
                    .stroke(Stroke::new(1.0, theme::GLASS_BORDER)),
            )
            .show_inside(ui, |ui| {
                ui.set_height(52.0);
                self.draw_title_tabs(ui);
            });

        // ── 3. Tab content ────────────────────────────────────────────────────
        egui::CentralPanel::default()
            .frame(Frame::new().fill(egui::Color32::TRANSPARENT).inner_margin(Margin::same(14)))
            .show_inside(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink(false)
                    .show(ui, |ui| match self.active_tab {
                        ActiveTab::Home => panels::home::draw(
                            ui,
                            self.snapshot.as_ref(),
                            &mut self.bot,
                            &cycle_stats,
                            &mut self.status_message,
                        ),
                        ActiveTab::Profiles => panels::profiles::draw(
                            ui,
                            &mut self.bot,
                            &mut self.profile_path_input,
                            &mut self.last_profile_dir,
                            &mut self.status_message,
                        ),
                        ActiveTab::Settings => panels::settings::draw(
                            ui,
                            &mut self.config,
                            &mut self.memory_reader,
                            &mut self.pid_input,
                            &mut self.status_message,
                        ),
                        ActiveTab::Debug => panels::debug::draw(
                            ui,
                            self.snapshot.as_ref(),
                            &self.bot,
                            &self.status_message,
                            &mut self.diagnostics_verbosity,
                        ),
                    });
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
    fn draw_title_tabs(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_centered(|ui| {
            // ── App branding ─────────────────────────────────────────
            ui.label(
                RichText::new("GliderRust")
                    .color(theme::TEXT_PRIMARY)
                    .size(16.0)
                    .strong(),
            );

            // Attachment dot
            let (dot, col) = if self.memory_reader.is_attached() {
                ("●", theme::ACCENT_GREEN)
            } else {
                ("○", theme::TEXT_DIM)
            };
            ui.label(RichText::new(dot).color(col).size(10.0));

            ui.add_space(20.0);

            // ── Tab buttons ───────────────────────────────────────────
            for &tab in ActiveTab::ALL {
                let active = self.active_tab == tab;
                let (fill, stroke_col, text_col) = if active {
                    (theme::TAB_ACTIVE_FILL, theme::TAB_ACTIVE_BORDER, theme::ACCENT_BLUE)
                } else {
                    (theme::TAB_INACTIVE_FILL, theme::TAB_INACTIVE_BORDER, theme::TEXT_SECONDARY)
                };

                let btn = egui::Button::new(
                    RichText::new(tab.label()).color(text_col).size(13.0),
                )
                .fill(fill)
                .stroke(Stroke::new(1.0, stroke_col))
                .min_size(egui::vec2(90.0, 34.0))
                .corner_radius(CornerRadius::same(17));

                if ui.add(btn).clicked() {
                    self.active_tab = tab;
                }
                ui.add_space(2.0);
            }

            // ── Right side: hands-off badge + bot state ───────────────
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let (label, col) = match self.bot.state() {
                    crate::bot::BotState::Running => ("● Running", theme::ACCENT_GREEN),
                    crate::bot::BotState::Paused => ("◐ Paused", theme::ACCENT_YELLOW),
                    crate::bot::BotState::Stopped => ("○ Stopped", theme::TEXT_DIM),
                };
                ui.label(RichText::new(label).color(col).size(12.0));

            });
        });
    }

    fn run_scheduled_cycle(&mut self) {
        if self.bot.state() != crate::bot::BotState::Running {
            return;
        }

        let now = Instant::now();
        let interval = Duration::from_millis(self.config.memory_poll_ms);
        let due = self
            .last_cycle_at
            .map(|t| now.duration_since(t) >= interval)
            .unwrap_or(true);

        if !due {
            return;
        }

        if now.duration_since(self.cycle_window_started) >= Duration::from_secs(60) {
            self.cycle_window_started = now;
            self.cycles_last_minute = 0;
        }

        let started = Instant::now();

        match self.memory_reader.read_snapshot() {
            Ok(snap) => self.snapshot = Some(snap),
            Err(err) => {
                self.snapshot = None;
                self.status_message = format!("Snapshot read failed: {err}");
            }
        }

        self.bot.tick(self.snapshot.as_ref());
        self.last_cycle_at = Some(now);
        self.cycles_last_minute += 1;
        self.total_cycles += 1;
        self.last_cycle_ms = started.elapsed().as_millis() as u64;
    }
}
