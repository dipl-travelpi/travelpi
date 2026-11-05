#!/usr/bin/env bash
# Runs the functional parts of V2 and V3 in Debian Trixie containers on a development computer
# and copies the results to results-dev/. Times measured here are NOT Raspberry Pi times.
#   podman build -t travelpi-setup-feasibility dev-container
#   bash dev-container/run-dev.sh [first runs per variant, default 3; 0 skips the first runs]
set -euo pipefail
cd "$(dirname "$0")/.."
runs=${1:-3}
out=results-dev
mkdir -p "$out"

# Stop containers that are still running if a step fails (set -e ends the script early)
trap 'podman ps -q --filter ancestor=localhost/travelpi-setup-feasibility | xargs -r podman stop > /dev/null' EXIT

start_container() {
  # Rootless Podman runs systemd without any access to the host. Never use Docker with
  # --privileged/--cgroupns=host here: systemd in such a container takes over the host's
  # terminals, kernel settings and cgroups and can end the desktop session.
  podman run -d --rm --systemd=always localhost/travelpi-setup-feasibility    # prints the ID
}

prepare() {    # copy the tests into a running container and wait for systemd
  local id=$1
  podman cp . "$id:/root/setup-feasibility" > /dev/null
  podman exec "$id" rm -rf /root/setup-feasibility/results /root/setup-feasibility/results-dev
  podman exec "$id" systemctl is-system-running --wait > /dev/null || true
}

collect() {
  podman cp "$1:/root/setup-feasibility/results/." "$out/"
  podman stop "$1" > /dev/null
}

# V2 first runs: a fresh container stands for a freshly flashed system
for variant in python shell ansible; do
  for _ in $(seq "$runs"); do
    id=$(start_container); prepare "$id"
    podman exec "$id" bash /root/setup-feasibility/v2-setup/run-v2.sh first "$variant"
    collect "$id"
  done
done

# V2 repeat and error test, V3
id=$(start_container); prepare "$id"
podman exec "$id" bash -c 'apt-get update -qq && DEBIAN_FRONTEND=noninteractive apt-get install -y -qq age ansible-core > /dev/null'
podman exec "$id" bash /root/setup-feasibility/env-info.sh
for variant in python shell ansible; do
  podman exec "$id" bash /root/setup-feasibility/v2-setup/run-v2.sh repeat "$variant"
  podman exec "$id" bash /root/setup-feasibility/v2-setup/run-v2.sh error "$variant"
done
podman exec "$id" bash /root/setup-feasibility/v3-backup/run-v3.sh
podman exec "$id" python3 /root/setup-feasibility/summarize.py > /dev/null
collect "$id"
echo "results in $out/"
