#!/usr/bin/env python3
"""V3, Fernet variant. Usage: fernet.py encrypt|decrypt INPUT OUTPUT

The password comes from the environment variable BACKUP_PASSWORD. The key is derived with
scrypt; the 16-byte salt is stored in front of the Fernet token."""
import base64
import os
import sys

from cryptography.fernet import Fernet, InvalidToken
from cryptography.hazmat.primitives.kdf.scrypt import Scrypt


def key(password, salt):
    kdf = Scrypt(salt=salt, length=32, n=2**14, r=8, p=1)
    return base64.urlsafe_b64encode(kdf.derive(password.encode()))


def main():
    mode, source, target = sys.argv[1:4]
    password = os.environ["BACKUP_PASSWORD"]
    with open(source, "rb") as f:
        data = f.read()

    if mode == "encrypt":
        salt = os.urandom(16)
        result = salt + Fernet(key(password, salt)).encrypt(data)
    else:
        try:
            result = Fernet(key(password, data[:16])).decrypt(data[16:])
        except InvalidToken:
            sys.exit("error: wrong password or modified file")

    with open(target, "wb") as f:
        f.write(result)


if __name__ == "__main__":
    main()
