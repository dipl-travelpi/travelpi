# Summary of the setup feasibility tests

## V1 Hardware detection

| Variant | Runtime (median) | Max. memory (median) | Failed runs |
|---|---|---|---|
| python | 45 ms | 14.7 MB | 0 |
| shell | 6 ms | 12.4 MB | 0 |
| ansible | 978 ms | 52.2 MB | 0 |

Detected features:

```
feature            python     shell   ansible
model                  no        no        no
memory                yes       yes       yes
cpu_cores              no        no        no
wifi_internal          no        no        no
bluetooth             yes       yes        no
wifi_usb               no        no        no
detected              2/6       2/6       1/6
```

Stick plugged in while running (fill in by hand, see README):

```
(missing)
```

## V2 Setup

| Variant | First run (each flash) | Repeat (median) | Changes in 2nd run | Error: exit code / went on | Code lines |
|---|---|---|---|---|---|
| python | 52 s, 42 s, 37 s | 0.1 s | 0 | 1 / no | 41 |
| shell | 37 s, 36 s, 37 s | 0.1 s | 0 | 100 / no | 31 |
| ansible | 46 s, 44 s, 35 s | 4.4 s | 0 | 2 / no | 46 |

Installing Ansible (part of the Ansible first run):

- v2-ansible-install-20261103-195709.txt: seconds=45.7, disk_mb=645
- v2-ansible-install-20261103-195843.txt: seconds=43.3, disk_mb=647
- v2-ansible-install-20261105-131322.txt: seconds=37.6, disk_mb=628

## V3 Backup

test backup: 4959 bytes

| Variant | Encrypt (median) | Decrypt (median) |
|---|---|---|
| fernet | 73 ms | 72 ms |
| openssl | 14 ms | 14 ms |
| age | 617 ms | 618 ms (without pty-run.py: 600 ms / 602 ms) |
| vault | 295 ms | 294 ms |

```
== fernet
round trip: output identical to the original
one byte modified: exit code 1, no output
wrong password: exit code 1, no output
without terminal: exit code 0, encrypted file written
== openssl
round trip: output identical to the original
one byte modified: exit code 0, output written (4959 bytes), differs from the original
wrong password: exit code 1, output written (4944 bytes), differs from the original
without terminal: exit code 0, encrypted file written
== age
round trip: output identical to the original
one byte modified: exit code 1, no output
wrong password: exit code 1, no output
without terminal: exit code 1, no output
== vault
round trip: output identical to the original
one byte modified: exit code 1, no output
wrong password: exit code 1, no output
without terminal: exit code 0, encrypted file written
```

Preinstalled on a fresh system:

```
(missing)
```

Environment:

```
--- 2026-11-05T13:29:17+00:00
model: 
PRETTY_NAME="Debian GNU/Linux 13 (trixie)"
VERSION_CODENAME=trixie
kernel: 7.2.14-arch1-1 (x86_64)
MemTotal:       16137256 kB
python: Python 3.13.5
cryptography: 43.0.0
openssl: OpenSSL 3.5.7 9 Jun 2026 (Library: OpenSSL 3.5.7 9 Jun 2026)
age: 1.2.1
ansible: ansible [core 2.19.11]
```
