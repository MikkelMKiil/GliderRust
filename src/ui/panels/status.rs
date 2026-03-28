use eframe::egui;
use std::path::PathBuf;

use crate::bot::BotRuntime;
use crate::memory::{MemoryReader, MemorySnapshot};
use crate::profile::load_profile;
use crate::ui::CycleStats;

pub fn draw(
    ui: &mut egui::Ui,
    bot: &mut BotRuntime,
    reader: &mut MemoryReader,
    snapshot: &mut Option<MemorySnapshot>,
    pid_input: &mut String,
    profile_path_input: &mut String,
    last_profile_dir: &mut Option<PathBuf>,
    cycle_stats: &CycleStats,
    status_message: &mut String,
) {
    ui.heading("Runtime");

    ui.horizontal(|ui| {
        ui.label("WoW PID");
        ui.text_edit_singleline(pid_input);

        if ui.button("Attach").clicked() {
            let parsed_pid = pid_input.trim().parse::<u32>();
            match parsed_pid {
                Ok(pid) => match reader.attach(pid) {
                    Ok(()) => *status_message = format!("Attached to PID {pid}"),
                    Err(err) => *status_message = format!("Attach failed: {err}"),
                },
                Err(_) => *status_message = "PID must be a valid number".to_string(),
            }
        }

        if ui.button("Auto Attach WoW").clicked() {
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

    ui.horizontal(|ui| {
        ui.label("Profile path");
        ui.text_edit_singleline(profile_path_input);

        if ui.button("Browse...").clicked() {
            let mut dialog = rfd::FileDialog::new().add_filter("Profile XML", &["xml"]);
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
                        *status_message = format!("Loaded profile with {wp_count} waypoints");
                    }
                    Err(err) => {
                        *status_message = format!("Profile load failed: {err}");
                    }
                }
            }
        }

        if ui.button("Clear Profile").clicked() {
            bot.clear_profile();
            *status_message = "Profile cleared".to_string();
        }
    });

    ui.separator();
    ui.label(format!("Bot state: {:?}", bot.state()));
    ui.label(format!("Runtime status: {:?}", bot.status()));
    ui.label(format!("Memory attached: {}", reader.is_attached()));
    ui.label(format!(
        "Cycle interval: {} ms (24/min target)",
        cycle_stats.interval_ms
    ));
    ui.label(format!(
        "Cycles in current minute window: {}",
        cycle_stats.cycles_last_minute
    ));
    ui.label(format!("Total cycles: {}", cycle_stats.total_cycles));
    ui.label(format!("Last cycle duration: {} ms", cycle_stats.last_cycle_ms));
    ui.label(format!(
        "Active profile: {}",
        bot.active_profile_name().unwrap_or("none")
    ));

    if let Some((current, total)) = bot.waypoint_progress() {
        ui.label(format!("Waypoint: {current}/{total}"));
    }

    let nav = bot.nav_output();
    ui.label(format!(
        "Nav: desired_heading={:.3?} error={:.3?} distance={:.2?} reached={}",
        nav.desired_heading_rad,
        nav.heading_error_rad,
        nav.distance_to_waypoint,
        nav.waypoint_reached
    ));

    ui.label(format!("Suggested inputs: {:?}", bot.suggested_inputs()));

    if let Some(s) = snapshot {
        ui.separator();
        ui.label(format!(
            "Player: {} (GUID: {})",
            s.player_name,
            s.player_guid
                .map(|guid| format!("0x{guid:016X}"))
                .unwrap_or_else(|| "unknown".to_string())
        ));
        ui.label(format!(
            "Player Meta: race={} level={} faction={}",
            race_name(s.player_race),
            s.player_level
                .map(|v| v.to_string())
                .unwrap_or_else(|| "unknown".to_string()),
            s.player_faction
                .map(|v| v.to_string())
                .unwrap_or_else(|| "unknown".to_string())
        ));
        draw_health_bar(
            ui,
            "Player HP",
            s.player_current_health,
            s.player_max_health,
        );
        ui.label(format!(
            "Player Model: object_type={} unit_flags={} display_id={} native_display_id={} monster_def={}",
            format_opt_u32(s.player_object_type),
            format_opt_u32_hex(s.player_unit_flags),
            format_opt_u32(s.player_display_id),
            format_opt_u32(s.player_native_display_id),
            format_opt_usize_hex(s.player_monster_definition_ptr)
        ));

        if s.target_guid.is_some() {
            ui.label(format!(
                "Target: {}",
                s.target_name.as_deref().unwrap_or("(unknown)")
            ));
            ui.label(format!(
                "Target Meta: race={} level={} hostility={} faction={}",
                race_name(s.target_race),
                s.target_level
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "unknown".to_string()),
                s.target_hostility.as_deref().unwrap_or("unknown"),
                s.target_faction
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "unknown".to_string())
            ));
            draw_health_bar(ui, "Target HP", s.target_current_health, s.target_max_health);
            ui.label(format!(
                "Target Model: object_type={} unit_flags={} display_id={} native_display_id={} monster_def={}",
                format_opt_u32(s.target_object_type),
                format_opt_u32_hex(s.target_unit_flags),
                format_opt_u32(s.target_display_id),
                format_opt_u32(s.target_native_display_id),
                format_opt_usize_hex(s.target_monster_definition_ptr)
            ));
        } else {
            ui.label("Target HP: no target selected");
        }

        ui.label(format!("Position: {:?}", s.position));
        ui.label(format!("Heading (rad): {:?}", s.heading_rad));
        ui.label(format!("Target GUID: {:?}", s.target_guid));
        ui.label(format!("Target distance: {:?}", s.target_distance));
    } else {
        ui.label("Snapshot: none");
    }

    if !status_message.is_empty() {
        ui.separator();
        ui.label(status_message.as_str());
    }
}

fn race_name(race_id: Option<u8>) -> &'static str {
    match race_id {
        Some(1) => "Human",
        Some(2) => "Orc",
        Some(3) => "Dwarf",
        Some(4) => "Night Elf",
        Some(5) => "Undead",
        Some(6) => "Tauren",
        Some(7) => "Gnome",
        Some(8) => "Troll",
        Some(10) => "Blood Elf",
        Some(11) => "Draenei",
        _ => "Unknown",
    }
}

fn format_opt_u32(value: Option<u32>) -> String {
    value
        .map(|v| v.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn format_opt_u32_hex(value: Option<u32>) -> String {
    value
        .map(|v| format!("0x{v:08X}"))
        .unwrap_or_else(|| "unknown".to_string())
}

fn format_opt_usize_hex(value: Option<usize>) -> String {
    value
        .map(|v| format!("0x{v:08X}"))
        .unwrap_or_else(|| "unknown".to_string())
}

fn draw_health_bar(ui: &mut egui::Ui, label: &str, current: Option<u32>, max: Option<u32>) {
    ui.label(label);

    let desired_size = egui::vec2(ui.available_width().max(120.0), 20.0);
    let (rect, _) = ui.allocate_exact_size(desired_size, egui::Sense::hover());

    let rounding = egui::CornerRadius::same(4);
    let stroke = egui::Stroke::new(1.0, egui::Color32::BLACK);
    let painter = ui.painter();

    painter.rect(
        rect,
        rounding,
        egui::Color32::from_rgb(170, 20, 20),
        stroke,
        egui::StrokeKind::Outside,
    );

    let fill_ratio = match (current, max) {
        (Some(cur), Some(mx)) if mx > 0 => (cur as f32 / mx as f32).clamp(0.0, 1.0),
        _ => 0.0,
    };

    if fill_ratio > 0.0 {
        let mut green_rect = rect;
        green_rect.max.x = rect.min.x + rect.width() * fill_ratio;
        painter.rect(
            green_rect,
            rounding,
            egui::Color32::from_rgb(20, 180, 20),
            egui::Stroke::NONE,
            egui::StrokeKind::Outside,
        );
    }

    let text = match (current, max) {
        (Some(cur), Some(mx)) if mx > 0 => {
            let percent = (cur as f32 / mx as f32 * 100.0).clamp(0.0, 100.0);
            format!("{cur}/{mx} ({percent:.1}%)")
        }
        _ => "unavailable".to_string(),
    };

    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        egui::FontId::proportional(13.0),
        egui::Color32::WHITE,
    );
}
