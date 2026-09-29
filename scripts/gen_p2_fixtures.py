#!/usr/bin/env python3
"""Generate the P2 Compare fixture pair and append it to the fixture manifest.

Why a pair instead of one binary: Compare answers "what changed between two builds", so the stage
needs two real linker outputs whose difference is authored on purpose. Every expectation this pair
carries — an unchanged section, a grown section, an added section, a removed section, a symbol whose
size changed, a symbol whose address moved, an added symbol, a removed symbol — is a fact a test can
assert against a genuine GNU ld result rather than against a description of one.

Why the two sides get their own linker script: an output section declared in the script but fed by
no input object still appears, at size zero, on both sides. That would turn "this section was
removed" into "this section shrank to zero", which is exactly the confusion Compare must not create.

The script compiles with relative paths, like gen_p0_fixtures.py: GNU ld writes the object paths it
was given into the MAP, so an absolute temp path would leak a private machine path into a committed
fixture.

Usage:  python scripts/gen_p2_fixtures.py [--force]

Requires arm-none-eabi-gcc. The binaries are committed, so running the parser, storage and diff
tests does NOT require this toolchain; only regenerating the pair does. Nothing under
fixtures/elf/p0-basic or fixtures/elf/p0-dual-region is read, written or re-hashed here.
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
PAIR = ROOT / "fixtures" / "elf" / "p2-diff"

ABSOLUTE_PATH_RE = re.compile(r"^[A-Za-z]:[\\/]|^/Users/|^/home/|^/tmp/")

COMMON_SOURCE = r"""/*
 * FirmwareSight P2 Compare fixture — the half that is byte-identical in both builds.
 *
 * Nothing in this file changes between the base and the target build, so every fact it produces is
 * a control: `.rodata` and `.data` keep their sizes, and `boot_check` and `main` keep both their
 * size and their address because this object is linked first. Compare must report those as
 * unchanged, and must not report a row position as a change.
 *
 * Deliberately not product demo data: the numbers are here to be checkable.
 */

/* read-only: contributes to the load image only, identical in both builds */
const unsigned int cfg_table[8] = {1u, 2u, 3u, 4u, 5u, 6u, 7u, 8u};

/* initialized data: image bytes AND live RAM, identical in both builds */
int g_threshold = 0x12345678;

/* zero initialized: RAM only, no image payload */
int g_scratch;

/*
 * A settings blob at a fixed flash address, identical in both builds. The linker script pins its
 * address, so it keeps its size, its VMA and its LMA while the code around it moves. Compare needs
 * a genuinely unchanged section to prove it can say "no change" — every other section in this pair
 * shifts somewhere.
 */
__attribute__((section(".settings"), used)) const unsigned char settings_page[64] = {
    'F',  'W',  'S',  0u,   1u,  0u,  0u,  0u,  0u,  0u,  0u,  0u,   0u,  0u,  0u,  0u,
    0u,   0u,   0u,   0u,   0u,  0u,  0u,  0u,  0u,  0u,  0u,  0u,   0u,  0u,  0u,  0u,
    0u,   0u,   0u,   0u,   0u,  0u,  0u,  0u,  0u,  0u,  0u,  0u,   0u,  0u,  0u,  0u,
    0u,   0u,   0u,   0u,   0u,  0u,  0u,  0u,  0u,  0u,  0u,  0u,   0u,  0u,  0u,  0u};

/*
 * Kept alive from main() so -Os cannot drop it. Its address is fixed by link order, which is what
 * makes it the unchanged-symbol control.
 */
__attribute__((noinline, used)) int boot_check(void) {
    return g_threshold ^ (int)cfg_table[0];
}

int mqtt_task(int seed);
int tls_handshake(int seed);
int payload_extra(int seed);
int calib_apply(int seed);

__attribute__((noinline)) int main_entry(int seed);

__attribute__((noinline)) int main_entry(int seed) {
    int sum = boot_check() + mqtt_task(seed);
    sum += tls_handshake(seed);
    sum += payload_extra(seed);
    sum += calib_apply(seed);
    return sum + g_scratch;
}

int main(void) {
    return main_entry(g_threshold);
}
"""

BASE_PAYLOAD = r"""/*
 * FirmwareSight P2 Compare fixture — the BASE side only.
 *
 * `calib_apply` and the `.calib` blob exist in this build and not in the target, so Compare has a
 * genuine Removed symbol and a genuine Removed section to report. `mqtt_task` is the small version:
 * the target's larger body is what makes it a Changed-size symbol.
 */

__attribute__((section(".calib"), used)) const unsigned char calib[32] = {
    0x10u, 0x11u, 0x12u, 0x13u, 0x14u, 0x15u, 0x16u, 0x17u, 0x18u, 0x19u, 0x1au,
    0x1bu, 0x1cu, 0x1du, 0x1eu, 0x1fu, 0x20u, 0x21u, 0x22u, 0x23u, 0x24u, 0x25u,
    0x26u, 0x27u, 0x28u, 0x29u, 0x2au, 0x2bu, 0x2cu, 0x2du, 0x2eu, 0x2fu};

extern const unsigned char calib[32];
extern int g_scratch;

__attribute__((noinline)) int calib_apply(int seed) {
    return (int)calib[seed & 31] + seed;
}

__attribute__((noinline, used)) int mqtt_task(int seed) {
    return seed + 1;
}

/* Same body as the target's. Its address still moves, because mqtt_task ahead of it grows. */
__attribute__((noinline, used)) int tls_handshake(int seed) {
    return (seed ^ 0xAA) + g_scratch;
}

__attribute__((noinline)) int payload_extra(int seed) {
    return calib_apply(seed);
}
"""

TARGET_PAYLOAD = r"""/*
 * FirmwareSight P2 Compare fixture — the TARGET side only.
 *
 * Three deliberate differences, each visible to one acceptance item of US-002:
 *   `mqtt_task`        — a larger body, so its size grows and `tls_handshake` moves with it.
 *   `packet_router`    — a symbol this build has and the base does not: an Added symbol.
 *   `staging_area`     — a `.ota_staging` input section the target linker script places in RAM,
 *                        loaded from ROM: an Added section that charges BOTH memory budgets.
 * `g_frame_counter` grows `.bss` without touching the load image, which is the runtime-only case.
 * The base's `calib_apply` and `.calib` are absent here, which is the Removed case.
 */

__attribute__((section(".ota_staging"), used)) volatile unsigned char staging_area[64] = {
    0xA0u, 0xA1u, 0xA2u, 0xA3u, 0xA4u, 0xA5u, 0xA6u, 0xA7u, 0xA8u, 0xA9u, 0xAAu,
    0xABu, 0xACu, 0xADu, 0xAEu, 0xAFu, 0xB0u, 0xB1u, 0xB2u, 0xB3u, 0xB4u, 0xB5u,
    0xB6u, 0xB7u, 0xB8u, 0xB9u, 0xBAu, 0xBBu, 0xBCu, 0xBDu, 0xBEu, 0xBFu};

extern volatile unsigned char staging_area[64];
extern int g_scratch;
int g_frame_counter;

__attribute__((noinline)) int calib_apply(int seed) {
    /* No calibration blob in this build: the entry point stays, its data is gone. */
    return seed;
}

__attribute__((noinline, used)) int mqtt_task(int seed) {
    int acc = seed + 1;
    acc = (acc * 3) ^ 0x5A;
    acc += (acc >> 2) - g_scratch;
    acc = (acc * 5) ^ 0x3C;
    return acc + staging_area[3];
}

/* Byte-for-byte the same body as the base's. Only its address differs. */
__attribute__((noinline, used)) int tls_handshake(int seed) {
    return (seed ^ 0xAA) + g_scratch;
}

__attribute__((noinline, used)) int packet_router(int seed) {
    int hop = seed + g_frame_counter;
    hop = (hop * 7) ^ 0x1F;
    hop += (int)staging_area[hop & 63];
    return hop + g_scratch;
}

__attribute__((noinline)) int payload_extra(int seed) {
    return calib_apply(seed) + packet_router(seed);
}
"""

LINKER_SCRIPT = """/*
 * Controlled ROM/RAM layout for the P2 Compare fixture pair.
 *
 * The two sides differ in exactly one rule each: this script declares only the custom section the
 * side it belongs to actually feeds. A rule with no matching input object would still emit an empty
 * output section, and an empty `.calib` in the target would be reported as a shrink rather than as
 * the removal it is.
 *
 * RAM (rwx) is writable, therefore live storage. ROM (rx) is not writable, therefore the load
 * image. `.ota` is VMA-resident in RAM and LMA-stored in ROM, so one section charges both budgets.
 */
MEMORY {{
  ROM (rx)  : ORIGIN = 0x08000000, LENGTH = 1024K
  RAM (rwx) : ORIGIN = 0x20000000, LENGTH = 256K
}}

SECTIONS {{
  .text : {{ *(.text*) }} > ROM

  .rodata : {{ *(.rodata*) }} > ROM

  /*
   * Pinned to a fixed flash address, and fed by identical content on both sides, so this is the
   * section a correct diff must report as unchanged: same size, same VMA, same LMA.
   */
  .settings 0x08010000 : {{ *(.settings*) }} > ROM

  _sidata = LOADADDR(.data);
  .data : {{ *(.data*) }} > RAM AT> ROM

  .bss (NOLOAD) : {{ *(.bss*) *(COMMON) }} > RAM
{extra}
}}
"""

BASE_EXTRA = """
  /* Base only: 32 calibration bytes that the target build no longer carries. */
  .calib : { *(.calib*) } > ROM"""

TARGET_EXTRA = """
  /* Target only: 64 bytes resident in RAM and loaded from ROM. */
  .ota : { *(.ota_staging*) } > RAM AT> ROM"""


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def tool_version(exe: str) -> str:
    out = subprocess.run([exe, "--version"], capture_output=True, text=True, check=True)
    return out.stdout.splitlines()[0].strip()


def run(cmd: list[str], cwd: Path) -> str:
    printable = " ".join(cmd)
    print(f"    $ {printable}")
    result = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)
    if result.returncode != 0:
        sys.exit(f"command failed: {printable}\n{result.stdout}\n{result.stderr}")
    return result.stdout


def write_sources(dirpath: Path, payload: str, extra: str) -> None:
    (dirpath / "source").mkdir(parents=True, exist_ok=True)
    (dirpath / "source" / "common.c").write_text(COMMON_SOURCE, encoding="utf-8", newline="\n")
    (dirpath / "source" / "payload.c").write_text(payload, encoding="utf-8", newline="\n")
    (dirpath / "firmware.ld").write_text(
        LINKER_SCRIPT.format(extra=extra), encoding="utf-8", newline="\n"
    )


def build(dirpath: Path, gcc: str) -> None:
    build_dir = dirpath / "build"
    if build_dir.exists():
        shutil.rmtree(build_dir)
    build_dir.mkdir()

    compile_flags = ["-g", "-Os", "-mcpu=cortex-m4", "-mthumb"]
    for unit in ("common", "payload"):
        run(
            [gcc, *compile_flags, "-c", f"source/{unit}.c", "-o", f"build/{unit}.o"],
            dirpath,
        )

    run(
        [
            gcc,
            "-g",
            "-Os",
            "-mcpu=cortex-m4",
            "-mthumb",
            "-nostdlib",
            "-nostartfiles",
            "-e",
            "main",
            "build/common.o",
            "build/payload.o",
            "-T",
            "firmware.ld",
            "-Wl,-Map=firmware.map",
            "-o",
            "firmware.elf",
        ],
        dirpath,
    )
    shutil.rmtree(build_dir)


def assert_no_absolute_paths(dirpath: Path) -> None:
    for candidate in sorted(dirpath.glob("*.map")):
        for lineno, line in enumerate(
            candidate.read_text(encoding="utf-8", errors="replace").splitlines(), 1
        ):
            for token in line.strip().split():
                if ABSOLUTE_PATH_RE.match(token):
                    sys.exit(
                        f"fixture {candidate.name}:{lineno} contains an absolute path "
                        f"({token!r}); refusing to commit a private machine path"
                    )


def report(dirpath: Path, readelf: str) -> None:
    """Print what the linker actually produced, so the recorded expectations are checked, not assumed."""
    sections = run([readelf, "-S", "-W", "firmware.elf"], dirpath)
    interesting = [
        line.strip()
        for line in sections.splitlines()
        if re.search(r"\s\.(text|rodata|data|bss|calib|ota)\s", line)
    ]
    print("      sections:")
    for line in interesting:
        print(f"        {line}")
    symbols = run([readelf, "--wide", "-s", "firmware.elf"], dirpath)
    print("      defined symbols:")
    for line in symbols.splitlines():
        if re.search(r"\b(GLB|WEA)\b.*\b(DEFINE|GLOBAL)\b|\.text|\.rodata|\.data|\.bss|\.calib|\.ota", line):
            if "UND" not in line:
                print(f"        {line.strip()}")


def fixture_toml(
    *,
    side: str,
    opposite: str,
    gcc: str,
    ld: str,
    files: list[str],
) -> str:
    generated = datetime.date.today().isoformat()
    purpose = (
        "P2 Compare fixture pair, %s side. Authored so that a build-to-build diff has one of each "
        "change kind to report: an unchanged section and symbol, a changed-size section and symbol, "
        "an address-only move, an added section and symbol, a removed section and symbol, and both "
        "memory budgets moving in a known direction with exact MAP-backed evidence." % side
    )
    limitations = [
        "controlled linker script, not a vendor SDK default",
        "no DWARF consumption: the debug sections are present but unread, as in P0",
        "not a real customer firmware image",
        "object and module attribution is absent on purpose, so Compare must report it unavailable",
    ]
    parts = [
        "# Provenance record for a committed P2 Compare fixture. Generated by scripts/gen_p2_fixtures.py.",
        "",
        f'fixture_id = "p2-diff-{side}"',
        f'purpose = "{purpose}"',
        'license = "Written for this project; no third-party code copied"',
        f'generated_on = "{generated}"',
        "",
        "[toolchain]",
        'name = "arm-none-eabi-gcc"',
        f'version = "{tool_version(gcc)}"',
        f'linker = "{tool_version(ld)}"',
        "",
        "[build]",
        "command = '''arm-none-eabi-gcc -g -Os -mcpu=cortex-m4 -mthumb -c source/common.c -o build/common.o"
        " && arm-none-eabi-gcc -g -Os -mcpu=cortex-m4 -mthumb -c source/payload.c -o build/payload.o"
        " && arm-none-eabi-gcc -g -Os -mcpu=cortex-m4 -mthumb -nostdlib -nostartfiles -e main"
        " build/common.o build/payload.o -T firmware.ld -Wl,-Map=firmware.map -o firmware.elf'''",
        'rebuild_script = "scripts/gen_p2_fixtures.py"',
        "",
        "[expectations]",
        'architecture = "ARM (cortex-m4, thumb, 32-bit little-endian)"',
        'expected_capability = "ELF Supported; MAP Supported via gnu_ld adapter"',
        'map_backed_evidence = "exact"',
        f'paired_with = "p2-diff-{opposite}"',
        "",
    ]
    parts += [f'[[known_limitations]]\ndescription = "{item}"\n' for item in limitations]
    parts += [f'[[files]]\npath = "{name}"\n' for name in files]
    return "\n".join(parts)


def update_manifest(entries: list[dict[str, str]]) -> None:
    """Rewrite fixtures/manifest.json keeping every entry that is not a P2 fixture."""
    path = ROOT / "fixtures" / "manifest.json"
    existing = json.loads(path.read_text(encoding="utf-8"))
    kept = [e for e in existing["files"] if not e["path"].startswith("fixtures/elf/p2-diff/")]
    merged = sorted(kept + entries, key=lambda e: e["path"])
    manifest = {
        "schema": existing["schema"],
        "generated_by": existing["generated_by"],
        "generators": [existing["generated_by"], "scripts/gen_p2_fixtures.py"],
        "note": existing["note"],
        "files": merged,
    }
    path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8", newline="\n")
    added = len(merged) - len(kept)
    print(f"\nmanifest: {len(kept)} existing entries kept, {added} P2 entries recorded")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--force", action="store_true", help="overwrite existing fixture binaries")
    args = parser.parse_args()

    gcc = shutil.which("arm-none-eabi-gcc")
    if gcc is None:
        sys.exit(
            "arm-none-eabi-gcc not found. The diff, storage and parser tests do NOT require it; it is "
            "only needed to regenerate this pair. The committed binaries remain valid."
        )
    readelf = gcc.replace("gcc", "readelf")
    ld = gcc.replace("gcc", "ld")

    sides = [
        ("base", "target", BASE_PAYLOAD, BASE_EXTRA),
        ("target", "base", TARGET_PAYLOAD, TARGET_EXTRA),
    ]

    manifest_entries: list[dict[str, str]] = []
    for side, opposite, payload, extra in sides:
        dirpath = PAIR / side
        print(f"[fixture] p2-diff/{side}")
        dirpath.mkdir(parents=True, exist_ok=True)
        write_sources(dirpath, payload, extra)
        elf = dirpath / "firmware.elf"
        if elf.exists() and not args.force:
            sys.exit(f"{elf} already exists; pass --force to regenerate and re-record hashes")
        build(dirpath, gcc)
        assert_no_absolute_paths(dirpath)
        report(dirpath, readelf)

        names = [
            p.relative_to(dirpath).as_posix()
            for p in sorted(dirpath.rglob("*"))
            if p.is_file() and p.name != "fixture.toml"
        ]
        (dirpath / "fixture.toml").write_text(
            fixture_toml(side=side, opposite=opposite, gcc=gcc, ld=ld, files=names),
            encoding="utf-8",
            newline="\n",
        )
        for name in names + ["fixture.toml"]:
            manifest_entries.append(
                {
                    "path": (dirpath / name).relative_to(ROOT).as_posix(),
                    "sha256": sha256(dirpath / name),
                    "fixture": f"p2-diff-{side}",
                    "purpose": "elf-binary"
                    if name.endswith(".elf")
                    else "p2-diff-support",
                }
            )

    update_manifest(manifest_entries)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
