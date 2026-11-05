# Debian Trixie with systemd, the base of Raspberry Pi OS (Trixie). Used to run the functional
# parts of V2 and V3 on a development computer before the Raspberry Pi is available.
# Like Raspberry Pi OS Lite, it has python3, python3-cryptography, openssl and iw preinstalled.
FROM debian:trixie
RUN apt-get update \
 && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
      systemd systemd-sysv dbus python3 python3-cryptography openssl iw procps diffutils \
 && rm -rf /var/lib/apt/lists/*
# Units that would touch the host if the container ever had access to it (terminals, kernel
# settings, devices). The tests do not need them.
RUN systemctl mask getty@.service serial-getty@.service console-getty.service getty.target \
      systemd-sysctl.service systemd-udevd.service systemd-udev-trigger.service \
      systemd-vconsole-setup.service systemd-modules-load.service
# Run with rootless Podman (see run-dev.sh), which sets up cgroups, /run and /tmp for systemd
CMD ["/sbin/init"]
