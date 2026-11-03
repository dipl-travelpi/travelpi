#!/usr/bin/env bash
# Appends the test environment (model, OS, versions, throttling) to results/env.txt.
# Run it before and after the tests.
cd "$(dirname "$0")"
mkdir -p results

{
  echo "--- $(date -Iseconds)"
  echo "model: $(tr -d '\0' < /sys/firmware/devicetree/base/model)"
  grep -E '^Revision' /proc/cpuinfo
  grep -E '^(PRETTY_NAME|VERSION_CODENAME)=' /etc/os-release
  [[ -f /etc/rpi-issue ]] && head -n 1 /etc/rpi-issue
  echo "kernel: $(uname -r) ($(uname -m))"
  grep -E '^MemTotal' /proc/meminfo
  echo "python: $(python3 -V 2>&1)"
  echo "cryptography: $(python3 -c 'import cryptography; print(cryptography.__version__)' 2>/dev/null || echo missing)"
  echo "openssl: $(openssl version 2>&1)"
  echo "age: $(age --version 2>/dev/null || echo missing)"
  echo "ansible: $(ansible --version 2>/dev/null | head -n 1 || echo missing)"
  command -v vcgencmd >/dev/null && echo "$(vcgencmd get_throttled)"
} | tee -a results/env.txt
