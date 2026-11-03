#!/usr/bin/env bash
# V3, openssl enc variant. Usage: openssl.sh encrypt|decrypt INPUT OUTPUT
# The password comes from the environment variable BACKUP_PASSWORD (PBKDF2, default iterations).
set -euo pipefail
case $1 in
  encrypt) openssl enc -aes-256-cbc -pbkdf2 -salt -in "$2" -out "$3" -pass env:BACKUP_PASSWORD ;;
  decrypt) openssl enc -d -aes-256-cbc -pbkdf2 -in "$2" -out "$3" -pass env:BACKUP_PASSWORD ;;
esac
