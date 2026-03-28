pub mod state;

use crate::memory::types::MemorySnapshot;

pub use state::{BotState, RuntimeStatus};

#[derive(Debug)]
pub struct BotRuntime {
    state: BotState,
    status: RuntimeStatus,
}

impl Default for BotRuntime {
    fn default() -> Self {
        Self {
            state: BotState::Stopped,
            status: RuntimeStatus::Idle,
        }
    }
}

impl BotRuntime {
    pub fn state(&self) -> BotState {
        self.state
    }

    pub fn status(&self) -> RuntimeStatus {
        self.status
    }

    pub fn start(&mut self) {
        self.state = BotState::Running;
        self.status = RuntimeStatus::PollingMemory;
    }

    pub fn pause(&mut self) {
        self.state = BotState::Paused;
        self.status = RuntimeStatus::Paused;
    }

    pub fn stop(&mut self) {
        self.state = BotState::Stopped;
        self.status = RuntimeStatus::Idle;
    }

    pub fn tick(&mut self, snapshot: Option<&MemorySnapshot>) {
        if self.state != BotState::Running {
            return;
        }

        self.status = if snapshot.is_some() {
            RuntimeStatus::RunningProfile
        } else {
            RuntimeStatus::PollingMemory
        };
    }
}
