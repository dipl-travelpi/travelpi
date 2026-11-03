#!/usr/bin/env python3
"""V2, Python variant: shortened TravelPi setup (install packages, write a config file,
enable a service). Run as root. BAD_PACKAGE=1 adds a package that does not exist."""
import os
import subprocess
from pathlib import Path

PACKAGES = ["hostapd", "dnsmasq", "wireguard-tools"]

CONFIG_FILE = Path("/etc/travelpi/travelpi.conf")
CONFIG = """# TravelPi test configuration (feasibility test V2)
interface=wlan0
ssid=TravelPi-Test
"""

UNIT_FILE = Path("/etc/systemd/system/travelpi-test.service")
UNIT = """[Unit]
Description=TravelPi feasibility test service

[Service]
ExecStart=/bin/sleep infinity

[Install]
WantedBy=multi-user.target
"""


def installed(package):
    result = subprocess.run(["dpkg-query", "-W", "-f=${Status}", package], capture_output=True, text=True)
    return result.stdout == "install ok installed"


def write_if_changed(path, content):
    if path.exists() and path.read_text() == content:
        return False
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content)
    return True


def main():
    packages = PACKAGES + (["travelpi-does-not-exist"] if os.environ.get("BAD_PACKAGE") == "1" else [])

    # Only install what is missing, so a second run does not touch apt
    missing = [package for package in packages if not installed(package)]
    if missing:
        subprocess.run(["apt-get", "update"], check=True)
        subprocess.run(["apt-get", "install", "-y", *missing], check=True,
                       env={**os.environ, "DEBIAN_FRONTEND": "noninteractive"})

    write_if_changed(CONFIG_FILE, CONFIG)
    if write_if_changed(UNIT_FILE, UNIT):
        subprocess.run(["systemctl", "daemon-reload"], check=True)

    subprocess.run(["systemctl", "enable", "--now", "travelpi-test.service"], check=True)


if __name__ == "__main__":
    main()
