use std::net::Ipv4Addr;
use std::process::Child;
use std::sync::Arc;
use std::time::Duration;

use network_manager::{Connection, Device, DeviceState, DeviceType, NetworkManager, ServiceState};
use tokio::sync::mpsc;
use tracing::{debug, error, info};

use crate::config::Config;
use crate::dnsmasq::{start_dnsmasq, stop_dnsmasq};
use crate::errors::{AppError, Result};
use crate::exit::trap_exit_signals;
use crate::server::{start_server, start_timer};

/// Commands sent to the network handler
#[derive(Debug)]
pub enum NetworkCommand {
    /// Activity timeout (no user connection)
    ActivityTimeout,
    /// Overall timeout reached
    OverallTimeout,
    /// Exit signal received
    Exit,
    /// User requested DHCP reset
    Reset,
    /// User accessed the portal (resets activity timeout)
    Activate,
}

/// Main network command handler
struct NetworkHandler {
    manager: NetworkManager,
    eth_device: Device,
    portal_connection: Option<Connection>,
    config: Arc<Config>,
    dnsmasq: Child,
    rx: mpsc::Receiver<NetworkCommand>,
    user_connected: bool,
}

impl NetworkHandler {
    /// Create handler and spawn background tasks
    async fn new(config: Arc<Config>) -> Result<Self> {
        let manager = NetworkManager::new();
        debug!("NetworkManager initialized");

        // Find WiFi device for the access point
        let wifi_device = find_device(&manager, config.interface.as_deref())?;

        // Find ethernet device to reset
        let eth_device = manager
            .get_device_by_interface(&config.ethernet_interface)
            .map_err(|_| AppError::DeviceNotFound(config.ethernet_interface.clone()))?;

        // Verify it's actually an ethernet device
        if eth_device.as_ethernet_device().is_none() {
            return Err(AppError::NotAnEthernetDevice(
                config.ethernet_interface.clone(),
            ));
        }

        // Create WiFi access point
        let portal_connection = Some(create_portal(&wifi_device, &config)?);

        // Start dnsmasq for DHCP/DNS
        let dnsmasq = start_dnsmasq(&config, &wifi_device)?;

        // Create command channel
        let (tx, rx) = mpsc::channel(32);

        // Spawn background tasks
        spawn_server(&config, tx.clone());
        spawn_activity_timeout(config.activity_timeout, tx.clone());
        spawn_overall_timeout(config.overall_timeout, tx.clone());
        spawn_signal_handler(tx);

        Ok(Self {
            manager,
            eth_device,
            portal_connection,
            config,
            dnsmasq,
            rx,
            user_connected: false,
        })
    }

    /// Run the main event loop
    async fn run(&mut self) -> Result<()> {
        loop {
            let Some(cmd) = self.rx.recv().await else {
                return Err(AppError::ChannelClosed);
            };

            match cmd {
                NetworkCommand::Activate => {
                    if !self.user_connected {
                        info!("User connected to captive portal");
                        self.user_connected = true;
                    }
                }
                NetworkCommand::OverallTimeout => {
                    info!("Overall timeout reached, exiting");
                    return Ok(());
                }
                NetworkCommand::ActivityTimeout => {
                    if !self.user_connected {
                        info!("Activity timeout reached, exiting");
                        return Ok(());
                    }
                }
                NetworkCommand::Exit => {
                    info!("Exit signal received");
                    return Ok(());
                }
                NetworkCommand::Reset => {
                    self.reset_to_dhcp()?;
                }
            }
        }
    }

    /// Reset ethernet to DHCP
    fn reset_to_dhcp(&self) -> Result<()> {
        info!("Resetting {} to DHCP", self.config.ethernet_interface);

        // Delete existing wired connections
        if let Ok(connections) = self.manager.get_connections() {
            for conn in connections {
                if conn.settings().kind == "802-3-ethernet" {
                    debug!("Deleting wired connection");
                    let _ = conn.delete();
                }
            }
        }

        // Set DHCP on the ethernet device
        let ethernet = self
            .eth_device
            .as_ethernet_device()
            .ok_or_else(|| AppError::NotAnEthernetDevice(self.config.ethernet_interface.clone()))?;

        ethernet
            .set_dhcp()
            .map_err(|e| AppError::SetDhcp(e.to_string()))?;

        info!("DHCP reset complete");
        Ok(())
    }

    /// Cleanup resources
    fn cleanup(&mut self) {
        let _ = stop_dnsmasq(&mut self.dnsmasq);

        if let Some(ref conn) = self.portal_connection {
            info!("Stopping access point '{}'", self.config.ssid);
            let _ = conn.deactivate();
            let _ = conn.delete();
        }
    }
}

/// Main entry point
pub async fn process_network_commands(config: &Config) -> Result<()> {
    let config = Arc::new(config.clone());
    let mut handler = NetworkHandler::new(config).await?;

    let result = handler.run().await;
    handler.cleanup();
    result
}

/// Initialize networking before starting the handler
pub fn init_networking(config: &Config) -> Result<()> {
    start_network_manager_service()?;

    // Delete any existing AP profile with same SSID
    let manager = NetworkManager::new();
    if let Ok(connections) = manager.get_connections() {
        for conn in connections {
            let settings = conn.settings();
            if settings.kind == "802-11-wireless"
                && settings.mode == "ap"
                && settings.ssid.as_str().ok() == Some(&config.ssid)
            {
                info!("Deleting existing AP profile for '{}'", config.ssid);
                let _ = conn.delete();
            }
        }
    }

    Ok(())
}

/// Find a WiFi device by interface name or auto-detect
fn find_device(manager: &NetworkManager, interface: Option<&str>) -> Result<Device> {
    if let Some(name) = interface {
        let device = manager
            .get_device_by_interface(name)
            .map_err(|_| AppError::DeviceNotFound(name.to_string()))?;
        info!("Using WiFi device: {}", name);
        return Ok(device);
    }

    // Auto-detect first managed WiFi device
    for device in manager.get_devices()? {
        if *device.device_type() == DeviceType::WiFi && device.get_state()? != DeviceState::Unmanaged
        {
            info!("Auto-detected WiFi device: {}", device.interface());
            return Ok(device);
        }
    }

    Err(AppError::NoWiFiDevice)
}

/// Create the captive portal AP
fn create_portal(device: &Device, config: &Config) -> Result<Connection> {
    info!("Creating access point '{}'", config.ssid);

    let wifi = device
        .as_wifi_device()
        .ok_or_else(|| AppError::NotAWiFiDevice(device.interface().to_string()))?;

    let passphrase = config.passphrase.as_deref();
    let (connection, _) = wifi
        .create_hotspot(config.ssid.as_str(), passphrase, Some(config.gateway))
        .map_err(|e| AppError::CreateCaptivePortal(e.to_string()))?;

    info!("Access point '{}' created", config.ssid);
    Ok(connection)
}

/// Ensure NetworkManager service is running
fn start_network_manager_service() -> Result<()> {
    let Ok(state) = NetworkManager::get_service_state() else {
        info!("Cannot get NetworkManager state, assuming it's running");
        return Ok(());
    };

    if state == ServiceState::Active {
        debug!("NetworkManager already running");
        return Ok(());
    }

    info!("Starting NetworkManager service...");
    let state =
        NetworkManager::start_service(15).map_err(|e| AppError::StartNetworkManager(e.to_string()))?;

    if state != ServiceState::Active {
        return Err(AppError::StartActiveNetworkManager);
    }

    info!("NetworkManager started");
    Ok(())
}

// --- Background task spawners ---

fn spawn_server(config: &Config, tx: mpsc::Sender<NetworkCommand>) {
    let gateway = config.gateway;
    let port = config.listening_port;
    let ui_dir = config.ui_directory();

    tokio::spawn(async move {
        if let Err(e) = start_server(gateway, port, tx, ui_dir).await {
            error!("HTTP server error: {}", e);
        }
    });
}

fn spawn_activity_timeout(timeout: u64, tx: mpsc::Sender<NetworkCommand>) {
    if timeout == 0 {
        return;
    }

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(timeout)).await;
        let _ = tx.send(NetworkCommand::ActivityTimeout).await;
    });
}

fn spawn_overall_timeout(timeout: u64, tx: mpsc::Sender<NetworkCommand>) {
    if timeout == 0 {
        return;
    }

    start_timer(timeout, tx);
}

fn spawn_signal_handler(tx: mpsc::Sender<NetworkCommand>) {
    tokio::spawn(async move {
        if let Err(e) = trap_exit_signals(tx).await {
            error!("Signal handler error: {}", e);
        }
    });
}
