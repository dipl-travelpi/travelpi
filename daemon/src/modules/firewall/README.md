# Firewall

All nftables rules of TravelPi live here. This first version only does routing and NAT: devices
in the own Wi-Fi reach the internet through the upstream Wi-Fi. There is no tunnel and no kill
switch yet; those follow in later issues.

## Files

| File | Goes to | What it does |
|---|---|---|
| `travelpi.nft` | `/etc/nftables.conf` | Forwards new connections from the own Wi-Fi (`wlan0`, `10.157.1.0/24`) to the uplink (`wlan1`) and lets the answers back. Hides the devices behind the address of the Pi (masquerade). Drops everything else that would be forwarded, including all IPv6. |
| `travelpi-forward.conf` | `/etc/sysctl.d/` | Turns on IPv4 forwarding in the kernel. Without it, the Pi does not pass packets between interfaces at all. |

Interface names and the address range are variables at the top of `travelpi.nft`. The names are
the ones found on the development Pi (onboard chip `wlan0`, USB adapter `wlan1`); check them with
`ip link` before relying on them.

The ruleset only touches its own table `inet travelpi`. Loading the file again replaces that
table in one step and leaves rules of other programs alone.

Not covered yet: rules for traffic to the Pi itself (input chain), the tunnel and the kill
switch.

## Check on a development computer

No root needed: the ruleset is loaded in a new, empty network namespace.

```sh
unshare -rn sh -c 'nft -c -f travelpi.nft && nft -f travelpi.nft && nft list ruleset'
```

`nft -c` only checks the file; `nft -f` really loads it, and `nft list ruleset` shows the result.

## Install on the Pi

The package `nftables` must be installed. On the current Raspberry Pi OS Lite image it is already
there, but `nftables.service` is disabled, so no rules are loaded at boot until it is enabled.

```sh
sudo apt install nftables
sudo cp travelpi.nft /etc/nftables.conf
sudo cp travelpi-forward.conf /etc/sysctl.d/
sudo sysctl --system
sudo systemctl enable --now nftables.service
```

`nftables.service` loads `/etc/nftables.conf` at every boot, so the rules are active before any
traffic is forwarded. Copying the file replaces the default `/etc/nftables.conf` of the system.

## Test on the Pi

1. `sysctl net.ipv4.ip_forward` shows `1`.
2. `sudo nft list ruleset` shows the table `inet travelpi` with the rules above.
3. The Pi is connected to an upstream Wi-Fi on `wlan1`, and the own Wi-Fi runs on `wlan0`.
4. A phone joins the own Wi-Fi, gets an address from `10.157.1.0/24` and can open a website.
5. After `sudo reboot`, steps 1, 2 and 4 still work without doing anything by hand.
6. An IPv6 test page on the phone shows no working IPv6 through the Pi.
