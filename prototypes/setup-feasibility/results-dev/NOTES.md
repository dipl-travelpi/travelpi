# Notes on the development computer results

Measured on 2026-11-03 (V2 first runs) and 2026-11-05 (rest) in Debian Trixie containers
(rootless Podman 6.1.4) on an Intel Core i5-10210U with 16 GB RAM. Not Raspberry Pi results.

Values in these files that are **not** meaningful here:

- **V1 "Detected features"** (`v1-check.txt`): the container has no Raspberry Pi hardware, and
  `reference.json` describes the Raspberry Pi 3 Model B. Only runtime, memory and the output
  format of V1 count. One thing does show: Bluetooth (`hci0`) is visible in the container and
  found by Python and shell, but the Ansible facts contain no Bluetooth entry at all.
- **`disk_mb`** in `v2-ansible-install-*.txt`: in a rootless container `df /` reports the disk of
  the host, so the value changes with everything else on the computer. The size of Ansible is
  taken from apt instead (`ansible-core-size.txt`): 325 MB with the recommended packages that apt
  installs by default (among them the full `ansible` collection), 49.8 MB with
  `--no-install-recommends` (checked separately).
- **Preinstalled tools**: not checked here, a container is not a fresh Raspberry Pi OS. Taken
  from the package list of Raspberry Pi OS Lite 2026-10-06 instead (python3-cryptography,
  openssl, iw, udev and bluez preinstalled; python3-psutil, python3-pyudev, age and
  ansible-core not).

`discarded/` holds one V2 first run (Ansible) during which the computer went to sleep.
`logs/` holds the console output; the first log ends with exit code 2 because of a bug in the
error test of `run-v2.sh` (fixed), so the repeat test, error test and V3 were run again.
