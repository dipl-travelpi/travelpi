# Setup

Everything that turns a fresh Raspberry Pi OS image into a TravelPi: the first-boot
configuration (cloud-init), the setup script and the systemd units.

## Base image

Every TravelPi, and every development device, starts from this one image, so that everybody
works on the same base. Do not use a newer release until this section is updated.

| | |
|---|---|
| System | Raspberry Pi OS Lite, 64-bit (arm64), based on Debian 13 "Trixie", no desktop |
| Release | 2026-10-06 |
| File | `2026-10-06-raspios-trixie-arm64-lite.img.xz` |
| Download | <https://downloads.raspberrypi.com/raspios_lite_arm64/images/raspios_lite_arm64-2026-10-06/2026-10-06-raspios-trixie-arm64-lite.img.xz> |
| SHA-256 | `483db18a48da399b5b7022ffae9a07bc6d99daab30e593aedac09b69ff422843` |
| Device | Raspberry Pi 3 Model B, with a USB Wi-Fi adapter for the upstream connection |

Raspberry Pi Imager only offers the latest release in its list, so download the file above and
use it as a custom image.

### 1. Download and verify

```bash
curl -LO https://downloads.raspberrypi.com/raspios_lite_arm64/images/raspios_lite_arm64-2026-10-06/2026-10-06-raspios-trixie-arm64-lite.img.xz
echo "483db18a48da399b5b7022ffae9a07bc6d99daab30e593aedac09b69ff422843  2026-10-06-raspios-trixie-arm64-lite.img.xz" | sha256sum -c
```

The second command must print `OK`. If it does not, delete the file and download it again.

### 2. Flash with Raspberry Pi Imager

Choose the device "Raspberry Pi 3", then "Use custom" with the downloaded file, then the SD card.
Apply these settings when Imager offers to customise the image:

| Setting | Value |
|---|---|
| Hostname | `pi` (the name `travelpi` comes with the hostname feature) |
| Time zone | `Europe/Vienna` |
| User | `pi`, with a password of your own |
| SSH | on, public-key authentication only; paste the public keys of everybody who works on this device |
| Wi-Fi | only for development, see below |
| Wi-Fi country | `AT` (needed later for the access point) |

Never write a password or Wi-Fi credentials into a file in this repository.

**Wi-Fi during development.** Until the own Wi-Fi exists, the device can join a known Wi-Fi
with the onboard adapter (`wlan0`) so that it is reachable over SSH. The onboard adapter later
becomes the access point. Before that, connect the USB adapter (`wlan1`) to the known Wi-Fi and
check SSH over it, otherwise access is lost.

### 3. First boot and check

Insert the card, connect the USB Wi-Fi adapter and power the device on. The first boot takes a
few minutes. Then, from the computer:

```bash
ssh pi@<address> 'bash -s' < setup/check-base.sh
```

`check-base.sh` only reads the system and changes nothing. Every line starts with `OK`, `WARN`
or `FAIL`. The device is a valid base when there is no `FAIL`; the exit code is then `0`.
