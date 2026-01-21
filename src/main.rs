mod config;
mod dnsmasq;
mod errors;
mod exit;
mod logger;
mod network;
mod privileges;
mod server;

use std::process;

use tracing::error;

use config::get_config;
use errors::exit_code;
use exit::block_exit_signals;
use network::{init_networking, process_network_commands};
use privileges::require_root;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        error!("\x1B[1;31mError: {}\x1B[0m", e);
        process::exit(exit_code(&e));
    }
}

async fn run() -> errors::Result<()> {
    block_exit_signals()?;

    logger::init();

    let config = get_config();

    require_root()?;

    init_networking(&config)?;

    process_network_commands(&config).await
}
