# Results (measured 2026-10-20)

## Setup

| | |
|---|---|
| Server | Raspberry Pi 3 Model B Rev 1.2 (4 cores, 1.2 GHz, 905 MB RAM) |
| OS | Raspberry Pi OS based on Debian 13 (trixie), 64-bit, kernel 6.18 |
| Node.js | Pi: 24.22.0 LTS · computer: 25.9.0 |
| Power | USB charger 5 V / 2.4 A |
| Network | Pi connected to the router by cable (100 Mbit/s); computer (5 GHz, 802.11ac) and phone on the home Wi-Fi. Ping computer → Pi: 14 ms on average, max. 76 ms |
| Computer | MacBook (Apple M4), Google Chrome |
| Phone | iPhone 15, Safari, iOS 18.7 |
| Code | V1 and V2: commit `18b8068` |

The Pi was not throttled during any measurement: CPU clock 1.2 GHz and no new under-voltage
event in every check (`pi-checks.log`). The kernel log since boot (`pi-voltage-since-boot.log`)
contains a single under-voltage event during start-up, before the measurements.
`throttled=0x50000` in the log only reports this past event; the current-state bits are 0.

## Files

| File | Test |
|---|---|
| `v1-chrome-<mode>.json` | V1, Chrome with mobile throttling |
| `v1-device-<mode>.jsonl` | V1, iPhone (one line per run) |
| `v1-load-<mode>.json` | V1, 5, 10 and 30 simulated devices |
| `v2-<method>.json` | V2, 1, 5 and 10 clients |
| `pi-checks.log` | temperature, clock and under-voltage before/after every series |
| `pi-voltage-since-boot.log` | all voltage messages of the kernel since boot |

Run 1 of every V1 series is a warm-up; the median of runs 2–16 is evaluated.
