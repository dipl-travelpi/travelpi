#!/usr/bin/env bash
# Check that a device runs the TravelPi base image (see setup/README.md).
# Only reads the system, changes nothing, needs no root.
# Every line starts with OK, WARN or FAIL. Exit code: 0 if there is no FAIL, 1 otherwise.

set -u

# Tools such as iw live in /usr/sbin, which is not in the PATH of a normal SSH session.
PATH="$PATH:/usr/sbin:/sbin"

EXPECTED_IMAGE="Raspberry Pi reference 2026-10-06"
EXPECTED_COUNTRY="AT"
EXPECTED_TIMEZONE="Europe/Vienna"

failures=0
warnings=0

ok()   { printf 'OK    %s\n' "$1"; }
warn() { printf 'WARN  %s\n' "$1"; warnings=$((warnings + 1)); }
fail() { printf 'FAIL  %s\n' "$1"; failures=$((failures + 1)); }

# Prints the kernel driver of a network interface, or nothing if it has none.
driver_of() {
    local driver="/sys/class/net/$1/device/driver"
    if [ -e "$driver" ]; then
        basename "$(readlink -f "$driver")"
    fi
}

# --- Image ---------------------------------------------------------------------------------

codename=$(sed -n 's/^VERSION_CODENAME=//p' /etc/os-release 2>/dev/null | tr -d '"')
if [ "$codename" = "trixie" ]; then
    ok "operating system is Debian trixie"
else
    fail "operating system is '${codename:-unknown}', expected trixie"
fi

# pi-gen writes the release and the build stage to the boot partition; stage2 is Lite.
issue=$(cat /boot/firmware/issue.txt 2>/dev/null || true)
release=$(printf '%s\n' "$issue" | head -n 1)
if [ "$release" = "$EXPECTED_IMAGE" ]; then
    ok "image is '$EXPECTED_IMAGE'"
else
    fail "image is '${release:-unknown}', expected '$EXPECTED_IMAGE'"
fi
if printf '%s\n' "$issue" | grep -q 'stage2$'; then
    ok "image is the Lite variant (no desktop)"
else
    fail "image is not the Lite variant"
fi

arch=$(uname -m)
if [ "$arch" = "aarch64" ]; then
    ok "architecture is aarch64 (64-bit)"
else
    fail "architecture is $arch, expected aarch64"
fi

if command -v cloud-init > /dev/null; then
    ci_status=$(cloud-init status 2>&1)
    ci_code=$?
else
    ci_status="cloud-init is not installed"
    ci_code=1
fi
case "$ci_code:$ci_status" in
    0:*"status: done"*) ok "first boot (cloud-init) has finished" ;;
    2:*"status: done"*) warn "first boot (cloud-init) finished with errors; see 'cloud-init status --long'" ;;
    *"status: running"*) fail "first boot (cloud-init) is still running; wait with 'cloud-init status --wait'" ;;
    *) fail "first boot (cloud-init) did not finish: $(printf '%s\n' "$ci_status" | head -n 1)" ;;
esac

# --- Hardware ------------------------------------------------------------------------------

# The 3 B+ reports "Model B Plus" and is a different board.
model=$(tr -d '\0' 2>/dev/null < /proc/device-tree/model || true)
case "$model" in
    "Raspberry Pi 3 Model B Rev "*) ok "board is $model" ;;
    *) fail "board is '${model:-unknown}', expected Raspberry Pi 3 Model B" ;;
esac

# Power: bit 0 = under-voltage now, bit 16 = under-voltage since boot.
if throttled=$(vcgencmd get_throttled 2>/dev/null); then
    value=$((${throttled#throttled=}))
    if (( value & 0x1 )); then
        fail "under-voltage right now ($throttled); use a 5.1 V / 2.5 A power supply"
    elif (( value & 0x10000 )); then
        warn "under-voltage since boot ($throttled); use a 5.1 V / 2.5 A power supply"
    else
        ok "power supply: no under-voltage since boot"
    fi
else
    warn "could not read the power state (vcgencmd get_throttled)"
fi

# Onboard Wi-Fi (driver brcmfmac) becomes the access point; a USB adapter connects upstream.
# An adapter counts as USB when its device sits on the USB bus.
onboard=""
usb=""
for path in /sys/class/net/wl*; do
    [ -e "$path" ] || continue
    name=${path##*/}
    driver=$(driver_of "$name")
    if [ "$driver" = "brcmfmac" ]; then
        onboard="$name"
    elif [ -n "$driver" ]; then
        case "$(readlink -f "$path/device")" in
            */usb*) usb="$name ($driver)" ;;
        esac
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

# --- Settings ------------------------------------------------------------------------------

if ! command -v iw > /dev/null; then
    fail "the tool iw is missing"
elif iw reg get 2>/dev/null | grep -q "country $EXPECTED_COUNTRY:"; then
    ok "Wi-Fi country is $EXPECTED_COUNTRY"
else
    fail "Wi-Fi country is not $EXPECTED_COUNTRY; set it with 'sudo raspi-config nonint do_wifi_country $EXPECTED_COUNTRY'"
fi

timezone=$(timedatectl show --property=Timezone --value 2>/dev/null || true)
if [ "$timezone" = "$EXPECTED_TIMEZONE" ]; then
    ok "time zone is $EXPECTED_TIMEZONE"
else
    warn "time zone is '${timezone:-unknown}', expected $EXPECTED_TIMEZONE"
fi

# sshd uses the first value it reads. The drop-in files are included at the top of
# sshd_config, so they come first. Without root some of them may not be readable.
unreadable=""
for file in /etc/ssh/sshd_config.d/*.conf /etc/ssh/sshd_config; do
    [ -e "$file" ] && [ ! -r "$file" ] && unreadable="$file"
done
if [ -n "$unreadable" ]; then
    warn "cannot read $unreadable; check SSH password login with 'sudo sshd -T | grep -i passwordauthentication'"
else
    password_auth=$(cat /etc/ssh/sshd_config.d/*.conf /etc/ssh/sshd_config 2>/dev/null \
        | awk 'tolower($1) == "passwordauthentication" { print tolower($2); exit }')
    if [ "$password_auth" = "no" ]; then
        ok "SSH password login is off"
    else
        warn "SSH password login is on; use public-key authentication only"
    fi
fi

# --- Result --------------------------------------------------------------------------------

printf '\n'
if [ "$failures" -eq 0 ]; then
    printf 'Base image check passed (%d warning(s)).\n' "$warnings"
    exit 0
fi
printf 'Base image check failed: %d problem(s), %d warning(s).\n' "$failures" "$warnings"
exit 1
