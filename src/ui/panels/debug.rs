use std::collections::BTreeMap;

use eframe::egui::{self, RichText};

use crate::bot::BotRuntime;
use crate::memory::MemorySnapshot;
use crate::ui::theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticsVerbosity {
    ErrorsAndWarnings,
    All,
}

impl DiagnosticsVerbosity {
    pub fn label(self) -> &'static str {
        match self {
            Self::ErrorsAndWarnings => "Errors / Warnings",
            Self::All => "All",
        }
    }

    fn includes(self, line: &str) -> bool {
        match self {
            Self::ErrorsAndWarnings => !line.starts_with("trace "),
            Self::All => true,
        }
    }
}

pub fn draw(
    ui: &mut egui::Ui,
    snapshot: Option<&MemorySnapshot>,
    bot: &BotRuntime,
    status_message: &str,
    verbosity: &mut DiagnosticsVerbosity,
) {
    // ── Nav / Bot internals ───────────────────────────────────────────────────
    theme::glass_frame_raised().show(ui, |ui| {
        ui.label(RichText::new("BOT INTERNALS").color(theme::TEXT_DIM).size(10.0));
        ui.add_space(4.0);

        ui.label(
            RichText::new(format!("Bot state: {:?}", bot.state()))
                .color(theme::TEXT_SECONDARY)
                .size(12.0),
        );
        ui.label(
            RichText::new(format!("Runtime status: {:?}", bot.status()))
                .color(theme::TEXT_SECONDARY)
                .size(12.0),
        );

        let nav = bot.nav_output();
        ui.label(
            RichText::new(format!(
                "Nav  desired={:.3?}  error={:.3?}  dist={:.2?}  reached={}",
                nav.desired_heading_rad,
                nav.heading_error_rad,
                nav.distance_to_waypoint,
                nav.waypoint_reached
            ))
            .color(theme::TEXT_SECONDARY)
            .size(12.0),
        );

        ui.label(
            RichText::new(format!("Inputs: {:?}", bot.suggested_inputs()))
                .color(theme::TEXT_SECONDARY)
                .size(12.0),
        );
    });

    ui.add_space(8.0);

    // ── Snapshot summary ──────────────────────────────────────────────────────
    if let Some(s) = snapshot {
        theme::glass_frame().show(ui, |ui| {
            ui.label(RichText::new("SNAPSHOT").color(theme::TEXT_DIM).size(10.0));
            ui.add_space(4.0);

            ui.label(
                RichText::new(format!(
                    "Player: {} (GUID 0x{:016X})",
                    s.player_name,
                    s.player_guid.unwrap_or(0)
                ))
                .color(theme::TEXT_SECONDARY)
                .size(12.0),
            );
            ui.label(
                RichText::new(format!(
                    "Target: {}",
                    s.target_name.as_deref().unwrap_or("none")
                ))
                .color(theme::TEXT_SECONDARY)
                .size(12.0),
            );
            ui.label(
                RichText::new(format!("Target GUID: {:?}", s.target_guid))
                    .color(theme::TEXT_SECONDARY)
                    .size(12.0),
            );
            ui.label(
                RichText::new(format!(
                    "Player HP: {:?} / {:?}",
                    s.player_current_health, s.player_max_health
                ))
                .color(theme::TEXT_SECONDARY)
                .size(12.0),
            );
            ui.label(
                RichText::new(format!("Position: {:?}", s.position))
                    .color(theme::TEXT_SECONDARY)
                    .size(12.0),
            );
        });

        ui.add_space(8.0);
    }

    // ── Diagnostics log ───────────────────────────────────────────────────────
    theme::glass_frame().show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new("DIAGNOSTICS").color(theme::TEXT_DIM).size(10.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                egui::ComboBox::from_id_salt("diag_verbosity")
                    .selected_text(
                        RichText::new(verbosity.label())
                            .color(theme::TEXT_SECONDARY)
                            .size(11.0),
                    )
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            verbosity,
                            DiagnosticsVerbosity::ErrorsAndWarnings,
                            DiagnosticsVerbosity::ErrorsAndWarnings.label(),
                        );
                        ui.selectable_value(
                            verbosity,
                            DiagnosticsVerbosity::All,
                            DiagnosticsVerbosity::All.label(),
                        );
                    });
            });
        });

        ui.add_space(4.0);

        let mut lines: Vec<String> = Vec::new();

        if !status_message.is_empty() {
            lines.push(format!("status {status_message}"));
        }

        if let Some(s) = snapshot {
            for line in &s.diagnostics {
                if verbosity.includes(line) {
                    lines.push(line.clone());
                }
            }
        }

        if lines.is_empty() {
            ui.label(RichText::new("No diagnostics available").color(theme::TEXT_DIM).size(12.0));
            return;
        }

        let mut grouped: BTreeMap<&'static str, Vec<String>> = BTreeMap::new();
        for line in lines {
            grouped.entry(category_for(&line)).or_default().push(line);
        }

        ui.add_space(4.0);
        egui::ScrollArea::vertical()
            .max_height(300.0)
            .auto_shrink(false)
            .show(ui, |ui| {
                for (category, items) in grouped {
                    egui::CollapsingHeader::new(
                        RichText::new(format!("{category}  ({})", items.len()))
                            .color(theme::TEXT_SECONDARY)
                            .size(12.0),
                    )
                    .default_open(matches!(
                        category,
                        "Traversal" | "Health" | "Model Chain"
                    ))
                    .show(ui, |ui| {
                        for line in items {
                            ui.label(
                                RichText::new(&line)
                                    .color(theme::TEXT_DIM)
                                    .size(11.0)
                                    .monospace(),
                            );
                        }
                    });
                }
            });
    });
}

fn category_for(line: &str) -> &'static str {
    let lower = line.to_ascii_lowercase();
    if lower.contains("display") || lower.contains("model") || lower.contains("monster_def") {
        return "Model Chain";
    }
    if lower.contains("traversal")
        || lower.contains("first object")
        || lower.contains("guid")
    {
        return "Traversal";
    }
    if lower.contains("health") || lower.contains("hp") {
        return "Health";
    }
    if lower.contains("target") {
        return "Target";
    }
    if lower.contains("attach")
        || lower.contains("pid")
        || lower.contains("process")
    {
        return "Process";
    }
    "General"
}
