# Daemon

`travelpid` is the system daemon of TravelPi, written in Rust. It runs with system privileges and
is the only component that configures the network, the tunnel and the firewall. `travelpi` is its
command-line client.

## Planned layout

| Path | Content |
|---|---|
| `src/core/` | API server, event stream, socket permissions, simulation mode |
| `src/modules/ap/` | Own Wi-Fi: access point, DHCP, DNS |
| `src/modules/portal/` | Captive portal detection and login mode |
| `src/modules/segmentation/` | Zones for devices |
| `src/modules/hardware/` | Hardware detection |
| `src/modules/config/` | Configuration and secrets |
| `src/modules/backup/` | Backup and restore |
| `src/modules/uplink/` | Upstream connection: scan, connect, credentials |
| `src/modules/tunnel/` | WireGuard tunnel and profiles |
| `src/modules/firewall/` | nftables ruleset and kill switch |
| `src/modules/failover/` | Switching between internet sources |
| `src/modules/adblock/` | DNS blocklist |

Each module lives in its own folder.
