#!/usr/bin/env python3
"""Runs a command in a pseudo terminal and types BACKUP_PASSWORD whenever it asks for a
passphrase. Only needed to time age, which reads the passphrase from the terminal.
Usage: pty-run.py COMMAND ..."""
import os
import pty
import sys


def main():
    password = os.environ["BACKUP_PASSWORD"].encode() + b"\n"
    pid, fd = pty.fork()
    if pid == 0:
        os.execvp(sys.argv[1], sys.argv[1:])

    buffer = b""
    while True:
        try:
            chunk = os.read(fd, 1024)
        except OSError:  # the command has closed the terminal
            break
        if not chunk:
            break
        buffer += chunk
        if b"passphrase" in buffer.lower() and buffer.rstrip().endswith(b":"):
            os.write(fd, password)
            buffer = b""

    _, status = os.waitpid(pid, 0)
    sys.exit(os.waitstatus_to_exitcode(status))


if __name__ == "__main__":
    main()
