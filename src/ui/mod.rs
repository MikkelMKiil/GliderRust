mod panels;

use std::path::PathBuf;
use std::time::{Duration, Instant};

use eframe::egui::Ui;

use crate::bot::BotRuntime;
use crate::config::AppConfig;
use crate::memory::{MemoryReader, MemorySnapshot};

pub struct CycleStats {
    pub interval_ms: u64,
    pub cycles_last_minute: u64,
    pub total_cycles: u64,
    pub last_cycle_ms: u64,
}

pub struct GliderApp {
    config: AppConfig,
    bot: BotRuntime,
    memory_reader: MemoryReader,
    snapshot: Option<MemorySnapshot>,
    pid_input: String,
    profile_path_input: String,
    last_profile_dir: Option<PathBuf>,
    status_message: String,
    last_cycle_at: Option<Instant>,
    cycle_window_started: Instant,
    cycles_last_minute: u64,
    total_cycles: u64,
    last_cycle_ms: u64,
}

impl Default for GliderApp {
    fn default() -> Self {
        Self {
            config: AppConfig::default(),
            bot: BotRuntime::default(),
            memory_reader: MemoryReader::default(),
            snapshot: None,
            pid_input: String::new(),
            profile_path_input: String::new(),
            last_profile_dir: None,
            status_message: String::new(),
            last_cycle_at: None,
            cycle_window_started: Instant::now(),
            cycles_last_minute: 0,
            total_cycles: 0,
            last_cycle_ms: 0,
        }
    }
}

impl eframe::App for GliderApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let cycle_stats = CycleStats {
            interval_ms: self.config.memory_poll_ms,
            cycles_last_minute: self.cycles_last_minute,
            total_cycles: self.total_cycles,
            last_cycle_ms: self.last_cycle_ms,
        };

        panels::status::draw(
            ui,
            &mut self.bot,
            &mut self.memory_reader,
            &mut self.snapshot,
            &mut self.pid_input,
            &mut self.profile_path_input,
            &mut self.last_profile_dir,
            &cycle_stats,
            &mut self.status_message,
        );

        self.run_scheduled_cycle();
    }
}

impl GliderApp {
    fn run_scheduled_cycle(&mut self) {
        if self.bot.state() != crate::bot::BotState::Running {
            return;
        }

        let now = Instant::now();
        let interval = Duration::from_millis(self.config.memory_poll_ms);
        let is_due = self
            .last_cycle_at
            .map(|last| now.duration_since(last) >= interval)
            .unwrap_or(true);

        if !is_due {
            return;
        }

        if now.duration_since(self.cycle_window_started) >= Duration::from_secs(60) {
            self.cycle_window_started = now;
            self.cycles_last_minute = 0;
        }

        let cycle_started = Instant::now();

        match self.memory_reader.read_snapshot() {
            Ok(latest) => {
                self.snapshot = Some(latest);
            }
            Err(err) => {
                self.snapshot = None;
                self.status_message = format!("Snapshot read failed: {err}");
            }
        }

        self.bot.tick(self.snapshot.as_ref());

        self.last_cycle_at = Some(now);
        self.cycles_last_minute += 1;
        self.total_cycles += 1;
        self.last_cycle_ms = cycle_started.elapsed().as_millis() as u64;
    }
}
