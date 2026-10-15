"""Generate the M2 object/library attribution fixtures (ADR-0033).

Two builds of one small Cortex-M4 project, `base` and `target`, each with two application objects and a
static library, compiled with -ffunction-sections and linked with --gc-sections, so the GNU ld MAP carries
the row shapes ADR-0033 D1 reads: wrapped input-section rows, an archive member, a COMMON symbol, padding
and a discarded-sections block. `target` grows the IMU code and links one more library member, so Compare
has a per-object and per-library delta to report.

All paths handed to the toolchain are relative, because GNU ld copies them into the MAP and a committed
fixture must not carry a private machine path. The script refuses to write one.

    python scripts/gen_m2_object_fixtures.py

Requires arm-none-eabi-gcc, -ar, -ld and -readelf on PATH (Arm GNU Toolchain 14.3.Rel1 was used).
"""

from __future__ import annotations

import hashlib
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "fixtures" / "elf" / "m2-objects"
ABSOLUTE_PATH_RE = re.compile(r"^(?:[A-Za-z]:[\\/]|/(?:home|Users|tmp|opt|usr|mnt|var)/|\\\\)")

LINKER_SCRIPT = """\
MEMORY
{
  FLASH (rx)  : ORIGIN = 0x08000000, LENGTH = 64K
  RAM   (rwx) : ORIGIN = 0x20000000, LENGTH = 16K
}

ENTRY(Reset_Handler)

SECTIONS
{
  .text :
  {
    KEEP(*(.text.Reset_Handler))
    . = ALIGN(64);
    *(.text*)
    *(.rodata*)
    . = ALIGN(16);
  } > FLASH

  .data :
  {
    *(.data*)
  } > RAM AT > FLASH

  .bss (NOLOAD) :
  {
    *(.bss*)
    *(COMMON)
  } > RAM
}
"""

MAIN_C = """\
/* Written for this project; no third-party code copied. */
extern int imu_read(int axis);
extern void uart_send(int byte);
{extra_decl}
int g_samples;                 /* uninitialized: placed by -fcommon as a COMMON symbol */
static volatile int g_threshold = 42;   /* initialized: .data, charged to Flash and RAM */

void Reset_Handler(void)
{{
    for (int i = 0; i < 3; i++) {{
        int v = imu_read(i);
        if (v > g_threshold) {{
            uart_send(v);
        }}
        g_samples += v;
    }}
{extra_call}
    for (;;) {{
    }}
}}

int unused_helper(int x)       /* never referenced: discarded by --gc-sections */
{{
    return x * 3;
}}
"""

IMU_C = """\
/* Written for this project; no third-party code copied. */
static const short imu_calibration[{count}] = {{ {values} }};
static int imu_history[{count}];

int imu_read_with_a_name_long_enough_to_make_gnu_ld_wrap_the_input_section_row(int axis)
{{
    int value = imu_calibration[axis % {count}] * 3;
    imu_history[axis % {count}] = value;
    return value;
}}

int imu_read(int axis)
{{
    return imu_read_with_a_name_long_enough_to_make_gnu_ld_wrap_the_input_section_row(axis);
}}
"""

UART_C = """\
/* Written for this project; no third-party code copied. */
static volatile int uart_tx;

void uart_send(int byte)
{
    uart_tx = byte;
}
"""

SPI_C = """\
/* Written for this project; no third-party code copied. */
static volatile int spi_buffer[8];

void spi_flush(void)
{
    for (int i = 0; i < 8; i++) {
        spi_buffer[i] = 0;
    }
}
"""

BUILDS = {
    "base": {"imu_count": 4, "extra_decl": "", "extra_call": ""},
    "target": {"imu_count": 24, "extra_decl": "extern void spi_flush(void);", "extra_call": "    spi_flush();"},
}

CFLAGS = ["-mcpu=cortex-m4", "-mthumb", "-Os", "-ffunction-sections", "-fdata-sections"]


def run(cmd: list[str], cwd: Path) -> str:
    result = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, check=False)
    if result.returncode != 0:
        sys.exit(f"command failed: {' '.join(cmd)}\n{result.stdout}{result.stderr}")
    return result.stdout


def version(exe: str) -> str:
    return run([exe, "--version"], ROOT).splitlines()[0].strip()


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8", newline="\n")


def assert_no_host_paths(path: Path) -> None:
    for lineno, line in enumerate(path.read_text(encoding="utf-8", errors="replace").splitlines(), 1):
        for token in line.split():
            if ABSOLUTE_PATH_RE.match(token):
                sys.exit(f"{path.relative_to(ROOT)}:{lineno} carries an absolute path ({token!r}); refusing")


def build(name: str, spec: dict) -> list[str]:
    directory = OUT / name
    if directory.exists():
        shutil.rmtree(directory)
    count = spec["imu_count"]
    values = ", ".join(str((i * 37) % 101) for i in range(count))
    write(directory / "objects.ld", LINKER_SCRIPT)
    write(directory / "source" / "app" / "main.c", MAIN_C.format(**spec))
    write(directory / "source" / "app" / "sensors" / "imu.c", IMU_C.format(count=count, values=values))
    write(directory / "source" / "drv" / "uart.c", UART_C)
    write(directory / "source" / "drv" / "spi.c", SPI_C)

    commands = [
        ["arm-none-eabi-gcc", *CFLAGS, "-fcommon", "-c", "source/app/main.c", "-o", "build/app/main.o"],
        ["arm-none-eabi-gcc", *CFLAGS, "-c", "source/app/sensors/imu.c", "-o", "build/app/sensors/imu.o"],
        ["arm-none-eabi-gcc", *CFLAGS, "-c", "source/drv/uart.c", "-o", "build/drv/uart.o"],
        ["arm-none-eabi-gcc", *CFLAGS, "-c", "source/drv/spi.c", "-o", "build/drv/spi.o"],
        ["arm-none-eabi-ar", "rcsD", "build/libdrv.a", "build/drv/uart.o", "build/drv/spi.o"],
        [
            "arm-none-eabi-ld", "--gc-sections", "-Tobjects.ld",
            "build/app/main.o", "build/app/sensors/imu.o", "build/libdrv.a",
            "-o", "firmware.elf", "-Map=firmware.map",
        ],
    ]
    for sub in ("build/app/sensors", "build/drv"):
        (directory / sub).mkdir(parents=True, exist_ok=True)
    for cmd in commands:
        run(cmd, directory)
    shutil.rmtree(directory / "build")

    for text_file in (directory / "firmware.map", directory / "objects.ld"):
        assert_no_host_paths(text_file)
    map_text = (directory / "firmware.map").read_text(encoding="utf-8")
    for required in ("Discarded input sections", "libdrv.a(uart.o)", "COMMON", "*fill*"):
        if required not in map_text:
            sys.exit(f"{name}: the MAP lacks {required!r}, so it does not exercise ADR-0033 D1")

    toml = [
        "# Provenance record for an M2 attribution fixture (ADR-0033).",
        "# Generated by scripts/gen_m2_object_fixtures.py; the ELF and MAP are tool output.",
        "",
        f'fixture_id = "m2-objects-{name}"',
        'purpose = "Object and library attribution: wrapped input-section rows, an archive member, COMMON, '
        'padding and a discarded block, with relative paths only."',
        'license = "Written for this project; no third-party code copied"',
        "",
        "[toolchain]",
        f'compiler = """{version("arm-none-eabi-gcc")}"""',
        f'linker = """{version("arm-none-eabi-ld")}"""',
        "",
        "[build]",
        'rebuild_script = "scripts/gen_m2_object_fixtures.py"',
        "commands = [",
        *[f'  """{" ".join(cmd)}""",' for cmd in commands],
        "]",
        "",
    ]
    write(directory / "fixture.toml", "\n".join(toml))
    return [p.relative_to(ROOT).as_posix() for p in sorted(directory.rglob("*")) if p.is_file()]


def update_manifest(paths: list[str]) -> None:
    path = ROOT / "fixtures" / "manifest.json"
    existing = json.loads(path.read_text(encoding="utf-8"))
    kept = [e for e in existing["files"] if not e["path"].startswith("fixtures/elf/m2-objects/")]
    entries = []
    for relative in paths:
        fixture = "m2-objects-" + relative.split("/")[3]
        purpose = "fixture-artifact" if relative.endswith((".elf", ".map")) else "fixture-support"
        entries.append({"path": relative, "sha256": sha256(ROOT / relative), "fixture": fixture, "purpose": purpose})
    generators = list(existing.get("generators", [existing["generated_by"]]))
    mine = "scripts/gen_m2_object_fixtures.py"
    manifest = {
        "schema": existing["schema"],
        "generated_by": existing["generated_by"],
        "generators": generators if mine in generators else [*generators, mine],
        "note": existing["note"],
        "files": sorted(kept + entries, key=lambda e: e["path"]),
    }
    path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8", newline="\n")
    print(f"manifest: {len(kept)} prior entries kept, {len(entries)} M2 entries recorded")


def main() -> int:
    paths: list[str] = []
    for name, spec in BUILDS.items():
        paths.extend(build(name, spec))
    update_manifest(paths)
    return 0


if __name__ == "__main__":
    sys.exit(main())
