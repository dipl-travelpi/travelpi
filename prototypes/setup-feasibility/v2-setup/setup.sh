#!/usr/bin/env bash
# V2, shell variant: shortened TravelPi setup (install packages, write a config file,
# enable a service). Run as root. BAD_PACKAGE=1 adds a package that does not exist.
set -euo pipefail

packages=(hostapd dnsmasq wireguard-tools)
[[ ${BAD_PACKAGE:-} == 1 ]] && packages+=(travelpi-does-not-exist)

config_file=/etc/travelpi/travelpi.conf
config='# TravelPi test configuration (feasibility test V2)
interface=wlan0
ssid=TravelPi-Test'

unit_file=/etc/systemd/system/travelpi-test.service
unit='[Unit]
Description=TravelPi feasibility test service

[Service]
ExecStart=/bin/sleep infinity

[Install]
WantedBy=multi-user.target'

# Only install what is missing, so a second run does not touch apt
missing=()
for package in "${packages[@]}"; do
  dpkg-query -W -f='${Status}' "$package" 2>/dev/null | grep -q "install ok installed" || missing+=("$package")
done
if (( ${#missing[@]} )); then
  apt-get update
  DEBIAN_FRONTEND=noninteractive apt-get install -y "${missing[@]}"
fi

# Only write files whose content differs
mkdir -p /etc/travelpi
if [[ ! -f $config_file || $(< "$config_file") != "$config" ]]; then
  printf '%s\n' "$config" > "$config_file"
fi
if [[ ! -f $unit_file || $(< "$unit_file") != "$unit" ]]; then
  printf '%s\n' "$unit" > "$unit_file"
  systemctl daemon-reload
fi

systemctl enable --now travelpi-test.service
