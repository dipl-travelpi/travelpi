#!/usr/bin/env python3
"""V1, Python variant: detects the hardware and prints it as JSON.

With --watch it keeps running afterwards and reports Wi-Fi sticks that are plugged in later.
"""
import json
import sys

import psutil
import pyudev

# Fields of the new-style revision code (Raspberry Pi documentation, "Raspberry Pi revision codes")
BOARD_TYPES = {0x08: "3B", 0x09: "Zero", 0x0C: "Zero W", 0x0D: "3B+", 0x0E: "3A+", 0x11: "4B",
               0x12: "Zero 2 W", 0x13: "400", 0x17: "5", 0x19: "500"}
MEMORY_MB = {0: 256, 1: 512, 2: 1024, 3: 2048, 4: 4096, 5: 8192, 6: 16384}


def board():
    with open("/proc/cpuinfo") as f:
        revision = next(int(line.split(":")[1], 16) for line in f if line.startswith("Revision"))
    if not revision & (1 << 23):
        return {"revision": f"{revision:x}", "model": "old-style revision code"}
    return {
        "revision": f"{revision:x}",
        "model": BOARD_TYPES.get((revision >> 4) & 0xFF, "unknown"),
        "memory_mb": MEMORY_MB.get((revision >> 20) & 0x7),
    }


def attribute(device, name):
    try:
        return device.attributes.asstring(name)
    except KeyError:
        return None


def wifi(context):
    internal, usb = [], []
    for device in context.list_devices(subsystem="net", DEVTYPE="wlan"):
        stick = device.find_parent("usb", "usb_device")
        if stick is None:
            internal.append(device.sys_name)
        else:
            usb.append({
                "interface": device.sys_name,
                "id": f"{attribute(stick, 'idVendor')}:{attribute(stick, 'idProduct')}",
                "product": attribute(stick, "product"),
            })
    return internal, usb


def main():
    context = pyudev.Context()
    internal, usb = wifi(context)
    result = {
        **board(),
        "memory_total_mb": round(psutil.virtual_memory().total / 2**20),
        "cpu_cores": psutil.cpu_count(),
        "wifi_internal": internal,
        "wifi_usb": usb,
        "bluetooth": [d.sys_name for d in context.list_devices(subsystem="bluetooth") if ":" not in d.sys_name],
    }
    print(json.dumps(result, indent=2), flush=True)

    if "--watch" in sys.argv:
        monitor = pyudev.Monitor.from_netlink(context)
        monitor.filter_by("net")
        for device in iter(monitor.poll, None):
            if device.action == "add" and device.get("DEVTYPE") == "wlan":
                print(json.dumps({"event": "wifi added", "interface": device.sys_name}), flush=True)


if __name__ == "__main__":
    main()
