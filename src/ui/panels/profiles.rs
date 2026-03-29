use std::path::PathBuf;

use eframe::egui::{self, RichText};

use crate::bot::BotRuntime;
use crate::profile::load_profile;
use crate::ui::theme;

pub fn draw(
    ui: &mut egui::Ui,
    bot: &mut BotRuntime,
    profile_path_input: &mut String,
    last_profile_dir: &mut Option<PathBuf>,
    status_message: &mut String,
) {
    // ── Bot controls ──────────────────────────────────────────────────────────
    theme::glass_frame_raised().show(ui, |ui| {
        ui.label(RichText::new("BOT CONTROLS").color(theme::TEXT_DIM).size(10.0));
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            if ui
                .add(
                    egui::Button::new(
                        RichText::new("  Start  ").color(theme::ACCENT_GREEN),
                    )
                    .fill(egui::Color32::from_rgb(14, 38, 26))
                    .stroke(egui::Stroke::new(1.0, theme::ACCENT_GREEN)),
                )
                .clicked()
            {
                bot.start();
                *status_message = "Bot started".to_string();
            }

            if ui
                .add(
                    egui::Button::new(
                        RichText::new("  Pause  ").color(theme::ACCENT_YELLOW),
                    )
                    .fill(egui::Color32::from_rgb(38, 30, 10))
                    .stroke(egui::Stroke::new(1.0, theme::ACCENT_YELLOW)),
                )
                .clicked()
            {
                bot.pause();
                *status_message = "Bot paused".to_string();
            }

            if ui
                .add(
                    egui::Button::new(
                        RichText::new("  Stop  ").color(theme::ACCENT_RED),
                    )
                    .fill(egui::Color32::from_rgb(38, 12, 12))
                    .stroke(egui::Stroke::new(1.0, theme::ACCENT_RED)),
                )
                .clicked()
            {
                bot.stop();
                *status_message = "Bot stopped".to_string();
            }
        });

        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.label(RichText::new("State:").color(theme::TEXT_SECONDARY));
            let (dot, label, col) = match bot.state() {
                crate::bot::BotState::Running => ("●", "Running", theme::ACCENT_GREEN),
                crate::bot::BotState::Paused => ("◐", "Paused", theme::ACCENT_YELLOW),
                crate::bot::BotState::Stopped => ("○", "Stopped", theme::TEXT_DIM),
            };
            ui.label(RichText::new(format!("{dot} {label}")).color(col));
        });

        if let Some((cur, total)) = bot.waypoint_progress() {
            ui.label(
                RichText::new(format!("Waypoint progress: {cur} / {total}"))
                    .color(theme::TEXT_SECONDARY)
                    .size(12.0),
            );
        }
    });

    ui.add_space(8.0);

    // ── Active profile ────────────────────────────────────────────────────────
    theme::glass_frame().show(ui, |ui| {
        ui.label(RichText::new("ACTIVE PROFILE").color(theme::TEXT_DIM).size(10.0));
        ui.add_space(4.0);

        let name = bot.active_profile_name().unwrap_or("None loaded");
        ui.label(RichText::new(name).color(theme::TEXT_PRIMARY).size(16.0).strong());

        ui.add_space(8.0);

        // Profile path input
        ui.horizontal(|ui| {
            ui.label(RichText::new("Path").color(theme::TEXT_SECONDARY));
            ui.add(
                egui::TextEdit::singleline(profile_path_input)
                    .hint_text("path/to/profile.xml")
                    .desired_width(f32::INFINITY),
            );
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            if ui.button("Browse…").clicked() {
                let mut dialog =
                    rfd::FileDialog::new().add_filter("Profile XML", &["xml"]);
                if let Some(dir) = last_profile_dir.as_ref() {
                    dialog = dialog.set_directory(dir);
                }
                if let Some(file) = dialog.pick_file() {
                    *profile_path_input = file.display().to_string();
                    *last_profile_dir = file.parent().map(std::path::Path::to_path_buf);
                }
            }

            if ui.button("Load Profile").clicked() {
                let path = profile_path_input.trim();
                if path.is_empty() {
                    *status_message = "Profile path cannot be empty".to_string();
                } else {
                    match load_profile(path) {
                        Ok(profile) => {
                            let wp_count = profile.waypoints.len();
                            bot.set_profile(profile);
                            *status_message =
                                format!("Loaded profile with {wp_count} waypoints");
                        }
                        Err(err) => {
                            *status_message = format!("Profile load failed: {err}");
                        }
                    }
                }
            }

            if ui.button("Clear").clicked() {
                bot.clear_profile();
                *status_message = "Profile cleared".to_string();
            }
        });
    });

    if !status_message.is_empty() {
        ui.add_space(8.0);
        theme::glass_frame_inset().show(ui, |ui| {
            ui.label(RichText::new(status_message.as_str()).color(theme::TEXT_SECONDARY).size(12.0));
        });
    }
}
