use eframe::egui::{self, RichText};

use crate::config::AppConfig;
use crate::memory::MemoryReader;
use crate::ui::theme;

pub fn draw(
    ui: &mut egui::Ui,
    config: &mut AppConfig,
    reader: &mut MemoryReader,
    pid_input: &mut String,
    status_message: &mut String,
) {
    // ── Process attach ────────────────────────────────────────────────────────
    theme::glass_frame_raised().show(ui, |ui| {
        ui.label(
            RichText::new("WOW PROCESS")
                .color(theme::TEXT_DIM)
                .size(10.0),
        );
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            let attached = reader.is_attached();
            let dot = if attached { "●" } else { "○" };
            let col = if attached {
                theme::ACCENT_GREEN
            } else {
                theme::TEXT_DIM
            };
            ui.label(RichText::new(dot).color(col));
            ui.label(RichText::new(if attached { "Attached" } else { "Not attached" }).color(col));
        });

        ui.add_space(6.0);

        // PID input row
        ui.horizontal(|ui| {
            ui.label(RichText::new("PID").color(theme::TEXT_SECONDARY));
            ui.add(
                egui::TextEdit::singleline(pid_input)
                    .hint_text("Process ID")
                    .desired_width(80.0),
            );

            if ui.button("Attach").clicked() {
                match pid_input.trim().parse::<u32>() {
                    Ok(pid) => match reader.attach(pid) {
                        Ok(()) => *status_message = format!("Attached to PID {pid}"),
                        Err(err) => *status_message = format!("Attach failed: {err}"),
                    },
                    Err(_) => *status_message = "PID must be a valid number".to_string(),
                }
            }
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            if ui
                .add(
                    egui::Button::new(RichText::new("Auto-Attach WoW").color(theme::ACCENT_BLUE))
                        .fill(theme::ACCENT_BLUE_DIM)
                        .stroke(egui::Stroke::new(1.0, theme::ACCENT_BLUE)),
                )
                .clicked()
            {
                match reader.attach_wow() {
                    Ok(pid) => {
                        *pid_input = pid.to_string();
                        *status_message = format!("Auto-attached to WoW PID {pid}");
                    }
                    Err(err) => {
                        *status_message = format!("Auto attach failed: {err}");
                    }
                }
            }

            if ui.button("Detach").clicked() {
                reader.detach();
                *status_message = "Detached from process".to_string();
            }
        });
    });

    ui.add_space(8.0);

    // ── Performance ───────────────────────────────────────────────────────────
    theme::glass_frame().show(ui, |ui| {
        ui.label(
            RichText::new("PERFORMANCE")
                .color(theme::TEXT_DIM)
                .size(10.0),
        );
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            ui.label(RichText::new("Memory poll interval").color(theme::TEXT_SECONDARY));
            let mut ms = config.memory_poll_ms as f32;
            if ui
                .add(
                    egui::Slider::new(&mut ms, 100.0..=5000.0)
                        .suffix(" ms")
                        .clamping(egui::SliderClamping::Always),
                )
                .changed()
            {
                config.memory_poll_ms = ms as u64;
            }
        });
    });

    ui.add_space(8.0);

    // ── Telemetry ─────────────────────────────────────────────────────────────
    theme::glass_frame().show(ui, |ui| {
        ui.label(RichText::new("TELEMETRY").color(theme::TEXT_DIM).size(10.0));
        ui.add_space(6.0);
        ui.checkbox(
            &mut config.telemetry_enabled,
            RichText::new("Enable telemetry").color(theme::TEXT_PRIMARY),
        );
    });

    if !status_message.is_empty() {
        ui.add_space(8.0);
        theme::glass_frame_inset().show(ui, |ui| {
            ui.label(
                RichText::new(status_message.as_str())
                    .color(theme::TEXT_SECONDARY)
                    .size(12.0),
            );
        });
    }
}
