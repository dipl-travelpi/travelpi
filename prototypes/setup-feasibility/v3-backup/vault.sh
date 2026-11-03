#!/usr/bin/env bash
# V3, Ansible Vault variant. Usage: vault.sh encrypt|decrypt INPUT OUTPUT
# The password comes from the environment variable BACKUP_PASSWORD; ansible-vault reads it
# from a temporary password file.
set -euo pipefail
password_file=$(mktemp)
trap 'rm -f "$password_file"' EXIT
printf '%s' "$BACKUP_PASSWORD" > "$password_file"

case $1 in
  encrypt) ansible-vault encrypt "$2" --output "$3" --vault-password-file "$password_file" ;;
  decrypt) ansible-vault decrypt "$2" --output "$3" --vault-password-file "$password_file" ;;
esac
