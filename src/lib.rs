pub mod bot;
pub mod backend_api;
pub mod config;
pub mod input;
pub mod memory;
pub mod net;
pub mod offsets;
pub mod profile;
pub mod service;
pub mod ui;

pub fn init_logging() {
    use tracing_subscriber::EnvFilter;

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}
