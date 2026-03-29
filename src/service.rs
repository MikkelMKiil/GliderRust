use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::bot::{BotRuntime, BotState, NavigationOutput, RuntimeStatus};
use crate::config::AppConfig;
use crate::input::InputCommand;
use crate::memory::{MemoryReader, MemorySnapshot};
use crate::profile::{load_profile, GlideProfile};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleStatsSnapshot {
    pub interval_ms: u64,
    pub cycles_last_minute: u64,
    pub total_cycles: u64,
    pub last_cycle_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppStateSnapshot {
    pub config: AppConfig,
    pub status_message: String,
    pub memory_attached: bool,
    pub bot_state: BotState,
    pub bot_status: RuntimeStatus,
    pub active_profile_name: Option<String>,
    pub waypoint_progress: Option<(usize, usize)>,
    pub nav_output: NavigationOutput,
    pub suggested_inputs: Vec<InputCommand>,
    pub snapshot: Option<MemorySnapshot>,
    pub cycle_stats: CycleStatsSnapshot,
}

#[derive(Debug)]
pub struct AppService {
    pub config: AppConfig,
    pub bot: BotRuntime,
    pub memory_reader: MemoryReader,
    pub snapshot: Option<MemorySnapshot>,
    pub status_message: String,
    last_cycle_at: Option<Instant>,
    cycle_window_started: Instant,
    cycles_last_minute: u64,
    total_cycles: u64,
    last_cycle_ms: u64,
}

impl Default for AppService {
    fn default() -> Self {
        Self {
            config: AppConfig::default(),
            bot: BotRuntime::default(),
            memory_reader: MemoryReader::default(),
            snapshot: None,
            status_message: String::new(),
            last_cycle_at: None,
            cycle_window_started: Instant::now(),
            cycles_last_minute: 0,
            total_cycles: 0,
            last_cycle_ms: 0,
        }
    }
}

impl AppService {
    pub fn cycle_stats(&self) -> CycleStatsSnapshot {
        CycleStatsSnapshot {
            interval_ms: self.config.memory_poll_ms,
            cycles_last_minute: self.cycles_last_minute,
            total_cycles: self.total_cycles,
            last_cycle_ms: self.last_cycle_ms,
        }
    }

    pub fn state_snapshot(&self) -> AppStateSnapshot {
        AppStateSnapshot {
            config: self.config.clone(),
            status_message: self.status_message.clone(),
            memory_attached: self.memory_reader.is_attached(),
            bot_state: self.bot.state(),
            bot_status: self.bot.status(),
            active_profile_name: self.bot.active_profile_name().map(str::to_string),
            waypoint_progress: self.bot.waypoint_progress(),
            nav_output: self.bot.nav_output().clone(),
            suggested_inputs: self.bot.suggested_inputs(),
            snapshot: self.snapshot.clone(),
            cycle_stats: self.cycle_stats(),
        }
    }

    pub fn start_bot(&mut self) -> Result<(), String> {
        if !self.memory_reader.is_attached() {
            self.status_message = "Cannot start bot: attach to a process first".to_string();
            return Err(self.status_message.clone());
        }

        self.bot.start();
        self.status_message = "Bot started".to_string();
        Ok(())
    }

    pub fn pause_bot(&mut self) {
        self.bot.pause();
        self.status_message = "Bot paused".to_string();
    }

    pub fn stop_bot(&mut self) {
        self.bot.stop();
        self.status_message = "Bot stopped".to_string();
    }

    pub fn set_profile(&mut self, profile: GlideProfile) {
        self.bot.set_profile(profile);
    }

    pub fn load_profile_from_path(&mut self, path: &str) -> Result<usize, String> {
        if path.trim().is_empty() {
            return Err("Profile path cannot be empty".to_string());
        }

        let profile = load_profile(path).map_err(|err| err.to_string())?;
        let wp_count = profile.waypoints.len();
        self.bot.set_profile(profile);
        self.status_message = format!("Loaded profile with {wp_count} waypoints");
        Ok(wp_count)
    }

    pub fn clear_profile(&mut self) {
        self.bot.clear_profile();
        self.status_message = "Profile cleared".to_string();
    }

    pub fn attach_pid(&mut self, pid: u32) -> Result<(), String> {
        self.memory_reader
            .attach(pid)
            .map_err(|err| err.to_string())?;
        self.status_message = format!("Attached to PID {pid}");
        Ok(())
    }

    pub fn attach_wow(&mut self) -> Result<u32, String> {
        let pid = self
            .memory_reader
            .attach_wow()
            .map_err(|err| err.to_string())?;
        self.status_message = format!("Auto-attached to WoW PID {pid}");
        Ok(pid)
    }

    pub fn detach(&mut self) {
        self.memory_reader.detach();
        self.status_message = "Detached from process".to_string();
    }

    pub fn set_memory_poll_ms(&mut self, interval_ms: u64) {
        self.config.memory_poll_ms = interval_ms.clamp(100, 5000);
    }

    pub fn set_telemetry_enabled(&mut self, enabled: bool) {
        self.config.telemetry_enabled = enabled;
    }

    pub fn set_keybind(&mut self, action: &str, key: &str) -> Result<(), String> {
        self.config.keybinds.set_binding(action, key)?;
        self.status_message = format!("Updated keybind '{action}' -> {key}");
        Ok(())
    }

    pub fn set_rotation_slot(&mut self, slot: u8, key: &str) -> Result<(), String> {
        self.config.keybinds.set_rotation_slot(slot, key)?;
        self.status_message = format!("Updated rotation slot {slot} -> {key}");
        Ok(())
    }

    pub fn run_cycle_now(&mut self) {
        let now = Instant::now();
        self.execute_cycle(now);
    }

    pub fn run_scheduled_cycle(&mut self) {
        if self.bot.state() != BotState::Running {
            return;
        }

        let now = Instant::now();
        let interval = Duration::from_millis(self.config.memory_poll_ms);
        let due = self
            .last_cycle_at
            .map(|t| now.duration_since(t) >= interval)
            .unwrap_or(true);

        if !due {
            return;
        }

        self.execute_cycle(now);
    }

    fn execute_cycle(&mut self, now: Instant) {
        if now.duration_since(self.cycle_window_started) >= Duration::from_secs(60) {
            self.cycle_window_started = now;
            self.cycles_last_minute = 0;
        }

        let started = Instant::now();

        match self.memory_reader.read_snapshot() {
            Ok(snap) => self.snapshot = Some(snap),
            Err(err) => {
                self.snapshot = None;
                self.status_message = format!("Snapshot read failed: {err}");
            }
        }

        self.bot.tick(self.snapshot.as_ref());
        self.last_cycle_at = Some(now);
        self.cycles_last_minute += 1;
        self.total_cycles += 1;
        self.last_cycle_ms = started.elapsed().as_millis() as u64;
    }
}

pub type SharedAppService = Arc<Mutex<AppService>>;

pub fn new_shared_app_service() -> SharedAppService {
    Arc::new(Mutex::new(AppService::default()))
}

#[cfg(test)]
mod tests {
    use super::AppService;
    use crate::bot::BotState;

    #[test]
    fn bot_control_methods_update_state() {
        let mut service = AppService::default();
        let pid = std::process::id();

        service
            .attach_pid(pid)
            .expect("attach should work for current process");

        service
            .start_bot()
            .expect("start should work when attached");
        assert_eq!(service.bot.state(), BotState::Running);

        service.pause_bot();
        assert_eq!(service.bot.state(), BotState::Paused);

        service.stop_bot();
        assert_eq!(service.bot.state(), BotState::Stopped);
    }

    #[test]
    fn start_requires_attached_memory() {
        let mut service = AppService::default();

        let error = service
            .start_bot()
            .expect_err("start should fail when not attached");

        assert_eq!(error, "Cannot start bot: attach to a process first");
        assert_eq!(service.bot.state(), BotState::Stopped);
    }
}
