#!/usr/bin/env bash
# V1: runs every variant 16 times and saves the output of the last run.
# The USB Wi-Fi stick must be plugged in. Run as the normal user (not with sudo).
set -euo pipefail
cd "$(dirname "$0")/.."

python3 measure.py --runs 16 --out results/v1-python.csv --save-output results/v1-python-output.txt \
  -- python3 v1-hardware/detect.py
python3 measure.py --runs 16 --out results/v1-shell.csv --save-output results/v1-shell-output.txt \
  -- bash v1-hardware/detect.sh
python3 measure.py --runs 16 --out results/v1-ansible.csv --save-output results/v1-ansible-output.txt \
  -- ansible localhost -m ansible.builtin.setup

python3 v1-hardware/check-v1.py
