#!/usr/bin/env bash
# Run on a freshly flashed system before installing anything (also before apt-get update).
# Records which tools and Python modules the variants need are already on Raspberry Pi OS.
cd "$(dirname "$0")"
mkdir -p results

{
  for command in python3 iw udevadm openssl tar age ansible ansible-vault; do
    if command -v "$command" >/dev/null; then echo "$command: yes"; else echo "$command: no"; fi
  done
  for module in psutil pyudev cryptography; do
    if python3 -c "import $module" 2>/dev/null; then echo "python3-$module: yes"; else echo "python3-$module: no"; fi
  done
  echo
  apt-cache policy python3-psutil python3-pyudev python3-cryptography age ansible-core
} | tee results/preinstalled.txt
