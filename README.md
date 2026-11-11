# TravelPi

A mobile security router on a Raspberry Pi. TravelPi connects to an untrusted network such as a
hotel Wi-Fi, opens its own Wi-Fi for your devices and sends their traffic through an encrypted
VPN tunnel. A kill switch blocks all traffic whenever the tunnel is down. Everything is operated
from a web dashboard.

TravelPi is a diploma project at TGM Wien (school year 2026/27). It is under development and not
ready for use.

## Repository layout

| Folder | Content |
|---|---|
| `api/` | OpenAPI description of the interface between the dashboard and the system daemon |
| `daemon/` | `travelpid`, the system daemon, and `travelpi`, its command-line client (Rust) |
| `dashboard/` | Web dashboard (SvelteKit, TypeScript) |
| `setup/` | First-boot configuration, setup script and systemd units |
| `tests/acceptance/` | Test plan and acceptance tests on the device |
| `docs/` | User documentation |
| `prototypes/` | Tests of the technical feasibility studies; not part of the product |

## How the parts fit together

`travelpid` runs with system privileges and is the only component that configures the network,
the tunnel and the firewall. It offers an HTTP API on a Unix socket. The dashboard runs without
privileges and is a client of that API; so is the `travelpi` command-line tool.

## Target device

Raspberry Pi 3 Model B with Raspberry Pi OS Lite (64-bit) and a USB Wi-Fi adapter for the
upstream connection.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Not decided yet.
