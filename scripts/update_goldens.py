#!/usr/bin/env python3
"""Regenerate the committed goldens from the real `fwsight` binary - only with --confirm.

Goldens exist to catch an unintended change, so this script is deliberately awkward to use:
without `--confirm` it prints what *would* change and writes nothing. A failing test never
rewrites its own expectation (`05_ENGINEERING/02_TEST_STRATEGY.md`), and a one-command
"make it green" path would defeat the point of the golden.

The numbers are produced by the binary, not by this script: the CLI is run with `--json` and its
stdout is reformatted with the key order it already arrives in. Nothing here recomputes a fact,
because a second implementation of the projection is a second thing that can be wrong.

Usage:
    python scripts/update_goldens.py             # show the semantic diff, write nothing
    python scripts/update_goldens.py --confirm   # write the new goldens
"""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parent.parent
FIXTURES = (
    # (golden name, elf path, map path or None)
    ("p0-basic", "fixtures/elf/p0-basic/firmware.elf", None),
    (
        "p0-dual-region",
        "fixtures/elf/p0-dual-region/firmware.elf",
        "fixtures/elf/p0-dual-region/firmware.map",
    ),
)


def cargo() -> str:
    for candidate in ("cargo", "cargo.exe", "cargo.cmd"):
        found = shutil.which(candidate)
        if found is not None:
            return found
    return "cargo"


def analyze(elf: str, map_path: str | None) -> dict[str, Any]:
    argv = [cargo(), "run", "-q", "-p", "fwsight", "--", "analyze", elf, "--json"]
    if map_path is not None:
        argv += ["--map", map_path]
    completed = subprocess.run(argv, cwd=ROOT, capture_output=True, text=True, check=False)
    if completed.returncode != 0:
        raise SystemExit(
            f"analyze failed for {elf} (exit {completed.returncode}):\n{completed.stderr}"
        )
    return json.loads(completed.stdout)


def pretty(document: dict[str, Any]) -> str:
    # Key order is left exactly as serde emitted it: declaration order is part of the contract.
    return json.dumps(document, indent=2, ensure_ascii=False) + "\n"


def flatten(node: Any, prefix: str = "") -> dict[str, Any]:
    """A dotted view of a document, so a diff can name the leaf that moved."""
    if isinstance(node, dict):
        out: dict[str, Any] = {}
        for key, value in node.items():
            out.update(flatten(value, f"{prefix}.{key}" if prefix else key))
        return out
    if isinstance(node, list):
        return {prefix: json.dumps(node, sort_keys=True)}
    return {prefix: node}


def describe(old: dict[str, Any], new: dict[str, Any]) -> list[str]:
    before, after = flatten(old), flatten(new)
    lines: list[str] = []
    for key in sorted(set(before) - set(after)):
        lines.append(f"- {key} (removed)")
    for key in sorted(set(after) - set(before)):
        lines.append(f"+ {key} = {after[key]}")
    for key in sorted(set(before) & set(after)):
        if before[key] != after[key]:
            lines.append(f"~ {key}: {before[key]} -> {after[key]}")
    return lines


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description="Regenerate P0 goldens.")
    parser.add_argument(
        "--confirm",
        action="store_true",
        help="actually write the files; without it nothing changes",
    )
    args = parser.parse_args(argv[1:])

    total = 0
    for name, elf, map_path in FIXTURES:
        document = analyze(elf, map_path)
        targets = {
            ROOT / "golden" / "cli" / f"{name}-analyze.json": document,
            ROOT / "golden" / "core" / f"{name}-memory.json": document["memory"],
        }
        for path, payload in targets.items():
            text = pretty(payload)
            previous = path.read_text(encoding="utf-8") if path.exists() else None
            if previous is not None and json.loads(previous) == payload:
                print(f"unchanged  {path.relative_to(ROOT)}")
                continue
            diff = describe(json.loads(previous), payload) if previous else ["(new file)"]
            total += 1
            print(f"changed    {path.relative_to(ROOT)}")
            for line in diff[:20]:
                print(f"           {line}")
            if len(diff) > 20:
                print(f"           ... {len(diff) - 20} more")
            if args.confirm:
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(text, encoding="utf-8", newline="\n")

    if not args.confirm:
        if total:
            print(
                f"\n{total} golden(s) would change. Review the diff above, then re-run with "
                "--confirm and record the reason in the commit message.",
                file=sys.stderr,
            )
            return 1
        print("\nall goldens match current output.")
        return 0

    print(f"\nwrote {total} changed golden(s).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
