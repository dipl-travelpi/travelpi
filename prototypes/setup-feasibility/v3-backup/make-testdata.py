#!/usr/bin/env python3
"""V3: writes a test backup with made-up placeholder values in the structure of /LD070/
(Wi-Fi networks, tunnel profiles, network areas, dashboard login). Usage: make-testdata.py FILE"""
import base64
import json
import random
import sys

random.seed(2026)  # same file on every run


def fake_key():
    return base64.b64encode(random.randbytes(32)).decode()


def fake_mac():
    return ":".join(f"{random.randrange(256):02x}" for _ in range(6))


backup = {
    "version": 1,
    "login": {"username": "admin", "password_hash": "$argon2id$v=19$m=65536,t=3,p=4$" + fake_key()},
    "wifi_networks": [
        {"ssid": f"Hotel-Guest-{i:02}", "password": fake_key()[:16], "security": "WPA2",
         "last_signal_dbm": -random.randrange(40, 90)}
        for i in range(20)
    ],
    "tunnel_profiles": [
        {"name": f"Profile {i}", "server": f"vpn{i}.example.org:51820",
         "config": f"[Interface]\nPrivateKey = {fake_key()}\nAddress = 10.8.0.{i + 2}/32\n\n"
                   f"[Peer]\nPublicKey = {fake_key()}\nEndpoint = vpn{i}.example.org:51820\nAllowedIPs = 0.0.0.0/0\n"}
        for i in range(5)
    ],
    "network_areas": [
        {"name": "Secure", "type": "secure", "devices": [fake_mac() for _ in range(4)]},
        {"name": "Guests", "type": "guest", "devices": [fake_mac() for _ in range(2)]},
    ],
}

with open(sys.argv[1], "w") as f:
    json.dump(backup, f, indent=2)
with open(sys.argv[1], "rb") as f:
    print(f"test backup: {len(f.read())} bytes")
