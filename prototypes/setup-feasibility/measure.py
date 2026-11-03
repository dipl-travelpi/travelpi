#!/usr/bin/env python3
"""Runs a command several times and writes wall time and peak memory of every run to a CSV file.

Usage: measure.py --runs 16 --out results/x.csv [--save-output FILE] -- COMMAND ...

Peak memory is ru_maxrss from wait4(), the same value that /usr/bin/time -v reports as
"Maximum resident set size". The command gets no input (stdin is /dev/null).
"""
import argparse
import csv
import os
import subprocess
import sys
import time


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--runs", type=int, default=16)
    parser.add_argument("--out", required=True)
    parser.add_argument("--save-output", help="file for the standard output of the last run")
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    command = args.command[1:] if args.command[:1] == ["--"] else args.command

    os.makedirs(os.path.dirname(args.out) or ".", exist_ok=True)
    with open(args.out, "w", newline="") as f:
        writer = csv.writer(f)
        writer.writerow(["run", "seconds", "max_rss_kb", "exit_code"])
        for run in range(1, args.runs + 1):
            start = time.perf_counter()
            process = subprocess.Popen(command, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE)
            output = process.stdout.read()
            _, status, usage = os.wait4(process.pid, 0)
            seconds = time.perf_counter() - start
            process.returncode = os.waitstatus_to_exitcode(status)
            writer.writerow([run, f"{seconds:.4f}", usage.ru_maxrss, process.returncode])
            print(f"run {run:2}: {seconds:7.3f} s  {usage.ru_maxrss / 1024:6.1f} MB  exit {process.returncode}",
                  file=sys.stderr)

    if args.save_output:
        with open(args.save_output, "wb") as f:
            f.write(output)


if __name__ == "__main__":
    main()
