# Uplink

The module `uplink` reports the connection to the internet source: the upstream Wi-Fi that the
USB adapter (`wlan1`) is in. It reads everything from NetworkManager over D-Bus and fills the
section `uplink` of `/v0/status` (fields and states in [`api/openapi.yaml`](../../../../api/openapi.yaml)).

This first version only **reads**. Joining a network is done by a stored NetworkManager
connection profile, created with `nmcli` (see below). Scanning, storing credentials and connecting
from the dashboard follow with the commands of the API (TRA-33, TRA-35).

## Files

| File | Content |
|---|---|
| `mod.rs` | The module task: reads the device, reports the section, limits signal-only changes, retries if NetworkManager is not reachable |
| `state.rs` | Maps what NetworkManager says onto the section; plain functions, no D-Bus |
| `networkmanager.rs` | Reads the device over D-Bus (`zbus`), and the `Source` trait the tests replace |

## How it works

1. Ask NetworkManager for the device with the interface name `wlan1` (`GetDeviceByIpIface`).
   No such device: `state: unavailable`, `interface: null`.
2. Read the device state, the active access point (network name, signal), the IPv4 address
   and the connectivity check.
3. Map the thirteen device states of NetworkManager onto the five of the API:

   | NetworkManager | API |
   |---|---|
   | unknown, unmanaged, unavailable | `unavailable` |
   | disconnected, deactivating | `disconnected` |
   | prepare, config, need-auth, ip-config, ip-check, secondaries | `connecting` |
   | activated | `connected` |
   | failed | `failed` |

4. Wait for the next state change (D-Bus signal `StateChanged`), at most 2 seconds, then read
   again. Signal strength and the connectivity check change without a state change, so they are
   picked up by the 2-second refresh.

Details that are easy to get wrong:

- **Failed only lasts a moment.** NetworkManager goes from `failed` to `disconnected` right
  away and drops the reason. The module therefore takes the reason from the `StateChanged`
  signal, reports `failed` at once and keeps it until NetworkManager starts a new attempt.
- **Own error texts only.** The reason code becomes a short text such as
  `wrong or missing password`, `network not found` or `no address received`, never raw output.
- **Signal limit.** A change of only `signal` is reported at most every 5 seconds; every other
  change is reported at once, with the current signal (agreed in TRA-21 and TRA-22).
- **`internet`** is the IPv4 connectivity check of NetworkManager for this device: `true` only
  with full access, `false` with none, limited or a captive portal, `null` while unknown or not
  connected. It is only meaningful if the check is enabled (see "Test on the Pi").
- **NetworkManager not reachable:** `state: failed`, `error: NetworkManager not reachable`, then
  a new try after 5 s, doubled up to 60 s.

## Check on a development computer

```sh
cargo test uplink
```

The tests replace NetworkManager by a script (`Source` in `networkmanager.rs`) and run with a
paused clock, so they take no real time.

Against the real NetworkManager of a Linux laptop (read-only, no root needed): change `INTERFACE`
in `mod.rs` to the Wi-Fi interface of the laptop (`ip link`), then

```sh
cargo run -- --socket ./travelpid.sock
curl --unix-socket travelpid.sock http://localhost/v0/status
```

Do not commit the changed interface name.

## Test on the Pi

Needs the binary built for the Pi (see `daemon/README.md`) and a connection profile for `wlan1`.

1. Create the profile once (example name `uplink-test`; `--ask` asks for the password, so it
   does not end up in the shell history):

   ```sh
   sudo nmcli --ask device wifi connect "<network name>" ifname wlan1 name uplink-test
   ```

2. Check that the connectivity check is enabled, otherwise `internet` stays meaningless:

   ```sh
   busctl get-property org.freedesktop.NetworkManager /org/freedesktop/NetworkManager \
     org.freedesktop.NetworkManager ConnectivityCheckAvailable ConnectivityCheckEnabled
   nmcli networking connectivity check
   ```

   The first command must print `b true` twice, the second `full` while the Pi has internet.
   With the check disabled, NetworkManager reports full connectivity whenever the device is
   connected (seen on a laptop with NetworkManager 1.5x), so `internet` would be `true` even
   behind a captive portal. Enabling the check is part of the setup.

3. Start `travelpid` and read the status:

   ```sh
   sudo ./travelpid
   sudo curl --unix-socket /run/travelpid/api.sock http://localhost/v0/status
   ```

   Expected: `state: connected`, the network name, a signal, the address of `wlan1`,
   `internet: true`.

4. In a second terminal, follow the events while changing the connection:

   ```sh
   sudo curl -N --unix-socket /run/travelpid/api.sock http://localhost/v0/events
   sudo nmcli con down uplink-test   # disconnected
   sudo nmcli con up uplink-test     # connecting, then connected
   ```

5. Unplug the USB adapter: `unavailable`, `interface: null`. Plug it in again.

Careful on the shared Pi: if SSH runs over `wlan1`, step 4 and 5 cut your own connection. Do
them only with another way to the Pi (for example `wlan0` in the home network, before the access
point is started).
