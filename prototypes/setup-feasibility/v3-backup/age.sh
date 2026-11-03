#!/usr/bin/env bash
# V3, age variant. Usage: age.sh encrypt|decrypt INPUT OUTPUT
# In passphrase mode age asks for the passphrase on the terminal. For the timed runs,
# run-v3.sh starts this script through pty-run.py, which types BACKUP_PASSWORD.
set -euo pipefail
case $1 in
  encrypt) age --passphrase --output "$3" "$2" ;;
  decrypt) age --decrypt --output "$3" "$2" ;;
esac
