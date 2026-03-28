#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BotState {
    Stopped,
    Running,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeStatus {
    Idle,
    PollingMemory,
    RunningProfile,
    Paused,
}
