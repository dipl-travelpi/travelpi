# Setup feasibility tests

Tests for the technical feasibility study of the TravelPi setup (diploma thesis, TGM Wien).
They run on a Raspberry Pi 3 Model B and compare tools for three tasks:

| Test | Compares | Measures | Criteria |
|---|---|---|---|
| **V1** Hardware detection | Python (psutil, pyudev), shell, Ansible facts | correctly detected features (6), runtime, peak memory, structured output, Wi-Fi stick plugged in while running | K1, K4, K5 |
| **V2** Setup | Python, shell, Ansible | first run on a fresh system, repeated run, changes made by a second run, reaction to a wrong package name, lines of code | K2, K4, K5, K6 |
| **V3** Backup | Fernet (Python), openssl enc, age, Ansible Vault | encrypt and decrypt time, modified byte detected, wrong password, works without a terminal, preinstalled | Muss, K1, K4, K5 |

Criteria (from the thesis): K1 task fulfilled, K2 error handling and repeatability, K4 runtime
and resources, K5 runs on different models / preinstalled, K6 effort. The six features of V1 are
model, memory, CPU cores, internal Wi-Fi, Bluetooth and a USB Wi-Fi stick.

## What is in this folder

| Path | What it is |
|---|---|
| `measure.py` | Runs a command several times, writes time and peak memory of every run to a CSV file |
| `env-info.sh` | Writes model, OS, versions and throttling to `results/env.txt` |
| `check-preinstalled.sh` | Checks which tools are already on a fresh Raspberry Pi OS |
| `v1-hardware/` | `detect.py`, `detect.sh` (the variants), `run-v1.sh`, `check-v1.py`, `reference.json` |
| `v2-setup/` | `setup.py`, `setup.sh`, `setup.yml` (the variants), `run-v2.sh`, `snapshot.sh` |
| `v3-backup/` | `fernet.py`, `openssl.sh`, `age.sh`, `vault.sh` (the variants), `run-v3.sh`, helpers |
| `summarize.py` | Collects all values for the thesis into `results/summary.md` |
| `results/` | Measurements from the Raspberry Pi |

The shortened setup in V2 stands for the real setup script: it installs `hostapd`, `dnsmasq`
and `wireguard-tools`, writes `/etc/travelpi/travelpi.conf` and enables `travelpi-test.service`.
All three variants only change what is not yet in the wanted state, so a second run should
change nothing.

## Order of the tests

V2 first runs need a freshly flashed system for every run, and the check for preinstalled tools
must happen before anything is installed. So the order is:

1. Fresh system: `check-preinstalled.sh`, then V2 first runs (flash again for every run)
2. Last flash: install the rest, then V2 repeat and error test, V1, V3

Copy `results/` to your computer **before every new flash**, otherwise the results are gone.

## 1. Prepare a fresh system

Flash Raspberry Pi OS Lite with Raspberry Pi Imager (enable SSH, set user `pi`). Connect the Pi
with a LAN cable, like in the dashboard tests. Then on the computer:

```bash
scp -r setup-feasibility pi@raspberrypi.local:
```

On the Pi, wait until the first boot is finished (otherwise apt may be locked in the background):

```bash
cloud-init status --wait
systemctl is-system-running --wait
```

Only on the very first flash:

```bash
cd setup-feasibility
bash check-preinstalled.sh
bash env-info.sh
```

## 2. V2 first runs (one run per flash)

```bash
cd setup-feasibility
sudo bash v2-setup/run-v2.sh first python    # or shell, or ansible
```

For `ansible`, the script first installs Ansible itself and records the time and disk space in
`results/v2-ansible-install-*.txt`. Copy `results/` to the computer, flash again, next variant.
Three runs per variant are a good number (nine flashes); one per variant is the minimum.

## 3. Install everything else (last flash, after the Ansible first run)

```bash
sudo apt-get install -y python3-psutil python3-pyudev python3-cryptography age ansible
bash env-info.sh
vcgencmd get_throttled    # should be throttled=0x0
```

## 4. V2 repeat and error test

```bash
for variant in python shell ansible; do
  sudo bash v2-setup/run-v2.sh repeat $variant
  sudo bash v2-setup/run-v2.sh error $variant
done
```

`dnsmasq` starts after it is installed; that does not affect the other tests.

## 5. V1 hardware detection

Fill in `v1-hardware/reference.json` with what is really there (the ID of the stick is shown by
`lsusb` or on the box). Plug in the USB Wi-Fi stick, then:

```bash
bash v1-hardware/run-v1.sh
```

Check `results/v1-check.txt` against the outputs `results/v1-*-output.txt`, especially for Ansible.

**Stick plugged in while running** (by hand): unplug the stick, start a variant, plug the stick
in and wait 10 seconds.

```bash
python3 v1-hardware/detect.py --watch    # prints "wifi added" when the stick is detected
bash v1-hardware/detect.sh --watch       # prints "wifi_added=..."
```

Ansible has no watch mode: it only sees the stick when it is run again. Write the result into
`results/v1-hotplug.txt`, for example:

```
python: yes, detected without a restart
shell: yes, detected without a restart
ansible: no, has to be run again
```

## 6. V3 backup

```bash
bash v3-backup/run-v3.sh
```

The test file has the structure of the backup in the requirements (/LD070/) with made-up values.
age asks for the passphrase on the terminal, so the timed runs go through `pty-run.py`, which
types the password. Its own start time is measured in `results/v3-pty-baseline.csv` and
subtracted in the summary. The check "without terminal" starts every variant like a service
would, without the helper.

## 7. Summary

```bash
bash env-info.sh
python3 summarize.py
```

`results/summary.md` contains all values for the tables in the thesis. Copy `results/` to the
computer and commit it.
