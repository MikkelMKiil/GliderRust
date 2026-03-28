#[derive(Debug, Clone)]
pub struct AppConfig {
    pub telemetry_enabled: bool,
    pub memory_poll_ms: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            telemetry_enabled: false,
            memory_poll_ms: 100,
        }
    }
}
