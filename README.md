# Ember Network Connect

> Captive portal for resetting network settings to DHCP

Ember Network Connect is a fork of [balena-os/wifi-connect](https://github.com/balena-os/wifi-connect) adapted for a different use case: resetting a device's Ethernet connection to DHCP addressing via a captive portal.

[![Docker Image](https://img.shields.io/badge/ghcr.io-ember--network--connect-blue)](https://ghcr.io/netfiredotnet/ember-network-connect)
[![License](https://img.shields.io/github/license/netfiredotnet/ember-network-connect.svg)](LICENSE)

---

## What is Ember?

Ember is a versatile edge device offered by [NetFire](https://netfire.com) that enables a wide range of capabilities for its clients, including:

- **Digital signage** - Display dynamic content on screens
- **Failover internet connection control** - Automatic network failover and management
- **Network and server monitoring** - Real-time visibility into infrastructure health
- **Scan-to-cloud** - Seamless document digitization workflows
- And more...

Head to [netfire.com](https://netfire.com) to get started.

---

## How It Works

1. **Device starts captive portal** - The application creates a temporary network configuration and serves a web UI
2. **User connects** - Connect to the device via the captive portal network
3. **User clicks reset** - The web UI shows a "Reset to DHCP" button with a countdown timer
4. **Network resets** - The device's network configuration is reset to use DHCP

This is useful for devices that may have been configured with static IP addresses and need to be reset to DHCP without physical access to reconfigure them.

---

## Installation

### Docker (Recommended)

Pull the image from GitHub Container Registry:

```bash
docker pull ghcr.io/netfiredotnet/ember-network-connect:latest
```

### balena Deployment

Add to your `docker-compose.yml`:

```yaml
version: "2.1"

services:
  network-connect:
    image: ghcr.io/netfiredotnet/ember-network-connect:latest
    network_mode: "host"
    labels:
      io.balena.features.dbus: "1"
    cap_add:
      - NET_ADMIN
    environment:
      DBUS_SYSTEM_BUS_ADDRESS: "unix:path=/host/run/dbus/system_bus_socket"
```

### From Release Binaries

Download pre-built binaries from the [Releases](https://github.com/netfiredotnet/ember-network-connect/releases) page:

- `ember-network-connect-vX.Y.Z-linux-amd64.tar.gz`
- `ember-network-connect-vX.Y.Z-linux-arm64.tar.gz`
- `ember-network-connect-vX.Y.Z-linux-armv7.tar.gz`

---

## Configuration

### Docker Environment Variables

When running the Docker image, configure via these environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `EMBER_WIFI_SSID` | `NetFire Ember` | SSID of the captive portal WiFi network |
| `EMBER_WIFI_PASSWORD` | (none) | WPA2 password for the portal (optional) |
| `EMBER_ACTIVITY_TIMEOUT` | `120` | Exit after N seconds of inactivity |
| `EMBER_NETWORK_TIMEOUT` | `300` | Overall timeout in seconds |

### Command Line Arguments

For direct binary usage, these arguments are available:

| Argument | Environment Variable | Default | Description |
|----------|---------------------|---------|-------------|
| `-i, --portal-interface` | `PORTAL_INTERFACE` | auto | Network interface to use |
| `-s, --portal-ssid` | `PORTAL_SSID` | `WiFi Connect` | SSID of the captive portal |
| `-p, --portal-passphrase` | `PORTAL_PASSPHRASE` | none | WPA2 passphrase for the portal |
| `-g, --portal-gateway` | `PORTAL_GATEWAY` | `192.168.42.1` | Gateway IP address |
| `-d, --portal-dhcp-range` | `PORTAL_DHCP_RANGE` | `192.168.42.2,192.168.42.254` | DHCP range |
| `-o, --portal-listening-port` | `PORTAL_LISTENING_PORT` | `80` | Web server port |
| `-a, --activity-timeout` | `ACTIVITY_TIMEOUT` | `0` (disabled) | Exit after N seconds of inactivity |
| `-n, --overall-timeout` | `OVERALL_TIMEOUT` | `0` (disabled) | Exit after N seconds total |
| `-u, --ui-directory` | `UI_DIRECTORY` | `ui` | Path to web UI files |

---

## Development

### Prerequisites

- [Rust](https://rustup.rs/) (1.76+)
- [Node.js](https://nodejs.org/) (20+)
- [pnpm](https://pnpm.io/) (9+)
- [just](https://github.com/casey/just) command runner

### Quick Start

```bash
# Install pnpm
npm install -g pnpm

# Install just (macOS)
brew install just

# Install just (Linux)
curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to ~/bin

# See available commands
just --list
```

### Development Commands

| Command | Description |
|---------|-------------|
| `just build` | Build everything (UI + Rust) to `out/` |
| `just build-ui` | Build UI only |
| `just build-rust` | Build Rust binary only |
| `just dev-ui` | Run UI dev server with mock API |
| `just dev-ui-backend <url>` | Run UI dev server connected to real backend |
| `just docker-build` | Build Docker image for current architecture |
| `just lint` | Run all linters |
| `just clean` | Remove all build artifacts |

### Project Structure

```
.
├── src/                 # Rust source code
├── ui/                  # React web UI
│   ├── src/
│   └── public/
├── Dockerfile.binary    # Builds standalone binary
├── Dockerfile.runtime   # Final runtime image
└── justfile             # Development commands
```

---

## CI/CD

The project uses GitHub Actions for continuous integration:

- **On push to master**: Builds and pushes Docker image tagged `latest`
- **On version tag (v\*)**: Creates GitHub Release with binaries, pushes Docker image with version tag

### Creating a Release

```bash
# Tag a new version
git tag v4.10.1
git push origin v4.10.1
```

This triggers the CI to:
1. Build binaries for amd64, arm64, armv7
2. Build and push multi-arch Docker image to GHCR
3. Create GitHub Release with downloadable tarballs

---

## License

Ember Network Connect is free software, and may be redistributed under the terms specified in the [LICENSE](LICENSE) file (Apache 2.0).

---

## Acknowledgments

This project is a fork of [balena-os/wifi-connect](https://github.com/balena-os/wifi-connect), originally developed by [balena.io](https://balena.io).
