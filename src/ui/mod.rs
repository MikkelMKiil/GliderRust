mod panels;

use eframe::egui::Ui;

use crate::bot::BotRuntime;
use crate::memory::{MemoryReader, MemorySnapshot};

#[derive(Default)]
pub struct GliderApp {
    bot: BotRuntime,
    memory_reader: MemoryReader,
    snapshot: Option<MemorySnapshot>,
    pid_input: String,
    status_message: String,
}

impl eframe::App for GliderApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        panels::status::draw(
            ui,
            &mut self.bot,
            &mut self.memory_reader,
            &mut self.snapshot,
            &mut self.pid_input,
            &mut self.status_message,
        );
    }
}
