#!/usr/bin/env bash
# Prints everything the setup touches. Comparing two snapshots shows whether a run changed
# the system (package versions, file content and modification time, service restarts).
for package in hostapd dnsmasq wireguard-tools; do
  dpkg-query -W -f='${Package} ${Version} ${Status}\n' "$package" 2>/dev/null || echo "$package missing"
done
for file in /etc/travelpi/travelpi.conf /etc/systemd/system/travelpi-test.service; do
  if [[ -f $file ]]; then
    echo "$file modified=$(stat -c %Y "$file") sha256=$(sha256sum < "$file" | cut -c1-16)"
  else
    echo "$file missing"
  fi
done
echo "service enabled: $(systemctl is-enabled travelpi-test.service 2>/dev/null)"
echo "service started at: $(systemctl show -p ActiveEnterTimestampMonotonic --value travelpi-test.service)"
