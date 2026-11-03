#!/usr/bin/env python3
"""Copies a file and changes one byte in the middle (to 'A', or 'B' if it already is 'A').
Usage: tamper.py INPUT OUTPUT"""
import sys

with open(sys.argv[1], "rb") as f:
    data = bytearray(f.read())
middle = len(data) // 2
data[middle] = ord("B") if data[middle] == ord("A") else ord("A")
with open(sys.argv[2], "wb") as f:
    f.write(data)
