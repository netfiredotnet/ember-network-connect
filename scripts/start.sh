#!/usr/bin/env bash

export DBUS_SYSTEM_BUS_ADDRESS=unix:path=/host/run/dbus/system_bus_socket

# Fetch the name of the current WiFi interface, as they can be variable
wifi_interface_name=$(iw dev | awk '$1=="Interface"{print $2}')

# Update the list of available WiFi networks before launch
iw dev "$wifi_interface_name" scan &> /dev/null || printf 'Error updating WiFi network list with IW\n'

# Configuration via environment variables with defaults
SSID="${EMBER_WIFI_SSID:-NetFire Ember}"
PASSWORD="${EMBER_WIFI_PASSWORD:-}"
ACTIVITY_TIMEOUT="${EMBER_ACTIVITY_TIMEOUT:-120}"
NETWORK_TIMEOUT="${EMBER_NETWORK_TIMEOUT:-300}"
ETH_INTERFACE="${EMBER_ETHERNET_INTERFACE:-eth0}"

# Build command arguments
CMD_ARGS="-s \"$SSID\" -a $ACTIVITY_TIMEOUT -n $NETWORK_TIMEOUT -e $ETH_INTERFACE"
if [ -n "$PASSWORD" ]; then
    CMD_ARGS="$CMD_ARGS -p \"$PASSWORD\""
fi

# Launch Ember Network Connect
printf 'Starting Ember Network Connect\n'
printf '  SSID: %s\n' "$SSID"
printf '  Ethernet interface: %s\n' "$ETH_INTERFACE"
printf '  Activity timeout: %s seconds\n' "$ACTIVITY_TIMEOUT"
printf '  Overall timeout: %s seconds\n' "$NETWORK_TIMEOUT"

# Sleep infinity when we exit successfully; this has the effect of deactivating 
# the AP and keeping it that way until an update or device reboot.
eval ./ember-network-connect $CMD_ARGS && sleep infinity
