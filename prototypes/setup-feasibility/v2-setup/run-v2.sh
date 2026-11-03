#!/usr/bin/env bash
# V2. Run as root:
#   run-v2.sh first  python|shell|ansible   on a freshly flashed system, once per flash
#   run-v2.sh repeat python|shell|ansible   on a system that is already set up (16 runs)
#   run-v2.sh error  python|shell|ansible   with a package name that does not exist
set -uo pipefail
cd "$(dirname "$0")/.."
mode=${1:?first, repeat or error}
variant=${2:?python, shell or ansible}
mkdir -p results

case $variant in
  python) setup=(python3 v2-setup/setup.py) ;;
  shell) setup=(bash v2-setup/setup.sh) ;;
  ansible) setup=(ansible-playbook -i localhost, -c local v2-setup/setup.yml) ;;
  *) echo "unknown variant: $variant" >&2; exit 2 ;;
esac

case $mode in
  first)
    stamp=$(date +%Y%m%d-%H%M%S)
    if [[ $variant == ansible ]]; then
      # Installing Ansible itself is part of the first run, but recorded separately
      used_before=$(df --output=used -B1M / | tail -n 1)
      start=$(date +%s.%N)
      apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y ansible
      end=$(date +%s.%N)
      used_after=$(df --output=used -B1M / | tail -n 1)
      awk -v s="$start" -v e="$end" -v d=$((used_after - used_before)) \
        'BEGIN { printf "seconds=%.1f\ndisk_mb=%d\n", e - s, d }' | tee "results/v2-ansible-install-$stamp.txt"
    fi
    python3 measure.py --runs 1 --out "results/v2-first-$variant-$stamp.csv" \
      --save-output "results/v2-first-$variant-$stamp-output.txt" -- "${setup[@]}"
    ;;

  repeat)
    "${setup[@]}" > /dev/null    # make sure the system is set up
    bash v2-setup/snapshot.sh > "results/v2-$variant-before.txt"
    "${setup[@]}" > "results/v2-$variant-second-run-output.txt"
    bash v2-setup/snapshot.sh > "results/v2-$variant-after.txt"
    diff "results/v2-$variant-before.txt" "results/v2-$variant-after.txt" > "results/v2-$variant-changes.txt"
    echo "changes in the second run: $(grep -c '^>' "results/v2-$variant-changes.txt")"
    python3 measure.py --runs 16 --out "results/v2-repeat-$variant.csv" -- "${setup[@]}"
    ;;

  error)
    # Without the config file we can see whether the setup went on after the failed step
    rm -f /etc/travelpi/travelpi.conf
    if [[ $variant == ansible ]]; then
      setup+=(-e bad_package=true)
    fi
    BAD_PACKAGE=1 "${setup[@]}" > "results/v2-error-$variant-output.txt" 2>&1
    code=$?
    if [[ -f /etc/travelpi/travelpi.conf ]]; then went_on=yes; else went_on=no; fi
    printf 'exit_code=%d\nwent_on_after_error=%s\n' "$code" "$went_on" | tee "results/v2-error-$variant.txt"
    "${setup[@]}" > /dev/null 2>&1    # restore the normal state
    ;;

  *) echo "unknown mode: $mode" >&2; exit 2 ;;
esac
