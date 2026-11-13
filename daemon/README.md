# Daemon

`travelpid` is the system daemon of TravelPi, written in Rust. It runs with system privileges and
is the only component that configures the network, the tunnel and the firewall. It offers the
HTTP API described in [`api/openapi.yaml`](../api/openapi.yaml) on a Unix socket. `travelpi`
will be its command-line client.

## Run it on a laptop

```sh
cd daemon
cargo run -- --simulate
```

`--simulate` reports made-up values for every section, so nothing touches the system. The socket
is `./travelpid.sock`; `--socket <path>` sets another one. On the device the default is
`/run/travelpid/api.sock`.

In a second terminal:

```sh
curl --unix-socket travelpid.sock http://localhost/v0/status
curl -N --unix-socket travelpid.sock http://localhost/v0/events
```

The event stream first sends the current state of every section, then the changed section on
every change, and a `: keep-alive` comment every 30 seconds.

## Build for the Pi

Rust is never compiled on the Pi. Until CI builds it, build in a Debian 13 container, which has
the same C library as the Pi. Run this in the repository root:

```sh
docker run --rm -v "$PWD":/src -v travelpi-cargo:/usr/local/cargo/registry -w /src/daemon \
  rust:1-trixie cargo build --release --target-dir target/pi
```

On an ARM computer (Apple Silicon) Docker runs ARM containers, so the result is the binary for
the Pi: `daemon/target/pi/release/travelpid`. On an x86 computer add `--platform linux/arm64`
(slower, emulated). Copy it with `scp` and start it there, for example with `--simulate`.

## Checks

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

The tests also read `api/openapi.yaml` and check that the examples there and the `required`
fields fit the Rust types, so code and contract cannot drift apart unnoticed.

## Layout

| Path | Content |
|---|---|
| `src/main.rs` | Command-line options, starts the modules and the server |
| `src/core/status.rs` | The sections of the API as Rust types |
| `src/core/reporter.rs` | How a module reports its section |
| `src/core/api.rs` | HTTP routes: `/v0/status`, `/v0/events`, error responses |
| `src/core/server.rs` | Unix socket: permissions, stale socket, stop signal |
| `src/core/simulation.rs` | Simulation mode, also the example of a module |
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

Each module lives in its own folder. The `modules/` folders are created by their owners.

## Adding a module

A module is an async function that gets a `Reporter` for its section and calls `set` with the
**whole** section whenever something changes. `src/core/simulation.rs` shows two complete
examples.

```rust
// src/modules/ap/mod.rs
use crate::core::reporter::Reporter;
use crate::core::status::ApStatus;

pub async fn run(reporter: Reporter<ApStatus>) {
    loop {
        let status = ApStatus { /* every field */ };
        reporter.set(status);
        // wait until something changes ...
    }
}
```

Then declare it in `src/modules/mod.rs` (`pub mod ap;`, the first module also adds `mod modules;`
to `src/main.rs`) and start it in `src/main.rs`, in the branch without `--simulate`:

```rust
spawn_module(ap_reporter, modules::ap::run);
```

What the core does for you:

- it keeps the latest value for `GET /v0/status`
- it sends an event to every client of `/v0/events`, and nothing if the value did not change
- before the first `set`, the section shows its initial state (`state: unavailable`,
  everything else `null`), never "running" or "connected"
- if `run` ends or panics, it logs it, sets `state: failed` and `error: module stopped`, and
  starts `run` again after a pause (5 s, doubled up to 60 s)

What the module does:

- it never ends on an error; it reports the error in `error` (own short text, never raw output
  of a tool) and tries again after a pause (5 s, doubled up to 60 s), so that tools are not
  called in a loop and the log does not fill up
- it never puts secrets into its section
- it limits frequent changes itself (for `uplink`: signal changes at most every 5 seconds)

A new field or section is added to `api/openapi.yaml` first, then to `src/core/status.rs`.
