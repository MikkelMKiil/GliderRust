mod panels;
pub mod theme;

use std::path::PathBuf;
use std::time::Duration;

use eframe::egui::{self, CornerRadius, Frame, Margin, RichText, Stroke, Ui};

use crate::service::AppService;

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
    service: AppService,
    active_tab: ActiveTab,
    pid_input: String,
    profile_path_input: String,
    last_profile_dir: Option<PathBuf>,
    diagnostics_verbosity: DiagnosticsVerbosity,
}

impl Default for GliderApp {
    fn default() -> Self {
        Self {
            service: AppService::default(),
            active_tab: ActiveTab::Home,
            pid_input: String::new(),
            profile_path_input: String::new(),
            last_profile_dir: None,
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
            app.service.status_message =
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

        let service_cycle = self.service.cycle_stats();

        let cycle_stats = CycleStats {
            interval_ms: service_cycle.interval_ms,
            cycles_last_minute: service_cycle.cycles_last_minute,
            total_cycles: service_cycle.total_cycles,
            last_cycle_ms: service_cycle.last_cycle_ms,
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
                            self.service.snapshot.as_ref(),
                            &mut self.service.bot,
                            &cycle_stats,
                            &mut self.service.status_message,
                        ),
                        ActiveTab::Profiles => panels::profiles::draw(
                            ui,
                            &mut self.service.bot,
                            &mut self.profile_path_input,
                            &mut self.last_profile_dir,
                            &mut self.service.status_message,
                        ),
                        ActiveTab::Settings => panels::settings::draw(
                            ui,
                            &mut self.service.config,
                            &mut self.service.memory_reader,
                            &mut self.pid_input,
                            &mut self.service.status_message,
                        ),
                        ActiveTab::Debug => panels::debug::draw(
                            ui,
                            self.service.snapshot.as_ref(),
                            &self.service.bot,
                            &self.service.status_message,
                            &mut self.diagnostics_verbosity,
                        ),
                    });
            });

        let repaint_ms = if self.service.bot.state() == crate::bot::BotState::Running {
            self.service.config.memory_poll_ms.max(16)
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
            let (dot, col) = if self.service.memory_reader.is_attached() {
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
                let (label, col) = match self.service.bot.state() {
                    crate::bot::BotState::Running => ("● Running", theme::ACCENT_GREEN),
                    crate::bot::BotState::Paused => ("◐ Paused", theme::ACCENT_YELLOW),
                    crate::bot::BotState::Stopped => ("○ Stopped", theme::TEXT_DIM),
                };
                ui.label(RichText::new(label).color(col).size(12.0));

            });
        });
    }

    fn run_scheduled_cycle(&mut self) {
        self.service.run_scheduled_cycle();
    }
}
