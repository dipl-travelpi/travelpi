#!/usr/bin/env bash
# Runs V1 in a Debian Trixie container on a development computer and copies the results to
# results-dev/. The container has no Raspberry Pi hardware, so this only measures runtime and
# memory of the variants and their output format; the detected features need the Raspberry Pi.
#   bash dev-container/run-dev-v1.sh
set -euo pipefail
trap 'podman ps -q --filter ancestor=localhost/travelpi-setup-feasibility | xargs -r podman stop > /dev/null' EXIT
cd "$(dirname "$0")/.."
out=results-dev
mkdir -p "$out"

id=$(podman run -d --rm --systemd=always localhost/travelpi-setup-feasibility)
podman cp . "$id:/root/setup-feasibility" > /dev/null
podman exec "$id" rm -rf /root/setup-feasibility/results /root/setup-feasibility/results-dev
podman exec "$id" systemctl is-system-running --wait > /dev/null || true
podman exec "$id" bash -c 'apt-get update -qq && mkdir -p /root/setup-feasibility/results'
# Disk space of ansible-core with its dependencies, as apt reports it before installing
podman exec "$id" bash -c 'apt-get install --assume-no ansible-core 2>&1 | grep -i "disk space" > /root/setup-feasibility/results/ansible-core-size.txt || true'
podman exec "$id" bash -c 'DEBIAN_FRONTEND=noninteractive apt-get install -y -qq python3-psutil python3-pyudev ansible-core > /dev/null'
podman exec "$id" bash -c 'cd /root/setup-feasibility && bash v1-hardware/run-v1.sh || true'
podman cp "$id:/root/setup-feasibility/results/." "$out/"
podman stop "$id" > /dev/null
echo "results in $out/"
