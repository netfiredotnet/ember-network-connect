use clap::Parser;
use std::net::Ipv4Addr;
use std::path::PathBuf;

const DEFAULT_GATEWAY: &str = "192.168.42.1";
const DEFAULT_DHCP_RANGE: &str = "192.168.42.2,192.168.42.254";
const DEFAULT_SSID: &str = "WiFi Connect";
const DEFAULT_UI_DIRECTORY: &str = "ui";
const DEFAULT_ETHERNET_INTERFACE: &str = "eth0";

#[derive(Parser, Clone, Debug)]
#[command(name = "ember-network-connect")]
#[command(about = "Captive portal for resetting network settings to DHCP")]
#[command(version)]
pub struct Config {
    /// Wireless network interface for the captive portal AP
    #[arg(short = 'i', long = "portal-interface", env = "PORTAL_INTERFACE")]
    pub interface: Option<String>,

    /// Ethernet interface to reset to DHCP
    #[arg(short = 'e', long = "ethernet-interface", env = "ETHERNET_INTERFACE", default_value = DEFAULT_ETHERNET_INTERFACE)]
    pub ethernet_interface: String,

    /// SSID of the captive portal WiFi network
    #[arg(short = 's', long = "portal-ssid", env = "PORTAL_SSID", default_value = DEFAULT_SSID)]
    pub ssid: String,

    /// WPA2 Passphrase of the captive portal WiFi network
    #[arg(short = 'p', long = "portal-passphrase", env = "PORTAL_PASSPHRASE")]
    pub passphrase: Option<String>,

    /// Gateway of the captive portal WiFi network
    #[arg(short = 'g', long = "portal-gateway", env = "PORTAL_GATEWAY", default_value = DEFAULT_GATEWAY)]
    pub gateway: Ipv4Addr,

    /// DHCP range of the WiFi network
    #[arg(short = 'd', long = "portal-dhcp-range", env = "PORTAL_DHCP_RANGE", default_value = DEFAULT_DHCP_RANGE)]
    pub dhcp_range: String,

    /// Listening port of the captive portal web server
    #[arg(short = 'o', long = "portal-listening-port", env = "PORTAL_LISTENING_PORT", default_value = "80")]
    pub listening_port: u16,

    /// Exit if no activity for the specified time (seconds). 0 = disabled.
    #[arg(short = 'a', long = "activity-timeout", env = "ACTIVITY_TIMEOUT", default_value = "0")]
    pub activity_timeout: u64,

    /// Overall timeout - exit after this many seconds regardless of activity. 0 = disabled.
    #[arg(short = 'n', long = "overall-timeout", env = "OVERALL_TIMEOUT", default_value = "0")]
    pub overall_timeout: u64,

    /// Web UI directory location
    #[arg(short = 'u', long = "ui-directory", env = "UI_DIRECTORY")]
    ui_directory_arg: Option<PathBuf>,
}

impl Config {
    /// Get the UI directory, checking multiple locations
    pub fn ui_directory(&self) -> PathBuf {
        if let Some(ref dir) = self.ui_directory_arg {
            return dir.clone();
        }

        if let Some(install_dir) = get_install_ui_directory() {
            return install_dir;
        }

        PathBuf::from(DEFAULT_UI_DIRECTORY)
    }
}

/// Check if running from install path (e.g. /usr/local/sbin -> /usr/local/share/ember-network-connect/ui)
fn get_install_ui_directory() -> Option<PathBuf> {
    let exe_path = std::env::current_exe().ok()?;
    let mut path = exe_path.canonicalize().ok()?;

    path.pop();
    if path.file_name()? != "sbin" {
        return None;
    }

    path.pop();
    path.push("share");
    path.push(env!("CARGO_PKG_NAME"));
    path.push("ui");

    path.is_dir().then_some(path)
}

pub fn get_config() -> Config {
    Config::parse()
}
