pub mod bot;
pub mod config;
pub mod memory;
pub mod net;
pub mod offsets;
pub mod profile;
pub mod ui;

pub fn init_logging() {
    use tracing_subscriber::EnvFilter;

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}
