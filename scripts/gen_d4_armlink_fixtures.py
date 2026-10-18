"""Generate the D4 armlink samples: a same-origin ArmClang/armlink set (06_DELIVERY/14).

The same small Cortex-M4 project as `gen_m2_object_fixtures.py` (two application objects and a static library;
`target` grows the IMU table and links one more library member), built with Arm Compiler for Embedded 6 and
armlink instead of GCC and GNU ld. Each build writes, from one run of one toolchain:

- `firmware.axf`   the ELF image;
- `firmware.map`   armlink `--map --list` output with `--info=sizes,totals,unused` and `--symbols`;
- `objects.sct`    the scatter file that sets the regions;
- `source/`        the sources, written for this project;
- `fixture.toml`   provenance: the exact tool versions and every command line.

`--no_scanlib` keeps the Arm C library out of the image, so every byte in it comes from the sources here and
the files can be committed. The image is a shaped sample for reading, not a program to flash: without the
library there is no scatter-loading code to copy `.data` or zero `.bss`.

All paths handed to the toolchain are relative, because armlink copies them into the MAP and a committed
fixture must not carry a private machine path. The script refuses to write one.

This is D4 *preparation*. FirmwareSight does not read armlink MAP files; nothing here is a support claim
(Standing Owner rule 3). `scripts/crosscheck_d4_armlink.py` records what independent tools say about the
images.

    python scripts/gen_d4_armlink_fixtures.py [--armclang-bin DIR]

Requires armclang, armlink and armar (Arm Compiler for Embedded 6; 6.24 from MDK 5.43 was used). The default
DIR is C:/Keil_v5/ARM/ARMCLANG/bin.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "fixtures" / "elf" / "d4-armlink"
ABSOLUTE_PATH_RE = re.compile(r"(?:[A-Za-z]:[\\/]|/(?:home|Users|tmp|opt|usr|mnt|var)/|\\\\)")

SCATTER = """\
; Regions of the D4 armlink sample: the same 64K Flash and 16K RAM as the M2 GNU ld fixture.
LR_IROM1 0x08000000 0x00010000
{
  ER_IROM1 0x08000000 0x00010000
  {
    *(+RO)
  }
  RW_IRAM1 0x20000000 0x00004000
  {
    *(+RW +ZI)
  }
}
"""

MAIN_C = """\
/* Written for this project; no third-party code copied. */
extern int imu_read(int axis);
extern void uart_send(int byte);
{extra_decl}
int g_samples;                 /* uninitialized: a common symbol with -fcommon */
static volatile int g_threshold = 42;   /* initialized: RW data, charged to Flash and RAM */

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

int unused_helper(int x)       /* never referenced: removed by armlink --remove */
{{
    return x * 3;
}}
"""

IMU_C = """\
/* Written for this project; no third-party code copied. */
static const short imu_calibration[{count}] = {{ {values} }};
static int imu_history[{count}];

int imu_read_with_a_name_long_enough_to_make_a_linker_wrap_its_input_section_row(int axis)
{{
    int value = imu_calibration[axis % {count}] * 3;
    imu_history[axis % {count}] = value;
    return value;
}}

int imu_read(int axis)
{{
    return imu_read_with_a_name_long_enough_to_make_a_linker_wrap_its_input_section_row(axis);
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

CFLAGS = ["--target=arm-arm-none-eabi", "-mcpu=cortex-m4", "-mthumb", "-Os", "-ffunction-sections", "-fdata-sections"]


def run(cmd: list[str], cwd: Path) -> str:
    result = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, check=False)
    if result.returncode != 0:
        sys.exit(f"command failed: {' '.join(cmd)}\n{result.stdout}{result.stderr}")
    return result.stdout + result.stderr


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8", newline="\n")


def assert_no_host_paths(path: Path) -> None:
    for lineno, line in enumerate(path.read_text(encoding="utf-8", errors="replace").splitlines(), 1):
        if ABSOLUTE_PATH_RE.search(line):
            sys.exit(f"{path.relative_to(ROOT)}:{lineno} carries an absolute path; refusing: {line.strip()!r}")


def build(name: str, spec: dict, tools: dict[str, str]) -> list[str]:
    directory = OUT / name
    if directory.exists():
        shutil.rmtree(directory)
    count = spec["imu_count"]
    values = ", ".join(str((i * 37) % 101) for i in range(count))
    write(directory / "objects.sct", SCATTER)
    write(directory / "source" / "app" / "main.c", MAIN_C.format(**spec))
    write(directory / "source" / "app" / "sensors" / "imu.c", IMU_C.format(count=count, values=values))
    write(directory / "source" / "drv" / "uart.c", UART_C)
    write(directory / "source" / "drv" / "spi.c", SPI_C)

    cc, ar, ld = tools["armclang"], tools["armar"], tools["armlink"]
    members = ["build/libdrv.lib(uart.o)"]
    if spec["extra_call"]:
        members.append("build/libdrv.lib(spi.o)")
    commands = [
        [cc, *CFLAGS, "-fcommon", "-c", "source/app/main.c", "-o", "build/app/main.o"],
        [cc, *CFLAGS, "-c", "source/app/sensors/imu.c", "-o", "build/app/sensors/imu.o"],
        [cc, *CFLAGS, "-c", "source/drv/uart.c", "-o", "build/drv/uart.o"],
        [cc, *CFLAGS, "-c", "source/drv/spi.c", "-o", "build/drv/spi.o"],
        [ar, "--create", "build/libdrv.lib", "build/drv/uart.o", "build/drv/spi.o"],
        [
            ld, "--cpu=Cortex-M4", "--scatter=objects.sct", "--entry=Reset_Handler", "--no_scanlib", "--remove",
            "--map", "--symbols", "--info=sizes,totals,unused", "--list=firmware.map",
            "build/app/main.o", "build/app/sensors/imu.o", *members,
            "-o", "firmware.axf",
        ],
    ]
    for sub in ("build/app/sensors", "build/drv"):
        (directory / sub).mkdir(parents=True, exist_ok=True)
    for cmd in commands:
        run(cmd, directory)
    shutil.rmtree(directory / "build")

    for text_file in (directory / "firmware.map", directory / "objects.sct"):
        assert_no_host_paths(text_file)
    map_text = (directory / "firmware.map").read_text(encoding="utf-8", errors="replace")
    for required in ("Memory Map of the image", "Image component sizes", "libdrv.lib", "Grand Totals"):
        if required not in map_text:
            sys.exit(f"{name}: the armlink listing lacks {required!r}")

    shown = [[Path(cmd[0]).name, *cmd[1:]] for cmd in commands]
    toml = [
        "# Provenance record for a D4 armlink sample (06_DELIVERY/14). Not a support claim.",
        "# Generated by scripts/gen_d4_armlink_fixtures.py; the AXF and MAP are tool output.",
        "",
        f'fixture_id = "d4-armlink-{name}"',
        'purpose = "Same-origin ArmClang/armlink set for the D4 armlink adapter: AXF, MAP listing with sizes and '
        'totals, scatter file, sources and exact tool versions."',
        'license = "Written for this project; no third-party code copied; --no_scanlib keeps the Arm C library out (an engineering note, not legal advice; the Owner confirms before any public release)"',
        "",
        "[toolchain]",
        f'armclang = """{tools["armclang_version"]}"""',
        f'armlink = """{tools["armlink_version"]}"""',
        "",
        "[build]",
        'rebuild_script = "scripts/gen_d4_armlink_fixtures.py"',
        "commands = [",
        *[f'  """{" ".join(cmd)}""",' for cmd in shown],
        "]",
        "",
    ]
    write(directory / "fixture.toml", "\n".join(toml))
    return [p.relative_to(ROOT).as_posix() for p in sorted(directory.rglob("*")) if p.is_file()]


def update_manifest(paths: list[str]) -> None:
    path = ROOT / "fixtures" / "manifest.json"
    existing = json.loads(path.read_text(encoding="utf-8"))
    kept = [e for e in existing["files"] if not e["path"].startswith("fixtures/elf/d4-armlink/")]
    entries = []
    for relative in paths:
        fixture = "d4-armlink-" + relative.split("/")[3]
        purpose = "fixture-artifact" if relative.endswith((".axf", ".map")) else "fixture-support"
        entries.append({"path": relative, "sha256": sha256(ROOT / relative), "fixture": fixture, "purpose": purpose})
    generators = list(existing.get("generators", [existing["generated_by"]]))
    mine = "scripts/gen_d4_armlink_fixtures.py"
    manifest = {
        "schema": existing["schema"],
        "generated_by": existing["generated_by"],
        "generators": generators if mine in generators else [*generators, mine],
        "note": existing["note"],
        "files": sorted(kept + entries, key=lambda e: e["path"]),
    }
    path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8", newline="\n")
    print(f"manifest: {len(kept)} prior entries kept, {len(entries)} D4 entries recorded")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--armclang-bin", default="C:/Keil_v5/ARM/ARMCLANG/bin")
    args = parser.parse_args()
    bin_dir = Path(args.armclang_bin)
    tools = {name: str(bin_dir / f"{name}.exe") for name in ("armclang", "armlink", "armar")}
    for name, exe in tools.items():
        if not Path(exe).exists():
            sys.exit(f"{name} not found at {exe}")
    tools["armclang_version"] = " ".join(run([tools["armclang"], "--version"], ROOT).split())
    tools["armlink_version"] = " ".join(run([tools["armlink"], "--vsn"], ROOT).split())
    paths: list[str] = []
    for name, spec in BUILDS.items():
        paths.extend(build(name, spec, tools))
    update_manifest(paths)
    return 0


if __name__ == "__main__":
    sys.exit(main())
