#!/usr/bin/env bash
# Check that a device runs the TravelPi base image (see setup/README.md).
# Only reads the system, changes nothing, needs no root.
# Exit code: 0 if there is no FAIL, 1 otherwise.

set -u

EXPECTED_IMAGE="Raspberry Pi reference 2026-10-06"
EXPECTED_COUNTRY="AT"
EXPECTED_TIMEZONE="Europe/Vienna"

failures=0

ok()   { printf 'OK    %s\n' "$1"; }
warn() { printf 'WARN  %s\n' "$1"; }
fail() { printf 'FAIL  %s\n' "$1"; failures=$((failures + 1)); }

# Prints the kernel driver of a network interface, or nothing.
driver_of() {
    local link
    link=$(readlink -f "/sys/class/net/$1/device/driver" 2>/dev/null) || return 0
    [ -n "$link" ] && basename "$link"
}

# Operating system: Debian 13 "Trixie"
codename=$(sed -n 's/^VERSION_CODENAME=//p' /etc/os-release 2>/dev/null)
if [ "$codename" = "trixie" ]; then
    ok "operating system is Debian trixie"
else
    fail "operating system is '${codename:-unknown}', expected trixie"
fi

# Image release and Lite variant, from the file pi-gen writes to the boot partition
issue=$(cat /boot/firmware/issue.txt 2>/dev/null || true)
if printf '%s' "$issue" | grep -q "^$EXPECTED_IMAGE"; then
    ok "image is '$EXPECTED_IMAGE'"
else
    fail "image is '$(printf '%s' "$issue" | head -n 1)', expected '$EXPECTED_IMAGE'"
fi
if printf '%s' "$issue" | grep -q 'stage2$'; then
    ok "image is the Lite variant (no desktop)"
else
    fail "image is not the Lite variant"
fi

# Architecture: 64-bit
arch=$(uname -m)
if [ "$arch" = "aarch64" ]; then
    ok "architecture is aarch64"
else
    fail "architecture is $arch, expected aarch64"
fi

# Board: Raspberry Pi 3 Model B (the 3 B+ reports "Model B Plus")
model=$(tr -d '\0' < /proc/device-tree/model 2>/dev/null || true)
case "$model" in
    "Raspberry Pi 3 Model B Rev "*) ok "board is $model" ;;
    *) fail "board is '${model:-unknown}', expected Raspberry Pi 3 Model B" ;;
esac

# First boot finished
if cloud-init status 2>/dev/null | grep -q '^status: done'; then
    ok "cloud-init has finished"
else
    fail "cloud-init has not finished: $(cloud-init status 2>&1 | head -n 1)"
fi

# Wi-Fi adapters: onboard chip (brcmfmac) and a second, USB adapter
onboard=""
usb=""
for path in /sys/class/net/wl*; do
    [ -e "$path" ] || continue
    name=$(basename "$path")
    drv=$(driver_of "$name")
    if [ "$drv" = "brcmfmac" ]; then
        onboard="$name"
    elif [ -n "$drv" ]; then
        usb="$name ($drv)"
    fi
done
if [ -n "$onboard" ]; then
    ok "onboard Wi-Fi found: $onboard"
else
    fail "onboard Wi-Fi (brcmfmac) not found"
fi
if [ -n "$usb" ]; then
    ok "USB Wi-Fi adapter found: $usb"
else
    fail "USB Wi-Fi adapter not found; it is needed for the upstream connection"
fi

# Wi-Fi country, needed for the access point
iw_cmd=$(command -v iw || ls /usr/sbin/iw 2>/dev/null || true)
if [ -z "$iw_cmd" ]; then
    fail "the tool iw is missing"
elif "$iw_cmd" reg get 2>/dev/null | grep -q "country $EXPECTED_COUNTRY:"; then
    ok "Wi-Fi country is $EXPECTED_COUNTRY"
else
    fail "Wi-Fi country is not $EXPECTED_COUNTRY"
fi

# Time zone
timezone=$(timedatectl show --property=Timezone --value 2>/dev/null || true)
if [ "$timezone" = "$EXPECTED_TIMEZONE" ]; then
    ok "time zone is $EXPECTED_TIMEZONE"
else
    warn "time zone is '${timezone:-unknown}', expected $EXPECTED_TIMEZONE"
fi

# SSH password login: sshd uses the first value it reads; the drop-in files are included
# at the top of sshd_config, so they are read first.
password_auth=$(cat /etc/ssh/sshd_config.d/*.conf /etc/ssh/sshd_config 2>/dev/null \
    | grep -i -m 1 '^[[:space:]]*PasswordAuthentication' | awk '{print tolower($2)}')
if [ "$password_auth" = "no" ]; then
    ok "SSH password login is off"
else
    warn "SSH password login is on; use public-key authentication only"
fi

if [ "$failures" -eq 0 ]; then
    printf '\nBase image check passed.\n'
    exit 0
fi
printf '\nBase image check failed: %d problem(s).\n' "$failures"
exit 1
