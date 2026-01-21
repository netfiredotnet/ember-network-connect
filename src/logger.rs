use tracing_subscriber::{fmt, EnvFilter};

/// Initialize the tracing subscriber for logging
pub fn init() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        // Default: show info for our crate, warn for others
        EnvFilter::new("ember_network_connect=info,tower_http=warn,warn")
    });

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .without_time()
        .init();
}
