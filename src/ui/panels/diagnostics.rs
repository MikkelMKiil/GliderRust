use std::collections::BTreeMap;

use eframe::egui;

use crate::memory::MemorySnapshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticsVerbosity {
    ErrorsAndWarnings,
    All,
}

impl DiagnosticsVerbosity {
    pub fn label(self) -> &'static str {
        match self {
            Self::ErrorsAndWarnings => "Errors/Warnings",
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
    status_message: &str,
    verbosity: &mut DiagnosticsVerbosity,
) {
    ui.heading("Diagnostics");
    ui.horizontal(|ui| {
        ui.label("View");
        egui::ComboBox::from_id_salt("diag_verbosity")
            .selected_text(verbosity.label())
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

    let mut lines: Vec<String> = Vec::new();
    if !status_message.is_empty() {
        lines.push(format!("status {status_message}"));
    }

    if let Some(snapshot) = snapshot {
        for line in &snapshot.diagnostics {
            if verbosity.includes(line) {
                lines.push(line.clone());
            }
        }
    }

    if lines.is_empty() {
        ui.separator();
        ui.label("No diagnostics available");
        return;
    }

    let mut grouped: BTreeMap<&'static str, Vec<String>> = BTreeMap::new();
    for line in lines {
        grouped
            .entry(category_for(&line))
            .or_default()
            .push(line);
    }

    ui.separator();
    for (category, items) in grouped {
        egui::CollapsingHeader::new(format!("{category} ({})", items.len()))
            .default_open(matches!(category, "Traversal" | "Health" | "Model Chain"))
            .show(ui, |ui| {
                for line in items {
                    ui.label(line);
                }
            });
    }
}

fn category_for(line: &str) -> &'static str {
    let lower = line.to_ascii_lowercase();

    if lower.contains("display") || lower.contains("model") || lower.contains("monster_def") {
        return "Model Chain";
    }

    if lower.contains("traversal") || lower.contains("first object") || lower.contains("guid") {
        return "Traversal";
    }

    if lower.contains("health") || lower.contains("hp") {
        return "Health";
    }

    if lower.contains("target") {
        return "Target";
    }

    if lower.contains("attach") || lower.contains("pid") || lower.contains("process") {
        return "Attach/Process";
    }

    "General"
}
