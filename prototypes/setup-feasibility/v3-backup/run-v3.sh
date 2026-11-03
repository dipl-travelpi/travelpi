#!/usr/bin/env bash
# V3: encrypts and decrypts the test backup with every variant (16 runs each) and checks how
# each variant reacts to a modified file, a wrong password and a missing terminal.
# Run as the normal user (not with sudo).
set -uo pipefail
cd "$(dirname "$0")/.."
mkdir -p results
export BACKUP_PASSWORD="travelpi-test-password"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

python3 v3-backup/make-testdata.py "$work/backup.json" | tee results/v3-testdata.txt

# How a variant is started for the timed runs, and how it is started without help
declare -A timed=(
  [fernet]="python3 v3-backup/fernet.py"
  [openssl]="bash v3-backup/openssl.sh"
  [age]="python3 v3-backup/pty-run.py bash v3-backup/age.sh"
  [vault]="bash v3-backup/vault.sh"
)
declare -A plain=(
  [fernet]="python3 v3-backup/fernet.py"
  [openssl]="bash v3-backup/openssl.sh"
  [age]="bash v3-backup/age.sh"
  [vault]="bash v3-backup/vault.sh"
)

describe() {
  if [[ ! -s $1 ]]; then echo "no output"
  elif cmp -s "$1" "$work/backup.json"; then echo "output identical to the original"
  else echo "output written ($(stat -c %s "$1") bytes), differs from the original"
  fi
}

# Start time of pty-run.py alone, to subtract it from the age times
python3 measure.py --runs 16 --out results/v3-pty-baseline.csv -- python3 v3-backup/pty-run.py true

: > results/v3-checks.txt
for variant in fernet openssl age vault; do
  read -ra cmd <<< "${timed[$variant]}"
  read -ra direct <<< "${plain[$variant]}"
  enc=$work/$variant.enc

  python3 measure.py --runs 16 --out "results/v3-$variant-encrypt.csv" -- "${cmd[@]}" encrypt "$work/backup.json" "$enc"
  python3 measure.py --runs 16 --out "results/v3-$variant-decrypt.csv" -- "${cmd[@]}" decrypt "$enc" "$work/$variant.dec"

  {
    echo "== $variant"
    echo "round trip: $(describe "$work/$variant.dec")"

    python3 v3-backup/tamper.py "$enc" "$work/$variant.tampered"
    rm -f "$work/out"
    "${cmd[@]}" decrypt "$work/$variant.tampered" "$work/out" > /dev/null 2>&1
    echo "one byte modified: exit code $?, $(describe "$work/out")"

    rm -f "$work/out"
    BACKUP_PASSWORD=wrong-password "${cmd[@]}" decrypt "$enc" "$work/out" > /dev/null 2>&1
    echo "wrong password: exit code $?, $(describe "$work/out")"

    # Started like a service or a web backend: no terminal, no input
    rm -f "$work/out"
    timeout 30 setsid -w "${direct[@]}" encrypt "$work/backup.json" "$work/out" < /dev/null > /dev/null 2>&1
    code=$?
    if [[ -s $work/out ]]; then result="encrypted file written"; else result="no output"; fi
    echo "without terminal: exit code $code, $result"
  } | tee -a results/v3-checks.txt
done
