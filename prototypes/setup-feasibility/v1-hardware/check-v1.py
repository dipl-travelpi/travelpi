#!/usr/bin/env python3
"""V1: compares the saved output of every variant with reference.json and counts the correctly
detected features (6 in total). Writes the result to results/v1-check.txt.

The Ansible check is a best effort, because Ansible facts have their own names. Look at
results/v1-ansible-output.txt yourself if a result seems wrong.
"""
import json
import pathlib

HERE = pathlib.Path(__file__).parent
RESULTS = HERE.parent / "results"
FEATURES = ["model", "memory", "cpu_cores", "wifi_internal", "bluetooth", "wifi_usb"]


def memory_ok(reference, nominal_mb, total_mb):
    # The total memory the OS sees is smaller than the nominal size (memory reserved for the GPU).
    return nominal_mb == reference or (total_mb or 0) >= 0.75 * reference


def stick_ok(reference, ids):
    return bool(ids) and (reference.startswith("xxxx") or reference in ids)


def check_python(text, ref):
    out = json.loads(text)
    return {
        "model": out.get("model") == ref["model"],
        "memory": memory_ok(ref["memory_mb"], out.get("memory_mb"), out.get("memory_total_mb")),
        "cpu_cores": out.get("cpu_cores") == ref["cpu_cores"],
        "wifi_internal": bool(out.get("wifi_internal")) == ref["wifi_internal"],
        "bluetooth": bool(out.get("bluetooth")) == ref["bluetooth"],
        "wifi_usb": stick_ok(ref["usb_stick_id"], " ".join(s["id"] for s in out.get("wifi_usb", []))),
    }


def check_shell(text, ref):
    lines = [line.split("=", 1) for line in text.splitlines() if "=" in line]
    values = {}
    for key, value in lines:
        values.setdefault(key, []).append(value)
    first = lambda key: values.get(key, [None])[0]
    total_kb = first("memory_total_kb")
    return {
        "model": first("model") == ref["model"],
        "memory": memory_ok(ref["memory_mb"], int(first("memory_mb") or 0), int(total_kb or 0) / 1024),
        "cpu_cores": first("cpu_cores") == str(ref["cpu_cores"]),
        "wifi_internal": ("wifi_internal" in values) == ref["wifi_internal"],
        "bluetooth": ("bluetooth" in values) == ref["bluetooth"],
        "wifi_usb": stick_ok(ref["usb_stick_id"], " ".join(values.get("wifi_usb", []))),
    }


def check_ansible(text, ref):
    facts = json.loads(text.split("=>", 1)[1])["ansible_facts"]
    interfaces = [v for k, v in facts.items() if isinstance(v, dict) and k.startswith("ansible_wl")]
    as_text = json.dumps(facts)
    return {
        "model": "Raspberry Pi" in as_text,
        "memory": memory_ok(ref["memory_mb"], None, facts.get("ansible_memtotal_mb")),
        "cpu_cores": ref["cpu_cores"] in (facts.get("ansible_processor_nproc"), facts.get("ansible_processor_vcpus")),
        "wifi_internal": any(i.get("module") == "brcmfmac" for i in interfaces) == ref["wifi_internal"],
        "bluetooth": ("hci0" in as_text) == ref["bluetooth"],
        "wifi_usb": any(i.get("module") not in (None, "brcmfmac") for i in interfaces),
    }


def main():
    ref = json.loads((HERE / "reference.json").read_text())
    checks = {"python": check_python, "shell": check_shell, "ansible": check_ansible}
    results = {name: check((RESULTS / f"v1-{name}-output.txt").read_text(), ref) for name, check in checks.items()}

    lines = [f"{'feature':15}" + "".join(f"{name:>10}" for name in results)]
    for feature in FEATURES:
        lines.append(f"{feature:15}" + "".join(f"{'yes' if r[feature] else 'no':>10}" for r in results.values()))
    lines.append(f"{'detected':15}" + "".join(f"{sum(r.values()):>8}/6" for r in results.values()))
    report = "\n".join(lines)
    print(report)
    (RESULTS / "v1-check.txt").write_text(report + "\n")


if __name__ == "__main__":
    main()
