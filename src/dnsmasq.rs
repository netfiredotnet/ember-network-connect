use std::process::{Child, Command};

use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;

use network_manager::Device;

use crate::config::Config;
use crate::errors::{AppError, Result};

/// Start dnsmasq for DHCP and DNS on the portal interface
pub fn start_dnsmasq(config: &Config, device: &Device) -> Result<Child> {
    let args = [
        &format!("--address=/#/{}", config.gateway),
        &format!("--dhcp-range={}", config.dhcp_range),
        &format!("--dhcp-option=option:router,{}", config.gateway),
        &format!("--interface={}", device.interface()),
        "--keep-in-foreground",
        "--bind-interfaces",
        "--except-interface=lo",
        "--conf-file",
        "--no-hosts",
    ];

    Command::new("dnsmasq")
        .args(&args)
        .spawn()
        .map_err(|e| AppError::Dnsmasq(e.to_string()))
}

/// Stop the dnsmasq process
pub fn stop_dnsmasq(dnsmasq: &mut Child) -> Result<()> {
    kill(Pid::from_raw(dnsmasq.id() as i32), Signal::SIGTERM)?;
    dnsmasq.wait()?;
    Ok(())
}
