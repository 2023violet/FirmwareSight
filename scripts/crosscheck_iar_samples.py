"""Cross-check FirmwareSight's IAR ILINK reading against IAR's own figures (ADR-0038).

Usage:
    python scripts/crosscheck_iar_samples.py <samples-dir> <fwsight-binary>

`<samples-dir>` holds one folder per sample with one `.out` and one `.map` that came from the same link.
The samples themselves are third-party files and are not committed; `evidence/D4-IAR/README.md` names
each one by repository, commit, path and SHA-256 so that this run can be repeated.

For every sample the script prints:
- IAR's totals from the map's last lines (`readonly code`, `readonly data`, `readwrite data`) beside
  FirmwareSight's Flash and RAM with the MAP;
- per module, IAR's `MODULE SUMMARY` (ro code + ro data, rw data) beside FirmwareSight's object rows.

IAR charges a module's share of the `Initializer bytes` (the stored copy of its initialized data) to that
module's `ro data`. FirmwareSight keeps those bytes apart as the linker's own, because the map does not
say each module's share once they are compressed. So a module with initialized data reads lower Flash
here by exactly its initializer share; the script reports that difference rather than hiding it.
"""

from __future__ import annotations

import hashlib
import re
import subprocess
import sys
from pathlib import Path


def number(text: str) -> int:
    return int(re.sub(r"[ ']", "", text))


def iar_totals(map_text: str) -> dict[str, int]:
    totals = {}
    for key, pattern in {
        "ro_code": r"([\d' ]+) bytes of readonly\s+code memory",
        "ro_data": r"([\d' ]+) bytes of readonly\s+data memory",
        "rw_data": r"([\d' ]+) bytes of readwrite data memory(?: \(\+ ([\d' ]+) absolute\))?",
    }.items():
        match = re.search(pattern, map_text)
        if not match:
            continue
        totals[key] = number(match.group(1).strip())
        if key == "rw_data" and match.group(2):
            totals["rw_absolute"] = number(match.group(2).strip())
    return totals


def iar_modules(map_text: str) -> dict[str, tuple[int, int]]:
    """`MODULE SUMMARY` rows as {key: (ro code + ro data, rw data)}, keyed like FirmwareSight's objects."""
    lines = map_text.splitlines()
    try:
        start = next(i for i, line in enumerate(lines) if "*** MODULE SUMMARY" in line)
    except StopIteration:
        return {}
    header = next(line for line in lines[start:] if re.match(r"\s+Module\s+ro code", line))
    # Numbers are right-aligned to the end of each column title.
    ends = [header.index(title) + len(title) for title in ("ro code", "ro data", "rw data")]
    index = {}
    for line in lines:
        match = re.match(r"^\[(\d+)\] = (.+)$", line)
        if match:
            name = re.split(r"[\\/]", match.group(2).strip())[-1]
            index[match.group(1)] = name if name.lower().endswith((".a", ".lib")) else None
    modules: dict[str, tuple[int, int]] = {}
    archive = None
    for line in lines[lines.index(header, start) + 1 :]:
        if line.startswith("***"):
            break
        group = re.match(r"^(\S.*): \[(\d+)\]\s*$", line)
        if group:
            archive = index.get(group.group(2))
            continue
        row = re.match(r"^    (\S+\.o)\b", line)
        if not row:
            continue
        values = []
        previous = len("    ") + len(row.group(1))
        for end in ends:
            cell = line[previous:end] if len(line) >= previous else ""
            previous = end
            cell = cell.strip()
            values.append(number(cell) if cell else 0)
        key = f"{archive}({row.group(1)})" if archive else row.group(1)
        flash, ram = values[0] + values[1], values[2]
        old = modules.get(key, (0, 0))
        modules[key] = (old[0] + flash, old[1] + ram)
    return modules


def fwsight_objects(text: str) -> dict[str, tuple[int, int]]:
    objects = {}
    inside = False
    for line in text.splitlines():
        if re.match(r"\s+Objects \(\d+\)", line):
            inside = True
            continue
        if re.match(r"\s+Libraries \(\d+\)", line):
            break
        if inside:
            match = re.match(r"\s+(\S+) bytes\s+(\S+) bytes\s+\d+\s+(.+)$", line)
            if match:
                flash, ram, name = match.groups()
                objects[name.strip()] = (int(flash), int(ram))
    return objects


def fwsight_totals(text: str) -> tuple[str, str]:
    flash = re.search(r"Nonvolatile / load image:\s+(.+)", text)
    ram = re.search(r"Runtime RAM:\s+(.+)", text)
    return (flash.group(1).strip() if flash else "?", ram.group(1).strip() if ram else "?")


def main() -> int:
    samples, binary = Path(sys.argv[1]), sys.argv[2]
    for folder in sorted(p for p in samples.iterdir() if p.is_dir()):
        out = next(folder.glob("*.out"))
        map_path = next(folder.glob("*.map"))
        map_text = map_path.read_text(encoding="utf-8", errors="replace")
        version = re.search(r"IAR ELF Linker (V[\d.]+)", map_text)
        result = subprocess.run(
            [binary, "analyze", str(out), "--map", str(map_path), "--objects"],
            capture_output=True,
            text=True,
            encoding="utf-8",
        )
        totals = iar_totals(map_text)
        flash, ram = fwsight_totals(result.stdout)
        print(f"## {folder.name}  ({version.group(1) if version else '?'})")
        print(f"   .out sha256 {hashlib.sha256(out.read_bytes()).hexdigest()}")
        print(f"   .map sha256 {hashlib.sha256(map_path.read_bytes()).hexdigest()}")
        print(f"   exit {result.returncode}")
        iar_flash = totals.get("ro_code", 0) + totals.get("ro_data", 0)
        iar_ram = totals.get("rw_data", 0) + totals.get("rw_absolute", 0)
        print(f"   IAR   Flash {iar_flash}  RAM {iar_ram}")
        print(f"   FS    Flash {flash}  RAM {ram}")
        ours = fwsight_objects(result.stdout)
        theirs = iar_modules(map_text)
        exact = [k for k in theirs if ours.get(k) == theirs[k]]
        ram_only = [k for k in theirs if k in ours and ours[k][1] == theirs[k][1] and ours[k][0] != theirs[k][0]]
        missing = [k for k in theirs if k not in ours and theirs[k] != (0, 0)]
        extra = [k for k in ours if k not in theirs]
        print(
            f"   modules: {len(theirs)} in IAR, {len(ours)} in FS; exact {len(exact)}; "
            f"RAM equal, Flash differs {len(ram_only)}; missing {len(missing)}; extra {len(extra)}"
        )
        for key in ram_only:
            print(f"     Flash differs: {key}: IAR {theirs[key][0]}, FS {ours[key][0]} (diff {theirs[key][0] - ours[key][0]})")
        for key in [k for k in theirs if k in ours and ours[k][1] != theirs[k][1]]:
            print(f"     RAM differs:   {key}: IAR {theirs[key]}, FS {ours[key]}")
        for key in missing:
            print(f"     missing: {key} {theirs[key]}")
        for key in extra:
            print(f"     extra:   {key} {ours[key]}")
        print()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
