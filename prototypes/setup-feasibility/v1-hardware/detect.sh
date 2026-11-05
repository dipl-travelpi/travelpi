#!/usr/bin/env bash
# V1, shell variant: detects the hardware and prints it as key=value lines.
# With --watch it keeps running afterwards and reports Wi-Fi sticks that are plugged in later.

# Fields of the new-style revision code (Raspberry Pi documentation, "Raspberry Pi revision codes")
revision=$(awk -F': ' '/^Revision/ {print $2}' /proc/cpuinfo)
code=$((16#${revision:-0}))    # no revision code: not a Raspberry Pi (e.g. the development container)
if [[ -z $revision ]]; then
  model=unknown
elif (( code & (1 << 23) )); then
  case $(( (code >> 4) & 0xff )) in
    8) model=3B ;; 9) model=Zero ;; 12) model="Zero W" ;; 13) model=3B+ ;; 14) model=3A+ ;;
    17) model=4B ;; 18) model="Zero 2 W" ;; 19) model=400 ;; 23) model=5 ;; 25) model=500 ;;
    *) model=unknown ;;
  esac
  memory_mb=$(( 256 << ((code >> 20) & 7) ))
else
  model="old-style revision code"
fi

echo "revision=$revision"
echo "model=$model"
echo "memory_mb=${memory_mb:-}"
echo "memory_total_kb=$(awk '/^MemTotal/ {print $2}' /proc/meminfo)"
echo "cpu_cores=$(nproc)"

for interface in $(iw dev | awk '/Interface/ {print $2}'); do
  device=$(readlink -f "/sys/class/net/$interface/device")
  if [[ $device == */usb* ]]; then
    stick=$(dirname "$device")
    echo "wifi_usb=$interface $(cat "$stick/idVendor"):$(cat "$stick/idProduct") $(cat "$stick/product" 2>/dev/null)"
  else
    echo "wifi_internal=$interface"
  fi
done

for adapter in /sys/class/bluetooth/hci*; do
  [[ -e $adapter && $(basename "$adapter") != *:* ]] && echo "bluetooth=$(basename "$adapter")"
done

if [[ ${1:-} == --watch ]]; then
  udevadm monitor --udev --subsystem-match=net --property | awk -F= '
    /^ACTION=/ {action = $2}
    /^DEVTYPE=/ {type = $2}
    /^INTERFACE=/ {name = $2}
    /^$/ {
      if (action == "add" && type == "wlan") { print "wifi_added=" name; fflush() }
      action = type = name = ""
    }'
fi
