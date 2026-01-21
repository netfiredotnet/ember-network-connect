use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Nix error: {0}")]
    Nix(#[from] nix::Error),

    #[error("NetworkManager error: {0}")]
    NetworkManager(#[from] network_manager::errors::Error),

    #[error("Cannot find network device '{0}'")]
    DeviceNotFound(String),

    #[error("Device '{0}' is not a WiFi device")]
    NotAWiFiDevice(String),

    #[error("Device '{0}' is not an Ethernet device")]
    NotAnEthernetDevice(String),

    #[error("Cannot find a managed WiFi device")]
    NoWiFiDevice,

    #[error("Creating captive portal failed: {0}")]
    CreateCaptivePortal(String),

    #[error("Stopping access point failed: {0}")]
    StopAccessPoint(String),

    #[error("Deleting connection profile failed: {0}")]
    DeleteConnection(String),

    #[error("Cannot start HTTP server on '{address}': {reason}")]
    StartHttpServer { address: String, reason: String },

    #[error("NetworkManager service failed to reach active state")]
    StartActiveNetworkManager,

    #[error("Starting NetworkManager service failed: {0}")]
    StartNetworkManager(String),

    #[error("Spawning dnsmasq failed: {0}")]
    Dnsmasq(String),

    #[error("Blocking exit signals failed: {0}")]
    BlockExitSignals(String),

    #[error("Trapping exit signals failed: {0}")]
    TrapExitSignals(String),

    #[error("Root privileges required to run {0}")]
    RootPrivilegesRequired(String),

    #[error("Network command channel closed")]
    ChannelClosed,

    #[error("Setting DHCP failed: {0}")]
    SetDhcp(String),
}

pub type Result<T> = std::result::Result<T, AppError>;

/// Map error to exit code
pub fn exit_code(e: &AppError) -> i32 {
    match e {
        AppError::Dnsmasq(_) => 3,
        AppError::ChannelClosed => 7,
        AppError::DeviceNotFound(_) => 10,
        AppError::NotAWiFiDevice(_) => 11,
        AppError::NoWiFiDevice => 12,
        AppError::CreateCaptivePortal(_) => 14,
        AppError::StopAccessPoint(_) => 15,
        AppError::DeleteConnection(_) => 16,
        AppError::StartHttpServer { .. } => 17,
        AppError::StartActiveNetworkManager => 18,
        AppError::StartNetworkManager(_) => 19,
        AppError::BlockExitSignals(_) => 21,
        AppError::TrapExitSignals(_) => 22,
        AppError::RootPrivilegesRequired(_) => 23,
        AppError::NotAnEthernetDevice(_) => 24,
        AppError::SetDhcp(_) => 25,
        _ => 1,
    }
}
