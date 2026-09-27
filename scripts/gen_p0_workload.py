#!/usr/bin/env python3
"""Generate the synthetic size-guard workloads for `P0_PERFORMANCE_REPORT.md`.

These are NOT firmware. Each file is the first bytes of a real linked ELF with trailing padding,
which is exactly the shape the full-buffer guard has to survive: a plausible header followed by
an unreasonable amount of data. Nothing about them is a claim that such a firmware exists.

They are never committed: `fixtures/generated/` is in `.gitignore`, and ~1.1 GiB of generated
bytes would bloat the repository for a number that has to be re-measured on each machine anyway.

Usage:
    python scripts/gen_p0_workload.py            # write the workloads
    python scripts/gen_p0_workload.py --clean    # delete them
    python scripts/gen_p0_workload.py --verify   # print size and SHA-256, write nothing
"""

from __future__ import annotations

import argparse
import hashlib
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / "fixtures" / "elf" / "p0-dual-region" / "firmware.elf"
OUT = ROOT / "fixtures" / "generated"

MIB = 1024 * 1024
# 512 MiB is the frozen default limit, so the boundary is tested from both sides.
WORKLOADS = (
    ("guard-100mib.elf", 100 * MIB, "near, under the limit"),
    ("guard-256mib.elf", 256 * MIB, "near, half the limit"),
    ("guard-512mib.elf", 512 * MIB, "exactly at the limit"),
    ("guard-over-512mib.elf", 512 * MIB + 1024, "over the limit"),
)

CHUNK = 4 * MIB


def build(name: str, size: int, head: bytes) -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    path = OUT / name
    with path.open("wb") as handle:
        handle.write(head)
        remaining = size - len(head)
        # Zeros rather than random bytes: the workload must be reproducible from the source ELF,
        # so a re-run on another machine measures the same content.
        block = b"\x00" * min(CHUNK, max(remaining, 0))
        while remaining > 0:
            piece = block[: min(len(block), remaining)]
            handle.write(piece)
            remaining -= len(piece)


def sha256_of(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(CHUNK), b""):
            digest.update(block)
    return digest.hexdigest()


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description="Generate synthetic size-guard workloads.")
    parser.add_argument("--clean", action="store_true", help="remove the generated workloads")
    parser.add_argument("--verify", action="store_true", help="report what is on disk")
    args = parser.parse_args(argv[1:])

    if args.clean:
        for name, _size, _note in WORKLOADS:
            path = OUT / name
            if path.exists():
                path.unlink()
                print(f"removed {path.relative_to(ROOT)}")
        return 0

    if not args.verify:
        if not SOURCE.exists():
            print(f"source fixture missing: {SOURCE}", file=sys.stderr)
            return 1
        head = SOURCE.read_bytes()
        for name, size, note in WORKLOADS:
            build(name, size, head)
            path = OUT / name
            print(
                f"wrote {path.relative_to(ROOT)}  {path.stat().st_size} bytes  ({note})  "
                f"sha256 {sha256_of(path)}"
            )
        return 0

    total = 0
    for name, size, note in WORKLOADS:
        path = OUT / name
        if not path.exists():
            print(f"missing {name} - run without --verify first", file=sys.stderr)
            total += 1
            continue
        print(f"{name}  {path.stat().st_size} bytes  {note}  sha256 {sha256_of(path)}")
    return 1 if total else 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
