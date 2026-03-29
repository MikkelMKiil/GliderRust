use eframe::egui::{self, Color32, CornerRadius, Frame, Margin, RichText, Stroke};

use crate::bot::{BotRuntime, BotState};
use crate::memory::MemorySnapshot;
use crate::ui::theme;
use crate::ui::CycleStats;

const HP_BAR_HEIGHT: f32 = 22.0;

pub fn draw(
    ui: &mut egui::Ui,
    snapshot: Option<&MemorySnapshot>,
    bot: &mut BotRuntime,
    cycle_stats: &CycleStats,
    status_message: &mut String,
) {
    // ── Player card ───────────────────────────────────────────────────────────
    draw_entity_card(
        ui,
        "PLAYER",
        snapshot.map(|s| s.player_name.as_str()),
        snapshot.and_then(|s| s.player_level),
        snapshot.and_then(|s| s.player_race).map(race_name),
        snapshot.and_then(|s| s.player_current_health),
        snapshot.and_then(|s| s.player_max_health),
        None,
    );

    ui.add_space(8.0);

    // ── Target card ───────────────────────────────────────────────────────────
    if let Some(s) = snapshot.filter(|s| s.target_guid.is_some()) {
        draw_entity_card(
            ui,
            "TARGET",
            Some(s.target_name.as_deref().unwrap_or("Unknown")),
            s.target_level,
            s.target_race.map(race_name),
            s.target_current_health,
            s.target_max_health,
            s.target_hostility.as_deref(),
        );
    } else {
        draw_empty_target_card(ui);
    }

    ui.add_space(8.0);

    // ── Bot status + controls card ────────────────────────────────────────────
    draw_status_controls_card(ui, bot, snapshot, cycle_stats, status_message);
}

// ── Entity cards ──────────────────────────────────────────────────────────────

fn draw_entity_card(
    ui: &mut egui::Ui,
    header: &str,
    name: Option<&str>,
    level: Option<u32>,
    race: Option<&str>,
    cur_hp: Option<u32>,
    max_hp: Option<u32>,
    hostility: Option<&str>,
) {
    let is_hostile = hostility == Some("Hostile");
    let border_col = if is_hostile { theme::ACCENT_RED } else { theme::GLASS_BORDER };

    let resp = Frame::new()
        .fill(theme::GLASS_FILL_RAISED)
        .inner_margin(Margin::same(16))
        .corner_radius(CornerRadius::same(20))
        .stroke(Stroke::new(1.0, border_col))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());

            // Header row
            ui.horizontal(|ui| {
                ui.label(RichText::new(header).color(theme::TEXT_DIM).size(10.0).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let dot_col = if name.is_some() && name != Some("Unknown") && name != Some("—") {
                        theme::ACCENT_GREEN
                    } else {
                        theme::TEXT_DIM
                    };
                    ui.label(RichText::new("●").color(dot_col).size(11.0));
                });
            });

            ui.add_space(6.0);

            // Entity name — large
            ui.label(
                RichText::new(name.unwrap_or("—"))
                    .color(theme::TEXT_PRIMARY)
                    .size(24.0)
                    .strong(),
            );

            // Meta line: level · race · hostility
            let meta: Vec<String> = [
                level.map(|l| format!("Lv. {l}")),
                race.map(str::to_string),
                hostility.map(|h| {
                    let pfx = match h {
                        "Hostile" => "⚔ ",
                        "Friendly" => "✓ ",
                        _ => "~ ",
                    };
                    format!("{pfx}{h}")
                }),
            ]
            .into_iter()
            .flatten()
            .collect();

            let meta_str = if meta.is_empty() { "—".to_string() } else { meta.join("  ·  ") };
            let meta_col = hostility.map(hostility_color).unwrap_or(theme::TEXT_SECONDARY);

            ui.label(RichText::new(meta_str).color(meta_col).size(12.0));
            ui.add_space(10.0);
            draw_hp_bar(ui, cur_hp, max_hp);
        });

    // Specular highlight
    let rect = resp.response.rect;
    let spec = egui::Rect::from_min_size(
        rect.min + egui::vec2(20.0, 1.5),
        egui::vec2((rect.width() - 40.0).max(0.0), 1.5),
    );
    ui.painter().rect_filled(spec, CornerRadius::same(1), theme::GLASS_SPECULAR);
}

fn draw_empty_target_card(ui: &mut egui::Ui) {
    let resp = Frame::new()
        .fill(theme::GLASS_FILL_RAISED)
        .inner_margin(Margin::same(16))
        .corner_radius(CornerRadius::same(20))
        .stroke(Stroke::new(1.0, theme::GLASS_BORDER))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.label(RichText::new("TARGET").color(theme::TEXT_DIM).size(10.0).strong());
            ui.add_space(12.0);
            ui.label(RichText::new("No target selected").color(theme::TEXT_DIM).size(14.0));
            ui.add_space(10.0);
            draw_hp_bar(ui, None, None);
        });

    let rect = resp.response.rect;
    let spec = egui::Rect::from_min_size(
        rect.min + egui::vec2(20.0, 1.5),
        egui::vec2((rect.width() - 40.0).max(0.0), 1.5),
    );
    ui.painter().rect_filled(spec, CornerRadius::same(1), theme::GLASS_SPECULAR);
}

// ── Bot status + controls card ────────────────────────────────────────────────

fn draw_status_controls_card(
    ui: &mut egui::Ui,
    bot: &mut BotRuntime,
    snapshot: Option<&MemorySnapshot>,
    cycle_stats: &CycleStats,
    status_message: &mut String,
) {
    let resp = Frame::new()
        .fill(theme::GLASS_FILL)
        .inner_margin(Margin::same(16))
        .corner_radius(CornerRadius::same(20))
        .stroke(Stroke::new(1.0, theme::GLASS_BORDER))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());

            // State row
            ui.horizontal(|ui| {
                let (dot, label, col) = bot_state_display(bot);
                ui.label(RichText::new(dot).color(col).size(14.0));
                ui.label(RichText::new(label).color(col).size(15.0).strong());

                if let Some(name) = bot.active_profile_name() {
                    ui.separator();
                    ui.label(RichText::new(name).color(theme::TEXT_SECONDARY).size(13.0));
                }

                if let Some((cur, total)) = bot.waypoint_progress() {
                    ui.separator();
                    ui.label(
                        RichText::new(format!("WP {cur}/{total}"))
                            .color(theme::TEXT_SECONDARY)
                            .size(13.0),
                    );
                }
            });

            ui.add_space(8.0);

            // Control buttons — full width row
            ui.horizontal(|ui| {
                let btn_w = (ui.available_width() - 16.0) / 3.0;

                // Start
                if ui
                    .add(
                        egui::Button::new(RichText::new("Start").color(theme::ACCENT_GREEN).size(14.0).strong())
                            .fill(Color32::from_rgba_premultiplied(12, 48, 28, 200))
                            .stroke(Stroke::new(1.0, theme::ACCENT_GREEN))
                            .min_size(egui::vec2(btn_w, 44.0))
                            .corner_radius(CornerRadius::same(22)),
                    )
                    .clicked()
                {
                    bot.start();
                    *status_message = "Bot started".to_string();
                }

                ui.add_space(8.0);

                // Pause
                if ui
                    .add(
                        egui::Button::new(RichText::new("Pause").color(theme::ACCENT_YELLOW).size(14.0).strong())
                            .fill(Color32::from_rgba_premultiplied(50, 38, 10, 200))
                            .stroke(Stroke::new(1.0, theme::ACCENT_YELLOW))
                            .min_size(egui::vec2(btn_w, 44.0))
                            .corner_radius(CornerRadius::same(22)),
                    )
                    .clicked()
                {
                    bot.pause();
                    *status_message = "Bot paused".to_string();
                }

                ui.add_space(8.0);

                // Stop
                if ui
                    .add(
                        egui::Button::new(RichText::new("Stop").color(theme::ACCENT_RED).size(14.0).strong())
                            .fill(Color32::from_rgba_premultiplied(50, 12, 12, 200))
                            .stroke(Stroke::new(1.0, theme::ACCENT_RED))
                            .min_size(egui::vec2(btn_w, 44.0))
                            .corner_radius(CornerRadius::same(22)),
                    )
                    .clicked()
                {
                    bot.stop();
                    *status_message = "Bot stopped".to_string();
                }
            });

            ui.add_space(10.0);

            // Stats row
            ui.horizontal_wrapped(|ui| {
                stat_chip(ui, "Cycles/min", &cycle_stats.cycles_last_minute.to_string());
                stat_chip(ui, "Last", &format!("{} ms", cycle_stats.last_cycle_ms));
                stat_chip(ui, "Total", &cycle_stats.total_cycles.to_string());
            });

            if let Some(s) = snapshot {
                if let Some((x, y, z)) = s.position {
                    ui.add_space(4.0);
                    ui.horizontal_wrapped(|ui| {
                        stat_chip(ui, "XYZ", &format!("{x:.1}, {y:.1}, {z:.1}"));
                        if let Some(h) = s.heading_rad {
                            stat_chip(ui, "Hdg", &format!("{h:.3}"));
                        }
                    });
                }
            }
        });

    let rect = resp.response.rect;
    let spec = egui::Rect::from_min_size(
        rect.min + egui::vec2(20.0, 1.5),
        egui::vec2((rect.width() - 40.0).max(0.0), 1.5),
    );
    ui.painter().rect_filled(spec, CornerRadius::same(1), theme::GLASS_SPECULAR);
}

// ── HP bar ────────────────────────────────────────────────────────────────────

pub fn draw_hp_bar(ui: &mut egui::Ui, current: Option<u32>, max: Option<u32>) {
    let size = egui::vec2(ui.available_width(), HP_BAR_HEIGHT);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let painter = ui.painter();
    let r = CornerRadius::same(HP_BAR_HEIGHT as u8 / 2);

    painter.rect_filled(rect, r, theme::GLASS_FILL_INSET);
    painter.rect_stroke(rect, r, Stroke::new(1.0, theme::GLASS_BORDER), egui::StrokeKind::Inside);

    let ratio = match (current, max) {
        (Some(c), Some(m)) if m > 0 => (c as f32 / m as f32).clamp(0.0, 1.0),
        _ => 0.0,
    };

    if ratio > 0.0 {
        let bar_col = theme::health_color(ratio);
        let mut fill = rect;
        fill.max.x = rect.min.x + rect.width() * ratio;
        let fill_r = if ratio < 0.97 {
            CornerRadius { nw: r.nw, sw: r.sw, ne: 4, se: 4 }
        } else {
            r
        };
        painter.rect_filled(fill, fill_r, bar_col);

        let mut hi = fill;
        hi.max.y = hi.min.y + 5.0;
        painter.rect_filled(
            hi,
            CornerRadius { nw: r.nw, sw: 0, ne: 3, se: 0 },
            Color32::from_rgba_premultiplied(255, 255, 255, 35),
        );
    }

    let text = match (current, max) {
        (Some(c), Some(m)) if m > 0 => format!("{c} / {m}  ({:.0}%)", c as f32 / m as f32 * 100.0),
        _ => "—".to_string(),
    };
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        egui::FontId::proportional(12.0),
        Color32::from_rgba_premultiplied(255, 255, 255, 200),
    );
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn stat_chip(ui: &mut egui::Ui, key: &str, val: &str) {
    Frame::new()
        .fill(theme::GLASS_FILL_INSET)
        .inner_margin(Margin::symmetric(8, 4))
        .corner_radius(CornerRadius::same(10))
        .stroke(Stroke::new(1.0, theme::GLASS_BORDER))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(key).color(theme::TEXT_DIM).size(10.0));
                ui.label(RichText::new(val).color(theme::TEXT_SECONDARY).size(11.0).strong());
            });
        });
}

fn bot_state_display(bot: &BotRuntime) -> (&'static str, &'static str, egui::Color32) {
    match bot.state() {
        BotState::Running => ("●", "Running", theme::ACCENT_GREEN),
        BotState::Paused => ("◐", "Paused", theme::ACCENT_YELLOW),
        BotState::Stopped => ("○", "Stopped", theme::TEXT_DIM),
    }
}

fn hostility_color(hostility: &str) -> egui::Color32 {
    match hostility {
        "Hostile" => theme::ACCENT_RED,
        "Friendly" => theme::ACCENT_GREEN,
        _ => theme::ACCENT_YELLOW,
    }
}

pub fn race_name(race_id: u8) -> &'static str {
    match race_id {
        1 => "Human", 2 => "Orc", 3 => "Dwarf", 4 => "Night Elf",
        5 => "Undead", 6 => "Tauren", 7 => "Gnome", 8 => "Troll",
        10 => "Blood Elf", 11 => "Draenei", _ => "Unknown",
    }
}
