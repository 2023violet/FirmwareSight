#!/usr/bin/env python3
"""Generate the real P0 ELF/MAP fixtures and their provenance records.

Why a generator instead of hand-written bytes: P0 requires an ELF that a real linker produced,
so the memory-accounting claims rest on genuine load evidence (PRE-0.5) and the GNU ld MAP
adapter parses a genuine MAP file (PRE-0.6).

The script compiles with relative paths on purpose. GNU ld writes the object paths it was given
into the MAP, so building from an absolute temp path would leak a private machine path into a
committed fixture and make goldens non-reproducible.

Usage:  python scripts/gen_p0_fixtures.py [--force]

Requires arm-none-eabi-gcc. Fixture binaries are committed, so running the parser tests does
NOT require this toolchain; only regenerating the fixtures does.
"""

from __future__ import annotations

import argparse
import datetime
import hashlib
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

ABSOLUTE_PATH_RE = re.compile(r"^[A-Za-z]:[\\/]|^/Users/|^/home/|^/tmp/")


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def tool_version(exe: str) -> str:
    out = subprocess.run([exe, "--version"], capture_output=True, text=True, check=True)
    return out.stdout.splitlines()[0].strip()


def run(cmd: list[str], cwd: Path) -> None:
    printable = " ".join(cmd)
    print(f"    $ {printable}")
    result = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)
    if result.returncode != 0:
        sys.exit(f"command failed: {printable}\n{result.stdout}\n{result.stderr}")


def write_source(dirpath: Path) -> None:
    """One source file exercising every budget category the memory model must separate."""
    (dirpath / "source").mkdir(parents=True, exist_ok=True)
    (dirpath / "source" / "main.c").write_text(
        '''/*
 * FirmwareSight P0 technical fixture.
 *
 * Each object below is placed in a different memory class on purpose so that section
 * classification, symbol tables and the two memory budgets can be asserted against a real
 * linker's output rather than against a description of one.
 *
 * Deliberately not product demo data: the numbers are here to be checkable.
 */

/* read-only: contributes to the load image only */
const unsigned int cfg_table[8] = {1u, 2u, 3u, 4u, 5u, 6u, 7u, 8u};

/* initialized data: image bytes AND live RAM */
int g_threshold = 0x12345678;

/* zero initialized: RAM only, no image payload */
int g_scratch;

/* custom region, explicitly placed by the linker script */
__attribute__((section(".ota_staging"), used)) unsigned char staging_area[64] = {9u, 8u, 7u};

int mqtt_task(void) {
    return g_threshold + (int)cfg_table[0] + g_scratch + (int)staging_area[3];
}

int tls_handshake(void) {
    return mqtt_task() ^ 0xAA;
}

int sensor_fifo(void) {
    return tls_handshake() + 1;
}

int main(void) {
    return sensor_fifo();
}
''',
        encoding="utf-8",
        newline="\n",
    )


def write_linker_script(dirpath: Path) -> None:
    """ROM/RAM split with .data and .ota resident in RAM but loaded from ROM."""
    (dirpath / "p0-dual-region.ld").write_text(
        '''/*
 * Controlled layout for the P0 memory-accounting proof.
 *
 * RAM  (rwx) is writable, therefore live storage.
 * ROM  (rx)  is not writable, therefore the load image.
 *
 * `.data` and `.ota` are VMA-resident in RAM but LMA-stored in ROM (AT> ROM), which is the
 * exact shape that makes nonvolatile footprint and runtime RAM footprint two different
 * numbers. A `.data`-by-name rule would produce the same totals here by luck; the proof is
 * that PT_LOAD carries both addresses independently.
 */
MEMORY {
  ROM (rx)  : ORIGIN = 0x08000000, LENGTH = 1024K
  RAM (rwx) : ORIGIN = 0x20000000, LENGTH = 256K
}

SECTIONS {
  .text : { *(.text*) } > ROM

  .rodata : { *(.rodata*) } > ROM

  _sidata = LOADADDR(.data);
  .data : { *(.data*) } > RAM AT> ROM

  .bss (NOLOAD) : { *(.bss*) *(COMMON) } > RAM

  .ota : { *(.ota_staging*) } > RAM AT> ROM
}
''',
        encoding="utf-8",
        newline="\n",
    )


def build(dirpath: Path, gcc: str, extra_ldflags: list[str], map_name: str | None) -> None:
    build_dir = dirpath / "build"
    if build_dir.exists():
        shutil.rmtree(build_dir)
    build_dir.mkdir()

    run([gcc, "-g", "-Os", "-mcpu=cortex-m4", "-mthumb", "-c", "source/main.c", "-o", "build/main.o"], dirpath)

    link = [
        gcc,
        "-g",
        "-Os",
        "-mcpu=cortex-m4",
        "-mthumb",
        "-nostdlib",
        "-nostartfiles",
        "-e",
        "main",
        "build/main.o",
        "-o",
        "firmware.elf",
    ]
    link += extra_ldflags
    if map_name:
        link.insert(-2, f"-Wl,-Map={map_name}")
    run(link, dirpath)
    shutil.rmtree(build_dir)


def assert_no_absolute_paths(dirpath: Path) -> None:
    """A committed MAP must not carry a private machine path."""
    for candidate in sorted(dirpath.glob("*.map")):
        for lineno, line in enumerate(candidate.read_text(encoding="utf-8", errors="replace").splitlines(), 1):
            field = line.strip().split()
            for token in field:
                if ABSOLUTE_PATH_RE.match(token):
                    sys.exit(
                        f"fixture {candidate.name}:{lineno} contains an absolute path "
                        f"({token!r}); refusing to commit a private machine path"
                    )


def fixture_toml(
    *,
    fixture_id: str,
    purpose: str,
    gcc: str,
    command: str,
    architecture: str,
    capability: str,
    limitations: list[str],
    files: list[str],
) -> str:
    generated = datetime.date.today().isoformat()
    parts = [
        "# Provenance record for a committed P0 fixture. Generated by scripts/gen_p0_fixtures.py.",
        "",
        f'fixture_id = "{fixture_id}"',
        f'purpose = "{purpose}"',
        'license = "Written for this project; no third-party code copied"',
        f'generated_on = "{generated}"',
        "",
        "[toolchain]",
        'name = "arm-none-eabi-gcc"',
        f'version = "{tool_version(gcc)}"',
        f'linker = "{tool_version(gcc.replace("gcc", "ld"))}"',
        "",
        "[build]",
        f"command = '''{command}'''",
        'rebuild_script = "scripts/gen_p0_fixtures.py"',
        "",
        "[expectations]",
        f'architecture = "{architecture}"',
        f'expected_capability = "{capability}"',
        "",
    ]
    parts += [f'[[known_limitations]]\ndescription = "{item}"\n' for item in limitations]
    parts += [f'[[files]]\npath = "{name}"\n' for name in files]
    return "\n".join(parts)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--force", action="store_true", help="overwrite existing fixture binaries")
    args = parser.parse_args()

    gcc = shutil.which("arm-none-eabi-gcc")
    if gcc is None:
        sys.exit(
            "arm-none-eabi-gcc not found. Parser and golden tests do NOT require it; it is only "
            "needed to regenerate these fixtures. The committed binaries remain valid."
        )

    fixtures = ROOT / "fixtures"
    manifest_files: list[dict[str, str]] = []

    targets = [
        {
            "dir": fixtures / "elf" / "p0-basic",
            "id": "p0-basic",
            "purpose": "Parser infrastructure fixture: format detection, sections, symbols, "
            "SHA-256, debug-section presence and capability reporting.",
            "flags": [],
            "map": None,
            "arch": "ARM (cortex-m4, thumb, 32-bit little-endian)",
            "capability": "ELF Supported; sections and symbols Available; MAP NotProvided",
            "limitations": [
                "single-region default layout, so it cannot prove dual memory accounting",
                "no DWARF consumption in P0 even though debug sections are present",
                "not a real customer firmware image",
            ],
        },
        {
            "dir": fixtures / "elf" / "p0-dual-region",
            "id": "p0-dual-region",
            "purpose": "Memory accounting proof fixture: ROM/RAM regions with .data and .ota "
            "VMA-resident in RAM and LMA-stored in ROM.",
            "flags": ["-T", "p0-dual-region.ld"],
            "map": "firmware.map",
            "arch": "ARM (cortex-m4, thumb, 32-bit little-endian)",
            "capability": "ELF Supported; MAP Supported via gnu_ld adapter",
            "limitations": [
                "layout is a controlled linker script, not a vendor SDK default",
                "proves the accounting rule on this evidence only, not on every toolchain",
            ],
        },
    ]

    for target in targets:
        dirpath = target["dir"]
        print(f"[fixture] {target['id']} -> {dirpath.relative_to(ROOT)}")
        dirpath.mkdir(parents=True, exist_ok=True)
        write_source(dirpath)
        if target["flags"]:
            write_linker_script(dirpath)
        elf = dirpath / "firmware.elf"
        if elf.exists() and not args.force:
            sys.exit(f"{elf} already exists; pass --force to regenerate and re-record hashes")
        build(dirpath, gcc, target["flags"], target["map"])
        assert_no_absolute_paths(dirpath)

        names = [
            p.relative_to(dirpath).as_posix()
            for p in sorted(dirpath.iterdir())
            if p.is_file() and p.name != "fixture.toml"
        ]
        command = " ".join(
            [
                "arm-none-eabi-gcc -g -Os -mcpu=cortex-m4 -mthumb",
                "-c source/main.c -o build/main.o",
                "&& arm-none-eabi-gcc -g -Os -mcpu=cortex-m4 -mthumb -nostdlib -nostartfiles",
                "-e main build/main.o",
                ("-T p0-dual-region.ld" if target["flags"] else ""),
                (f"-Wl,-Map=firmware.map" if target["map"] else ""),
                "-o firmware.elf",
            ]
        )
        (dirpath / "fixture.toml").write_text(
            fixture_toml(
                fixture_id=target["id"],
                purpose=target["purpose"],
                gcc=gcc,
                command=command,
                architecture=target["arch"],
                capability=target["capability"],
                limitations=target["limitations"],
                files=names,
            ),
            encoding="utf-8",
            newline="\n",
        )
        for name in names + ["fixture.toml"]:
            manifest_files.append(
                {
                    "path": (dirpath / name).relative_to(ROOT).as_posix(),
                    "sha256": sha256(dirpath / name),
                    "fixture": target["id"],
                    "purpose": "elf-binary" if name.endswith(".elf") else "fixture-support",
                }
            )

    # Malformed fixtures: tiny, committed, and each one exercises a distinct early rejection.
    malformed = fixtures / "malformed"
    malformed.mkdir(parents=True, exist_ok=True)
    (malformed / "empty.bin").write_bytes(b"")
    (malformed / "wrong-magic.bin").write_bytes(b"\x7fELF\x02\x01\x01\x00"[:4] + b"not-elf-at-all")
    source_elf = fixtures / "elf" / "p0-basic" / "firmware.elf"
    prefix = source_elf.read_bytes()[:64]
    (malformed / "truncated-elf.bin").write_bytes(prefix)
    (malformed / "sparse-elf-header.bin").write_bytes(b"\x7fELF" + (b"\x00" * 16))
    (malformed / "sparse-elf-header.bin").with_name("README.md").write_text(
        "# Malformed fixtures\n\n"
        "Each file here is a regression input for the untrusted-intake path. None of them is\n"
        "large; they are named for what they claim, not for what they are.\n\n"
        "- `empty.bin` - zero bytes: rejected as malformed, no panic.\n"
        "- `wrong-magic.bin` - ELF magic followed by prose: rejected as an unsupported format.\n"
        "- `truncated-elf.bin` - a real ELF header cut short, so table offsets point past EOF.\n"
        "- `sparse-elf-header.bin` - ELF magic plus a zero-filled header, so every declared\n"
        "  size and offset reads as zero: rejected rather than trusted.\n\n"
        "The >512 MiB guard workload is never committed; generate it with\n"
        "`python scripts/gen_p0_workload.py`.\n",
        encoding="utf-8",
        newline="\n",
    )
    for path in sorted(malformed.glob("*.bin")):
        manifest_files.append(
            {
                "path": path.relative_to(ROOT).as_posix(),
                "sha256": sha256(path),
                "fixture": "malformed",
                "purpose": "negative-input-regression",
            }
        )

    (fixtures / "manifest.json").write_text(
        json.dumps(
            {
                "schema": "firmwaresight-fixtures-1",
                "generated_by": "scripts/gen_p0_fixtures.py",
                "note": "Tests verify these hashes before trusting any golden.",
                "files": sorted(manifest_files, key=lambda e: e["path"]),
            },
            indent=2,
        )
        + "\n",
        encoding="utf-8",
        newline="\n",
    )

    print(f"\nwrote {len(manifest_files)} manifest entries")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
