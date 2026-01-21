use nix::sys::signal::{SigSet, Signal};
use tokio::sync::mpsc;
use tracing::info;

use crate::errors::{AppError, Result};
use crate::network::NetworkCommand;

/// Block exit signals from the main thread with mask inherited by children
pub fn block_exit_signals() -> Result<()> {
    let mask = create_exit_sigmask();
    mask.thread_block()
        .map_err(|e| AppError::BlockExitSignals(e.to_string()))
}

/// Trap exit signals and send Exit command when received
pub async fn trap_exit_signals(network_tx: mpsc::Sender<NetworkCommand>) -> Result<()> {
    // Use tokio's signal handling for async
    let mut sigint = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
        .map_err(|e| AppError::TrapExitSignals(e.to_string()))?;
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .map_err(|e| AppError::TrapExitSignals(e.to_string()))?;
    let mut sigquit = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::quit())
        .map_err(|e| AppError::TrapExitSignals(e.to_string()))?;
    let mut sighup = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::hangup())
        .map_err(|e| AppError::TrapExitSignals(e.to_string()))?;

    tokio::select! {
        _ = sigint.recv() => info!("Received SIGINT"),
        _ = sigterm.recv() => info!("Received SIGTERM"),
        _ = sigquit.recv() => info!("Received SIGQUIT"),
        _ = sighup.recv() => info!("Received SIGHUP"),
    }

    let _ = network_tx.send(NetworkCommand::Exit).await;
    Ok(())
}

fn create_exit_sigmask() -> SigSet {
    let mut mask = SigSet::empty();
    mask.add(Signal::SIGINT);
    mask.add(Signal::SIGQUIT);
    mask.add(Signal::SIGTERM);
    mask.add(Signal::SIGHUP);
    mask
}
