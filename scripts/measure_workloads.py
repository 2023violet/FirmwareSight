#!/usr/bin/env python3
"""Measure the P0 size-guard and analysis workload, and print rows for the performance report.

Every number in the emitted table is read from a real run of the release binary on this machine:
stage timings come from the CLI's own stderr diagnostics line, and the total is measured around
the process. Nothing is estimated, and a workload that was not run is reported as such.

Peak RSS is deliberately not invented. Reading a child process's peak working set on Windows
needs either a Win32 API dependency in the CLI or polling from here; neither is sanctioned for
P0, so the column says NOT MEASURED and the report explains why. `04_TECH` forbids presenting a
guess as an observation.

Usage:
    python scripts/measure_workloads.py            # real fixtures + generated workloads
    python scripts/measure_workloads.py --runs 5   # repeat count (default 3)
"""

from __future__ import annotations

import argparse
import os
import platform
import re
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BIN = ROOT / "target" / "release" / ("fwsight.exe" if platform.system() == "Windows" else "fwsight")

DIAGNOSTICS = re.compile(
    r"hash (?P<hash>\d+) ms, read (?P<read>\d+) ms, parse (?P<parse>\d+) ms, "
    r"normalize (?P<normalize>\d+) ms"
)
ERROR_LINE = re.compile(r"^error: (.*?) \(code (ERR-[A-Z]+-\d+)", re.MULTILINE)

WORKLOADS = (
    ("fixtures/elf/p0-basic/firmware.elf", "real ELF fixture", None),
    ("fixtures/elf/p0-dual-region/firmware.elf", "real ELF fixture + GNU ld MAP",
     "fixtures/elf/p0-dual-region/firmware.map"),
    ("fixtures/generated/guard-100mib.elf", "Synthetic Performance Workload", None),
    ("fixtures/generated/guard-256mib.elf", "Synthetic Performance Workload", None),
    ("fixtures/generated/guard-512mib.elf", "Synthetic Performance Workload", None),
    ("fixtures/generated/guard-over-512mib.elf", "Synthetic Performance Workload", None),
)


def measure(path: Path, map_path: Path | None) -> dict:
    argv = [str(BIN), "analyze", str(path)]
    if map_path is not None:
        argv += ["--map", str(map_path)]

    # One cold run so the first sample is not paying for the operating system's page cache being
    # cold on a gigabyte file, then timed repeats.
    subprocess.run(argv, capture_output=True, text=True, check=False)

    rows: list[dict] = []
    for _ in range(runs):
        started = time.perf_counter()
        completed = subprocess.run(argv, capture_output=True, text=True, check=False)
        total_ms = (time.perf_counter() - started) * 1000.0

        match = DIAGNOSTICS.search(completed.stderr)
        error = ERROR_LINE.search(completed.stderr)
        rows.append(
            {
                "total_ms": round(total_ms, 1),
                "exit": completed.returncode,
                "hash": int(match.group("hash")) if match else None,
                "read": int(match.group("read")) if match else None,
                "parse": int(match.group("parse")) if match else None,
                "normalize": int(match.group("normalize")) if match else None,
                "result": "accepted" if completed.returncode == 0 else f"rejected: {error.group(1) if error else 'unknown'}",
                "code": error.group(2) if error else "",
            }
        )

    return {
        "bytes": path.stat().st_size,
        "rows": rows,
        "best": min(rows, key=lambda row: float(row["total_ms"])),
    }


def median(values: list[float]) -> float:
    ordered = sorted(values)
    middle = len(ordered) // 2
    if len(ordered) % 2:
        return ordered[middle]
    return (ordered[middle - 1] + ordered[middle]) / 2


runs = 3


def main(argv: list[str]) -> int:
    global runs
    parser = argparse.ArgumentParser(description="Measure the P0 analysis workloads.")
    parser.add_argument("--runs", type=int, default=3, help="timed repetitions per workload")
    args = parser.parse_args(argv[1:])
    runs = max(1, args.runs)

    if not BIN.exists():
        print(
            f"{BIN.name} not found. Build it first:\n"
            "  cargo build --release -p fwsight\n"
            "  python scripts/gen_p0_workload.py",
            file=sys.stderr,
        )
        return 1

    print("| workload | file size | hash ms | read ms | parse ms | normalize ms | total ms | exit | result |")
    print("| --- | --- | --- | --- | --- | --- | --- | --- | --- |")

    for rel, kind, map_rel in WORKLOADS:
        path = ROOT / rel
        if not path.exists():
            print(
                f"| `{Path(rel).name}` ({kind}) | - | - | - | - | - | - | - | "
                "NOT RUN (workload missing; run scripts/gen_p0_workload.py) |"
            )
            continue
        out = measure(path, ROOT / map_rel if map_rel else None)
        best = out["best"]
        totals = [float(row["total_ms"]) for row in out["rows"]]
        print(
            f"| `{path.name}` ({kind}) | {out['bytes']:,} | {best['hash']} | {best['read']} | "
            f"{best['parse']} | {best['normalize']} | {median(totals):.1f} (median of {len(totals)}) | "
            f"{best['exit']} | {best['result']} {best['code']} |"
        )

    print()
    print(f"- Host: {platform.system()} {platform.release()} ({platform.machine()})")
    print(f"- Python harness: {sys.version.split()[0]}")
    print(f"- Logical CPUs: {os.cpu_count()}")
    print("- Build profile: release (opt-level 3, lto thin, codegen-units 1), Cargo.lock as committed")
    print(f"- Binary: {BIN.relative_to(ROOT)}, {BIN.stat().st_size:,} bytes")
    print("- Peak RSS: NOT MEASURED (no sanctioned way to read a child's peak working set on this host)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
