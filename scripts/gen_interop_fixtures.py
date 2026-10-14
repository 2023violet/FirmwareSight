#!/usr/bin/env python3
"""Generate compiler-produced ELF format fixtures, NOT Keil/IAR vendor fixtures.

Requires existing clang and llvm-readobj. No IDE installation or user project build.
Invoke with NEW fixture and external evidence directories; builds use relative paths.
These relocatable objects prove metadata parsing, not executable memory accounting.
"""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

TARGETS = ("armv7-none-eabi", "armeb-none-eabi", "aarch64-none-elf", "aarch64_be-none-elf")
SOURCE = """/* Synthetic public test facts; no device/customer code. */
const unsigned char payload[4] = {1, 2, 3, 4};
volatile unsigned int initialized = 7;
volatile unsigned int zeroed;
unsigned int fixture_function(void) { return initialized + zeroed + payload[0]; }
"""


def main():
    output = Path(sys.argv[1])
    evidence = Path(sys.argv[2])
    output.mkdir(parents=True, exist_ok=False)
    evidence.mkdir(parents=True, exist_ok=False)
    (output / "fixture.c").write_text(SOURCE, encoding="utf-8", newline="\n")
    compiler = subprocess.check_output(["clang", "--version"], text=True).splitlines()[0]
    reader = subprocess.check_output(["llvm-readobj", "--version"], text=True).splitlines()[0]
    records = []
    for target in TARGETS:
        filename = f"{target}.o"
        command = ["clang", f"--target={target}", "-c", "-O0", "-g0", "fixture.c", "-o", filename]
        subprocess.run(command, cwd=output, check=True, capture_output=True)
        inspected = subprocess.check_output(
            ["llvm-readobj", "--file-headers", "--sections", "--symbols", filename], cwd=output
        )
        (evidence / f"{target}.readobj.txt").write_bytes(inspected)
        data = (output / filename).read_bytes()
        records.append({"file": filename, "target": target, "sha256": hashlib.sha256(data).hexdigest(),
                        "bytes": len(data), "command": command, "container": "ELF",
                        "evidence": "CLANG_RELOCATABLE_BASIC_METADATA_ONLY", "vendor_cohort": "NOT_KEIL_OR_IAR"})
    (output / "provenance.json").write_text(json.dumps(
        {"compiler": compiler, "independent_reader": reader, "files": records}, indent=2
    ) + "\n", encoding="utf-8", newline="\n")


if __name__ == "__main__":
    main()
