use eframe::egui;

use crate::bot::BotRuntime;
use crate::memory::{MemoryReader, MemorySnapshot};

pub fn draw(
    ui: &mut egui::Ui,
    bot: &mut BotRuntime,
    reader: &mut MemoryReader,
    snapshot: &mut Option<MemorySnapshot>,
    pid_input: &mut String,
    status_message: &mut String,
) {
    egui::CentralPanel::default().show_inside(ui, |ui| {
        ui.heading("GliderRust");
        ui.label("Windows-only single-process MVP");

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
            if ui.button("Start").clicked() {
                bot.start();
            }
            if ui.button("Pause").clicked() {
                bot.pause();
            }
            if ui.button("Stop").clicked() {
                bot.stop();
            }
        });

        if bot.state() == crate::bot::BotState::Running {
            let latest = reader.read_snapshot().ok();
            *snapshot = latest;
            bot.tick(snapshot.as_ref());
        }

        ui.separator();
        ui.label(format!("Bot state: {:?}", bot.state()));
        ui.label(format!("Runtime status: {:?}", bot.status()));
        ui.label(format!("Memory attached: {}", reader.is_attached()));

        if let Some(s) = snapshot {
            ui.label(format!(
                "Snapshot: name={} hp={} pos=({:.2}, {:.2}, {:.2})",
                s.player_name, s.player_health, s.position.0, s.position.1, s.position.2
            ));
        } else {
            ui.label("Snapshot: none");
        }

        if !status_message.is_empty() {
            ui.separator();
            ui.label(status_message.as_str());
        }
    });
}
