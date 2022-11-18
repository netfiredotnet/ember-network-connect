use std::collections::HashSet;
use std::error::Error;
use std::net::Ipv4Addr;
use std::process;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Duration;

use config::Config;
use dnsmasq::{start_dnsmasq, stop_dnsmasq};
use errors::*;
use exit::{exit, trap_exit_signals, ExitResult};
use network_manager::{
    AccessPoint, AccessPointCredentials, Connection, ConnectionState, Connectivity, Device,
    DeviceState, DeviceType, EthernetDevice, NetworkManager, Security, ServiceState,
};
use server::{start_server, start_timer};
use std::string::ToString;

pub enum NetworkCommand {
    Timeout,
    OverallTimeout,
    Exit,
    Reset,
    Activate,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Network {
    ssid: String,
    security: String,
}

pub enum NetworkCommandResponse {
    Networks(Vec<Network>),
    Timer(u64),
}

struct NetworkCommandHandler {
    manager: NetworkManager,
    device: Device,
    eth_device: Device,
    portal_connection: Option<Connection>,
    config: Config,
    dnsmasq: process::Child,
    server_tx: Sender<NetworkCommandResponse>,
    network_rx: Receiver<NetworkCommand>,
    activated: bool,
}

impl NetworkCommandHandler {
    fn new(config: &Config, exit_tx: &Sender<ExitResult>) -> Result<Self> {
        let (network_tx, network_rx) = channel();

        Self::spawn_trap_exit_signals(exit_tx, network_tx.clone());

        let manager = NetworkManager::new();
        debug!("NetworkManager connection initialized");

        let device = find_device(&manager, &config.interface)?;
        let eth_device = find_device(&manager, &Some(String::from("eth0")))?;

        let portal_connection = Some(create_portal(&device, config)?);

        let dnsmasq = start_dnsmasq(config, &device)?;

        let (server_tx, server_rx) = channel();

        Self::spawn_server(config, exit_tx, server_rx, network_tx.clone());

        Self::spawn_activity_timeout(config, network_tx.clone());

        let config = config.clone();
        let activated = false;

        Ok(NetworkCommandHandler {
            manager,
            device,
            eth_device,
            portal_connection,
            config,
            dnsmasq,
            server_tx,
            network_rx,
            activated,
        })
    }

    fn spawn_server(
        config: &Config,
        exit_tx: &Sender<ExitResult>,
        server_rx: Receiver<NetworkCommandResponse>,
        network_tx: Sender<NetworkCommand>,
    ) {
        let gateway = config.gateway;
        let listening_port = config.listening_port;
        let exit_tx_server = exit_tx.clone();
        let ui_directory = config.ui_directory.clone();

        let overall_timeout = config.overall_timeout;

        if overall_timeout != 0 {
            let net_tx = network_tx.clone();
            thread::spawn(move || {
                start_timer(overall_timeout, net_tx);
            });
        }

        thread::spawn(move || {
            start_server(
                gateway,
                listening_port,
                server_rx,
                network_tx,
                exit_tx_server,
                &ui_directory,
            );
        });
    }

    fn spawn_activity_timeout(config: &Config, network_tx: Sender<NetworkCommand>) {
        let activity_timeout = config.activity_timeout;

        if activity_timeout == 0 {
            return;
        }

        thread::spawn(move || {
            thread::sleep(Duration::from_secs(activity_timeout));

            if let Err(err) = network_tx.send(NetworkCommand::Timeout) {
                error!(
                    "Sending NetworkCommand::Timeout failed: {}",
                    err.description()
                );
            }
        });
    }

    fn spawn_trap_exit_signals(exit_tx: &Sender<ExitResult>, network_tx: Sender<NetworkCommand>) {
        let exit_tx_trap = exit_tx.clone();

        thread::spawn(move || {
            if let Err(e) = trap_exit_signals() {
                exit(&exit_tx_trap, e);
                return;
            }

            if let Err(err) = network_tx.send(NetworkCommand::Exit) {
                error!("Sending NetworkCommand::Exit failed: {}", err.description());
            }
        });
    }

    fn run(&mut self, exit_tx: &Sender<ExitResult>) {
        let result = self.run_loop();
        self.stop(exit_tx, result);
    }

    fn run_loop(&mut self) -> ExitResult {
        loop {
            let command = self.receive_network_command()?;

            match command {
                NetworkCommand::Activate => {
                    if !self.activated {
                        info!("User connected to the captive portal");
                        self.activated = true;
                    }
                },
                NetworkCommand::OverallTimeout => {
                    info!("Overall timeout reached. Exiting...");
                    return Ok(());
                },
                NetworkCommand::Timeout => {
                    if !self.activated {
                        info!("Connect timeout reached. Exiting...");
                        return Ok(());
                    }
                },
                NetworkCommand::Exit => {
                    info!("Exiting...");
                    return Ok(());
                },
                NetworkCommand::Reset => {
                    self.set_dhcp();
                },
            }
        }
    }

    fn receive_network_command(&self) -> Result<NetworkCommand> {
        match self.network_rx.recv() {
            Ok(command) => Ok(command),
            Err(e) => {
                // Sleep for a second, so that other threads may log error info.
                thread::sleep(Duration::from_secs(1));
                Err(e).chain_err(|| ErrorKind::RecvNetworkCommand)
            },
        }
    }

    fn stop(&mut self, exit_tx: &Sender<ExitResult>, result: ExitResult) {
        let _ = stop_dnsmasq(&mut self.dnsmasq);

        if let Some(ref connection) = self.portal_connection {
            let _ = stop_portal_impl(connection, &self.config);
        }

        let _ = exit_tx.send(result);
    }

    fn set_dhcp(&mut self) {
        info!("DHCP reset requested. Removing all existing wired connections...");
        delete_existing_wired_connections(&self.manager);

        // let devices = &self.manager.get_devices().unwrap();
        // for d in devices {
        //     info!("Path: {} | Type: {} | Interface: {}", d.path(), d.device_type().to_string(), d.interface());
        //     // let i = devices.iter().position(|ref d| *d.device_type() == DeviceType::Ethernet).unwrap();
        //     // d.connect().unwrap();
        // }

        info!("Creating new DHCP connection profile...");
        let ethernet_device = self.eth_device.as_ethernet_device().unwrap();
        match ethernet_device.set_dhcp() {
            Err(e) => {
                error!("Setting DHCP failed: {}", e.description());
            },
            Ok(_) => (),
        };
    }
}

fn init_access_point_credentials(
    access_point: &AccessPoint,
    identity: &str,
    passphrase: &str,
) -> AccessPointCredentials {
    if access_point.security.contains(Security::ENTERPRISE) {
        AccessPointCredentials::Enterprise {
            identity: identity.to_string(),
            passphrase: passphrase.to_string(),
        }
    } else if access_point.security.contains(Security::WPA2)
        || access_point.security.contains(Security::WPA)
    {
        AccessPointCredentials::Wpa {
            passphrase: passphrase.to_string(),
        }
    } else if access_point.security.contains(Security::WEP) {
        AccessPointCredentials::Wep {
            passphrase: passphrase.to_string(),
        }
    } else {
        AccessPointCredentials::None
    }
}

pub fn process_network_commands(config: &Config, exit_tx: &Sender<ExitResult>) {
    let mut command_handler = match NetworkCommandHandler::new(config, exit_tx) {
        Ok(command_handler) => command_handler,
        Err(e) => {
            exit(exit_tx, e);
            return;
        },
    };

    command_handler.run(exit_tx);
}

pub fn init_networking(config: &Config) -> Result<()> {
    start_network_manager_service()?;

    delete_exising_wifi_connect_ap_profile(&config.ssid).chain_err(|| ErrorKind::DeleteAccessPoint)
}

pub fn find_device(manager: &NetworkManager, interface: &Option<String>) -> Result<Device> {
    if let Some(ref interface) = *interface {
        let device = manager
            .get_device_by_interface(interface)
            .chain_err(|| ErrorKind::DeviceByInterface(interface.clone()))?;

        info!("Targeted device: {}", interface);

        Ok(device)
    } else {
        let devices = manager.get_devices()?;

        if let Some(device) = find_wifi_managed_device(devices)? {
            info!("WiFi device: {}", device.interface());
            Ok(device)
        } else {
            bail!(ErrorKind::NoWiFiDevice)
        }
    }
}

fn find_wifi_managed_device(devices: Vec<Device>) -> Result<Option<Device>> {
    for device in devices {
        if *device.device_type() == DeviceType::WiFi
            && device.get_state()? != DeviceState::Unmanaged
        {
            return Ok(Some(device));
        }
    }

    Ok(None)
}

fn get_networks(access_points: &[AccessPoint]) -> Vec<Network> {
    access_points
        .iter()
        .map(|ap| get_network_info(ap))
        .collect()
}

fn get_network_info(access_point: &AccessPoint) -> Network {
    Network {
        ssid: access_point.ssid().as_str().unwrap().to_string(),
        security: get_network_security(access_point).to_string(),
    }
}

fn get_network_security(access_point: &AccessPoint) -> &str {
    if access_point.security.contains(Security::ENTERPRISE) {
        "enterprise"
    } else if access_point.security.contains(Security::WPA2)
        || access_point.security.contains(Security::WPA)
    {
        "wpa"
    } else if access_point.security.contains(Security::WEP) {
        "wep"
    } else {
        "none"
    }
}

fn create_portal(device: &Device, config: &Config) -> Result<Connection> {
    let portal_passphrase = config.passphrase.as_ref().map(|p| p as &str);

    create_portal_impl(device, &config.ssid, &config.gateway, &portal_passphrase)
        .chain_err(|| ErrorKind::CreateCaptivePortal)
}

fn create_portal_impl(
    device: &Device,
    ssid: &str,
    gateway: &Ipv4Addr,
    passphrase: &Option<&str>,
) -> Result<Connection> {
    info!("Starting access point...");
    let wifi_device = device.as_wifi_device().unwrap();
    let (portal_connection, _) = wifi_device.create_hotspot(ssid, *passphrase, Some(*gateway))?;
    info!("Access point '{}' created", ssid);
    Ok(portal_connection)
}

fn stop_portal(connection: &Connection, config: &Config) -> Result<()> {
    stop_portal_impl(connection, config).chain_err(|| ErrorKind::StopAccessPoint)
}

fn stop_portal_impl(connection: &Connection, config: &Config) -> Result<()> {
    info!("Stopping access point '{}'...", config.ssid);
    connection.deactivate()?;
    connection.delete()?;
    thread::sleep(Duration::from_secs(1));
    info!("Access point '{}' stopped", config.ssid);
    Ok(())
}

fn wait_for_connectivity(manager: &NetworkManager, timeout: u64) -> Result<bool> {
    let mut total_time = 0;

    loop {
        let connectivity = manager.get_connectivity()?;

        if connectivity == Connectivity::Full || connectivity == Connectivity::Limited {
            debug!(
                "Connectivity established: {:?} / {}s elapsed",
                connectivity, total_time
            );

            return Ok(true);
        } else if total_time >= timeout {
            debug!(
                "Timeout reached in waiting for connectivity: {:?} / {}s elapsed",
                connectivity, total_time
            );

            return Ok(false);
        }

        ::std::thread::sleep(::std::time::Duration::from_secs(1));

        total_time += 1;

        debug!(
            "Still waiting for connectivity: {:?} / {}s elapsed",
            connectivity, total_time
        );
    }
}

pub fn start_network_manager_service() -> Result<()> {
    let state = match NetworkManager::get_service_state() {
        Ok(state) => state,
        _ => {
            info!("Cannot get the NetworkManager service state");
            return Ok(());
        },
    };

    if state != ServiceState::Active {
        let state =
            NetworkManager::start_service(15).chain_err(|| ErrorKind::StartNetworkManager)?;
        if state != ServiceState::Active {
            bail!(ErrorKind::StartActiveNetworkManager);
        } else {
            info!("NetworkManager service started successfully");
        }
    } else {
        debug!("NetworkManager service already running");
    }

    Ok(())
}

fn delete_exising_wifi_connect_ap_profile(ssid: &str) -> Result<()> {
    let manager = NetworkManager::new();

    for connection in &manager.get_connections()? {
        if is_access_point_connection(connection) && is_same_ssid(connection, ssid) {
            info!(
                "Deleting already created by WiFi Connect access point connection profile: {:?}",
                connection.settings().ssid,
            );
            connection.delete()?;
        }
    }

    Ok(())
}

fn delete_existing_wired_connections(manager: &NetworkManager) {
    let connections = match manager.get_connections() {
        Ok(connections) => connections,
        Err(e) => {
            error!("Getting existing connections failed: {}", e);
            return;
        },
    };

    for connection in &connections {
        if is_wired_connection(connection) {
            info!("Deleting existing wired connection");

            if let Err(e) = connection.delete() {
                error!("Deleting existing wired connection failed: {}", e);
            }
        }
    }
}

fn is_same_ssid(connection: &Connection, ssid: &str) -> bool {
    connection_ssid_as_str(connection) == Some(ssid)
}

fn connection_ssid_as_str(connection: &Connection) -> Option<&str> {
    // An access point SSID could be random bytes and not a UTF-8 encoded string
    connection.settings().ssid.as_str().ok()
}

fn is_access_point_connection(connection: &Connection) -> bool {
    is_wifi_connection(connection) && connection.settings().mode == "ap"
}

fn is_wifi_connection(connection: &Connection) -> bool {
    connection.settings().kind == "802-11-wireless"
}

fn is_wired_connection(connection: &Connection) -> bool {
    connection.settings().kind == "802-3-ethernet"
}
