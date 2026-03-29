use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub telemetry_enabled: bool,
    pub memory_poll_ms: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            telemetry_enabled: true,
            memory_poll_ms: 2500,
        }
    }
}
