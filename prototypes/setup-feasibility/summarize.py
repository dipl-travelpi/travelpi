#!/usr/bin/env python3
"""Collects the values for the tables in the thesis from results/ and writes results/summary.md.
Another folder can be given as argument, e.g. `python3 summarize.py results-dev`.

Times and memory are medians of runs 2 to 16 (run 1 is a warm-up run), like in the dashboard
tests. First runs of V2 (one per freshly flashed system) are all used."""
import csv
import pathlib
import statistics
import sys

HERE = pathlib.Path(__file__).parent
RESULTS = pathlib.Path(sys.argv[1]) if len(sys.argv) > 1 else HERE / "results"


def runs(name, skip_first=True):
    path = RESULTS / name
    if not path.exists():
        return None
    rows = list(csv.DictReader(path.open()))
    return rows[1:] if skip_first and len(rows) > 1 else rows


def median(rows, column):
    return statistics.median(float(row[column]) for row in rows)


def failed(rows):
    return sum(row["exit_code"] != "0" for row in rows)


def code_lines(path):
    lines = [line.strip() for line in path.read_text().splitlines()]
    return sum(1 for line in lines if line and not line.startswith("#"))


def read(name):
    path = RESULTS / name
    return path.read_text().strip() if path.exists() else "(missing)"


def main():
    out = ["# Summary of the setup feasibility tests", ""]

    out += ["## V1 Hardware detection", "",
            "| Variant | Runtime (median) | Max. memory (median) | Failed runs |", "|---|---|---|---|"]
    for variant in ["python", "shell", "ansible"]:
        rows = runs(f"v1-{variant}.csv")
        if rows:
            out.append(f"| {variant} | {median(rows, 'seconds') * 1000:.0f} ms "
                       f"| {median(rows, 'max_rss_kb') / 1024:.1f} MB | {failed(rows)} |")
    out += ["", "Detected features:", "", "```", read("v1-check.txt"), "```", "",
            "Stick plugged in while running (fill in by hand, see README):", "", "```", read("v1-hotplug.txt"), "```", ""]

    out += ["## V2 Setup", "",
            "| Variant | First run (each flash) | Repeat (median) | Changes in 2nd run | Error: exit code / went on | Code lines |",
            "|---|---|---|---|---|---|"]
    files = {"python": "setup.py", "shell": "setup.sh", "ansible": "setup.yml"}
    for variant, file in files.items():
        first = [float(row["seconds"]) for path in sorted(RESULTS.glob(f"v2-first-{variant}-*.csv"))
                 for row in csv.DictReader(path.open())]
        repeat = runs(f"v2-repeat-{variant}.csv")
        changes = RESULTS / f"v2-{variant}-changes.txt"
        changed = sum(line.startswith(">") for line in changes.read_text().splitlines()) if changes.exists() else "?"
        error = dict(line.split("=") for line in read(f"v2-error-{variant}.txt").splitlines() if "=" in line)
        first_text = ", ".join(f"{s:.0f} s" for s in first) or "?"
        repeat_text = f"{median(repeat, 'seconds'):.1f} s" if repeat else "?"
        out.append(
            f"| {variant} | {first_text} | {repeat_text} | {changed} "
            f"| {error.get('exit_code', '?')} / {error.get('went_on_after_error', '?')} "
            f"| {code_lines(HERE / 'v2-setup' / file)} |")
    out += ["", "Installing Ansible (part of the Ansible first run):", ""]
    out += [f"- {path.name}: {path.read_text().strip().replace(chr(10), ', ')}"
            for path in sorted(RESULTS.glob("v2-ansible-install-*.txt"))] or ["- (missing)"]
    out.append("")

    out += ["## V3 Backup", "", read("v3-testdata.txt"), "",
            "| Variant | Encrypt (median) | Decrypt (median) |", "|---|---|---|"]
    baseline = runs("v3-pty-baseline.csv")
    for variant in ["fernet", "openssl", "age", "vault"]:
        encrypt, decrypt = runs(f"v3-{variant}-encrypt.csv"), runs(f"v3-{variant}-decrypt.csv")
        if encrypt and decrypt:
            e, d = median(encrypt, "seconds") * 1000, median(decrypt, "seconds") * 1000
            note = ""
            if variant == "age" and baseline:
                b = median(baseline, "seconds") * 1000
                note = f" (without pty-run.py: {e - b:.0f} ms / {d - b:.0f} ms)"
            out.append(f"| {variant} | {e:.0f} ms | {d:.0f} ms{note} |")
    out += ["", "```", read("v3-checks.txt"), "```", "", "Preinstalled on a fresh system:", "",
            "```", read("preinstalled.txt"), "```", "", "Environment:", "", "```", read("env.txt"), "```"]

    report = "\n".join(out) + "\n"
    (RESULTS / "summary.md").write_text(report)
    print(report)


if __name__ == "__main__":
    main()
