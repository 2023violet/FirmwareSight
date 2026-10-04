#!/usr/bin/env python3
"""Generate the P5 Commit E compatibility fixtures and their provenance records.

Why these exist: the committed cohort was produced entirely by `arm-none-eabi-gcc` + GNU ld, which
left the Clang half of the supported-cohort claim with no compiler-produced evidence and left most of
the eight layout cases the product claims to handle unproven. Every file this script writes is real
tool output. Nothing here edits a byte of a produced artifact: the only post-link step is
`objcopy -S`, a binutils operation the prompt names, not a patch.

Provenance discipline, inherited from scripts/gen_p0_fixtures.py and kept:

* compile and link with RELATIVE paths, because GNU ld writes the object paths it was given into the
  MAP, and an absolute build path would put a private machine path into committed evidence;
* record the compiler and the linker as two separate identities, and prove the compiler one from the
  artifact's own bytes by reading the `.comment` section the compiler wrote into the ELF;
* keep the binaries committed so the parser and CLI tests never need a toolchain installed;
* remove every temporary build file, so a fixture directory holds evidence and nothing else.

What this file does NOT record: what FirmwareSight concludes about a fixture. The `[layout]` block
states addresses and region names, which `readelf` and the MAP confirm, and points at the test that
carries the product claim. Accounting verdicts are measured from the bytes after generation, never
written here in advance of a run.

Usage:  python scripts/gen_p5_compat_fixtures.py [--force]

Requires arm-none-eabi-gcc, arm-none-eabi-ld, arm-none-eabi-objcopy, arm-none-eabi-readelf, and for
the clang-arm fixture a clang that can target arm-none-eabi. Regenerating is the only time any of
them is needed.
"""

from __future__ import annotations

import argparse
import datetime
import hashlib
import re
import json
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PARENT = ROOT / "fixtures" / "elf" / "p5-compat"

# The shape gen_p0_fixtures.py:32 guards with, widened to every textual artifact this writes.
ABSOLUTE_PATH_RE = re.compile(r"^[A-Za-z]:[\\/]|^/Users/|^/home/|^/tmp/")

# Case E is only evidence while the banner sits far enough in. If a future toolchain shortens the
# discarded-section block below this, the fixture stops proving the MAP-reader case it exists for, so
# the generator refuses rather than committing a file that no longer carries its own claim.
MIN_BANNER_OFFSET = 4096
BANNER = "Memory Configuration"


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def first_version_line(exe: str) -> str:
    out = subprocess.run([exe, "--version"], capture_output=True, text=True, check=True)
    return out.stdout.splitlines()[0].strip()


def run(cmd: list[str], cwd: Path) -> None:
    printable = " ".join(cmd)
    print(f"    $ {printable}")
    result = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)
    if result.returncode != 0:
        sys.exit(f"command failed: {printable}\n{result.stdout}\n{result.stderr}")


def comment_of(readelf: str, elf: Path) -> str:
    """The compiler identity the artifact carries in its own bytes.

    This is what a relabelled fixture cannot survive: a GCC-built ELF names GCC here whatever its
    provenance record claims, and `p5_compat_fixtures.rs` reads the string back out of the file.
    """
    out = subprocess.run(
        [readelf, "-p", ".comment", str(elf)], capture_output=True, text=True, check=False
    )
    if out.returncode != 0:
        return ""
    entries = [line.split("]", 1)[1].strip() for line in out.stdout.splitlines() if line.lstrip().startswith("[")]
    return " | ".join(part for part in entries if part)


SECTION_ROW_RE = re.compile(
    r"^\s*\[\s*\d+\]\s+(?P<name>\S*)\s+(?P<type>\S+)\s+(?P<addr>[0-9a-f]+)\s+(?P<off>[0-9a-f]+)"
    r"\s+(?P<size>[0-9a-f]+)\s+(?P<es>\d+)\s*(?P<flags>[A-Z]*)\s+\d+\s+\d+\s+\d+\s*$"
)


def section_table(readelf: str, elf: Path) -> list[tuple[str, str, int, int, str]]:
    """(name, type, address, size, flags) for every named section the linker left in the file.

    Parsed with a regex rather than by column index because `readelf -S` prints an empty `Flg` column
    for non-allocated sections, which shifts a naive split, and prints the header index as `[ 2]`,
    which splits into two tokens. Both were measured on this toolchain before this function was.
    """
    out = subprocess.run([readelf, "-S", "-W", str(elf)], capture_output=True, text=True, check=True)
    rows: list[tuple[str, str, int, int, str]] = []
    for line in out.stdout.splitlines():
        match = SECTION_ROW_RE.match(line)
        if match and match["name"]:
            rows.append(
                (
                    match["name"],
                    match["type"],
                    int(match["addr"], 16),
                    int(match["size"], 16),
                    match["flags"],
                )
            )
    return rows


MAP_REGION_RE = re.compile(
    r"^(?P<name>\S+)\s+0x(?P<origin>[0-9a-f]+)\s+0x(?P<length>[0-9a-f]+)\s+(?P<attrs>[a-z]+)$"
)


def map_regions(map_path: Path) -> list[tuple[str, int, int, str]]:
    """The linker's own `Memory Configuration` table: (name, origin, length, attribute string).

    This is where the recipe learns which declared regions are writable, read from the MAP rather
    than from the product, so the expectation it produces is arithmetic on tool output and not a copy
    of the answer being tested. The `*default*` row is skipped: it spans the whole address space, so
    counting it would make every section look at home everywhere.
    """
    text = map_path.read_text(encoding="utf-8", errors="replace")
    start = text.find(BANNER)
    if start < 0:
        sys.exit(f"{map_path} carries no '{BANNER}' table; cannot derive region attributes")
    rows: list[tuple[str, int, int, str]] = []
    for line in text[start:].splitlines()[1:]:
        stripped = line.strip()
        if not stripped:
            # GNU ld puts a blank line between the banner and the column header, and another after
            # the table. Only the second one ends the region list.
            if rows:
                break
            continue
        match = MAP_REGION_RE.match(stripped)
        if match:
            rows.append(
                (match["name"], int(match["origin"], 16), int(match["length"], 16), match["attrs"])
            )
    if not rows:
        sys.exit(f"{map_path}: the '{BANNER}' table parsed as empty, which is not a fact to record")
    return rows


def entry_point(readelf: str, elf: Path) -> int:
    out = subprocess.run([readelf, "-h", str(elf)], capture_output=True, text=True, check=True)
    for line in out.stdout.splitlines():
        if "Entry point address" in line:
            return int(line.split(":")[-1].strip(), 16)
    sys.exit(f"{elf}: readelf -h reported no entry point address")


def expected_totals(
    elf: Path, rows: list[tuple[str, str, int, int, str]], regions: list[tuple[str, int, int, str]]
) -> tuple[str, str]:
    """(image bytes, live RAM bytes) from this file's section table and the MAP's region attributes.

    The rule is `04_TECH/23_MEMORY_ACCOUNTING_MODEL.md` §3 for what each section class costs and §4
    item 3 for what may decide it: an allocated section occupies the image unless the file stores no
    payload for it (`SHT_NOBITS`, which is `.bss`), and it occupies live storage where the region its
    VMA falls in is declared writable. Section names take part in neither half, which is what makes the
    recorded expectation a check on the product instead of a restatement of it.
    """
    image, ram, image_parts, ram_parts = 0, 0, [], []
    for name, sect_type, addr, size, flags in rows:
        if "A" not in flags:
            continue
        home = [r for r in regions if r[1] <= addr < r[1] + r[2]]
        if not home:
            sys.exit(
                f"{elf}: allocated section {name} at 0x{addr:08x} falls inside no region the MAP "
                "declares; that is a layout finding, not a number to record"
            )
        if sect_type != "NOBITS":
            image += size
            image_parts.append(f"{size} {name}")
        if any("w" in r[3] for r in home):
            ram += size
            ram_parts.append(f"{size} {name}")

    def render(total: int, parts: list[str]) -> str:
        return f"{total} = " + (" + ".join(parts) if parts else "none")

    return render(image, image_parts), render(ram, ram_parts)


def load_segments(readelf: str, elf: Path) -> int:
    out = subprocess.run([readelf, "-l", str(elf)], capture_output=True, text=True, check=True)
    return sum(1 for line in out.stdout.splitlines() if line.strip().startswith("LOAD"))


def banner_offset(map_path: Path) -> int:
    return map_path.read_text(encoding="utf-8", errors="replace").find(BANNER)


# ---- sources -------------------------------------------------------------------------------------
# One source per case, so every allocatable section in a committed ELF is one the fixture's own
# linker script names. A shared source would leave the extra sections to GNU ld's orphan-placement
# rules, and then a reader could not tell the script's decision from the linker's guess.

_PLAIN_BODY = """/* read-only: load image only */
const unsigned int cfg_table[8] = {1u, 2u, 3u, 4u, 5u, 6u, 7u, 8u};

/* initialized data: image bytes AND live RAM */
int g_threshold = 0x12345678;

/* zero initialized: RAM only, no image payload */
int g_scratch;
"""


def _source(purpose: str, placed: str, ret: str) -> str:
    return f"""/*
 * FirmwareSight P5 compatibility fixture source. Written for this project.
 *
 * {purpose}
 */

{_PLAIN_BODY}
{placed}
int main(void) {{
    return {ret};
}}
"""


# Case B: code that runs out of writable RAM, loaded from the image.
RAM_EXEC_C = _source(
    "One section, `.fast_text`, is executable and resident in RAM while its load address stays in FLASH, "
    "which is the shape that makes VMA and LMA disagree for code.",
    '__attribute__((section(".fast_text"), used)) int ram_resident_fn(int x) {\n'
    "    return x + g_threshold;\n"
    "}",
    "ram_resident_fn((int)cfg_table[1]) + g_scratch",
)

# Case C: a live pool in a third, external region.
EXTSRAM_C = _source(
    "`.extsram_pool` is placed in a region above every internal-RAM address, so the region is knowable "
    "only from the linker's own memory map.",
    '__attribute__((section(".extsram_pool"), used)) unsigned int ext_pool[32];',
    "(int)ext_pool[0] + g_scratch",
)

# Case D: a custom-named writable region holding a buffer.
DMA_REGION_C = _source(
    "`.dma_buffer` is placed in a region named for its use rather than for any name the product "
    "recognises. Nothing here claims anything about caches, because a linker script cannot.",
    '__attribute__((section(".dma_buffer"), used)) unsigned int dma_ring[16];',
    "(int)dma_ring[0] + g_scratch",
)

# The reference shapes: plain placement, varying only in what the artifact carries.
PLAIN_C = _source(
    "Plain FLASH and RAM placement, used where the case under test is the compiler or the presence of "
    "debug information rather than the layout.",
    "",
    "(int)cfg_table[0] + g_scratch",
)


def discarded_c(count: int) -> str:
    """Real dead code, so a real linker discards real sections under `--gc-sections`.

    Nothing is padded and nothing is pasted. With `-ffunction-sections` each unreferenced function
    becomes its own `.text.<name>` input section, and `--gc-sections` drops every one unreachable
    from the entry point into the head of the MAP — the same block a large vendor-HAL build produces
    naturally, and the one a fixed-size reader used to miss (post-G2 finding F003). `used` keeps the
    compiler from deleting the functions before the linker sees them, so the discarding stays the
    linker's own act.
    """
    bodies = "\n".join(
        f"__attribute__((used)) int never_called_{i}(int x) {{ return x + {i}; }}" for i in range(count)
    )
    return f"""/*
 * FirmwareSight P5 compatibility fixture source: a long discarded-input-sections preamble.
 *
 * Written for this project. Every function below is genuinely unreferenced from `main`.
 */

{bodies}

int main(void) {{
    return 0;
}}
"""


# ---- linker scripts ------------------------------------------------------------------------------

SCRIPTS = {
    # Case B: code whose runtime address is writable memory, loaded from the image.
    "ram-exec.ld": """/*
 * Case B, executable resident in RAM. `.fast_text` runs from RAM and is stored in FLASH, so VMA and
 * LMA disagree for a section holding code. A rule that equates `.text` with the nonvolatile side
 * charges this section wrongly on purpose-built evidence.
 */
MEMORY {
  FLASH (rx)  : ORIGIN = 0x08000000, LENGTH = 256K
  RAM   (rwx) : ORIGIN = 0x20000000, LENGTH = 64K
}
SECTIONS {
  .text      : { *(.text*) } > FLASH
  .rodata    : { *(.rodata*) } > FLASH
  _sidata = LOADADDR(.data);
  .data      : { *(.data*) } > RAM AT> FLASH
  .fast_text : { *(.fast_text*) } > RAM AT> FLASH
  .bss (NOLOAD) : { *(.bss*) *(COMMON) } > RAM
}
""",
    # Case C: a third writable region, known only from the MAP's region table.
    "extsram.ld": """/*
 * Case C, external SRAM. EXTSRAM is a distinct declared region whose address is outside every
 * internal-RAM assumption, so the product can only know it from the MAP Memory Configuration table.
 */
MEMORY {
  FLASH   (rx)   : ORIGIN = 0x08000000, LENGTH = 256K
  RAM     (rwx)  : ORIGIN = 0x20000000, LENGTH = 64K
  EXTSRAM (rwx)  : ORIGIN = 0x60000000, LENGTH = 512K
}
SECTIONS {
  .text         : { *(.text*) } > FLASH
  .rodata       : { *(.rodata*) } > FLASH
  _sidata = LOADADDR(.data);
  .data         : { *(.data*) } > RAM AT> FLASH
  .extsram_pool : { *(.extsram_pool*) } > EXTSRAM AT> FLASH
  .bss (NOLOAD) : { *(.bss*) *(COMMON) } > RAM
}
""",
    # Case D: a custom-named writable region. No cacheability claim anywhere.
    "dma-region.ld": """/*
 * Case D, a custom writable region holding a buffer pool. The claim under test is narrow: an
 * unfamiliar region name must not make its memory disappear. Nothing in this file says anything
 * about caches, because a linker script cannot.
 */
MEMORY {
  FLASH  (rx)  : ORIGIN = 0x08000000, LENGTH = 256K
  RAM    (rwx) : ORIGIN = 0x20000000, LENGTH = 64K
  DMARAM (rw)  : ORIGIN = 0x30000000, LENGTH = 16K
}
SECTIONS {
  .text       : { *(.text*) } > FLASH
  .rodata     : { *(.rodata*) } > FLASH
  _sidata = LOADADDR(.data);
  .data       : { *(.data*) } > RAM AT> FLASH
  .dma_buffer : { *(.dma_buffer*) } > DMARAM AT> FLASH
  .bss (NOLOAD) : { *(.bss*) *(COMMON) } > RAM
}
""",
    # The remaining fixtures vary in what the artifact carries, not in where sections live.
    "plain.ld": """/*
 * Plain FLASH/RAM split, shared by the debug/no-debug pair, the Clang fixture and the long-preamble
 * fixture, so that the compiler or the debug presence is the only variable between them and a
 * baseline.
 */
MEMORY {
  FLASH (rx)  : ORIGIN = 0x08000000, LENGTH = 256K
  RAM   (rwx) : ORIGIN = 0x20000000, LENGTH = 64K
}
SECTIONS {
  .text   : { *(.text*) } > FLASH
  .rodata : { *(.rodata*) } > FLASH
  _sidata = LOADADDR(.data);
  .data   : { *(.data*) } > RAM AT> FLASH
  .bss (NOLOAD) : { *(.bss*) *(COMMON) } > RAM
}
""",
}

# ---- build helpers -------------------------------------------------------------------------------


def prepare(dirpath: Path, source_text: str, script_name: str) -> None:
    (dirpath / "source").mkdir(parents=True, exist_ok=True)
    (dirpath / "source" / "main.c").write_text(source_text, encoding="utf-8", newline="\n")
    (dirpath / script_name).write_text(SCRIPTS[script_name], encoding="utf-8", newline="\n")
    build = dirpath / "build"
    if build.exists():
        shutil.rmtree(build)
    build.mkdir()


def compile_with(cc: str, dirpath: Path, obj: str, extra: list[str], clang: bool) -> str:
    out = f"build/{obj}.o"
    front = ["--target=arm-none-eabi", "-ffreestanding", "-fno-builtin"] if clang else []
    run([cc, *front, "-mcpu=cortex-m4", "-mthumb", *extra, "-c", "source/main.c", "-o", out], dirpath)
    return out


def link_with_gnu_ld(
    dirpath: Path, ld: str, objects: list[str], script: str, elf: str, map_name: str | None, extra: list[str]
) -> None:
    cmd = [ld, *extra, "-e", "main", f"-T{script}", *objects, "-o", elf]
    if map_name:
        cmd.append(f"-Map={map_name}")
    run(cmd, dirpath)


def assert_no_host_paths(dirpath: Path) -> None:
    """Every textual artifact in a committed fixture must be free of a private machine path."""
    for candidate in sorted(dirpath.rglob("*")):
        if not candidate.is_file() or candidate.suffix not in {".map", ".toml", ".c", ".ld"}:
            continue
        for lineno, line in enumerate(candidate.read_text(encoding="utf-8", errors="replace").splitlines(), 1):
            for token in line.strip().split():
                if ABSOLUTE_PATH_RE.match(token):
                    sys.exit(
                        f"fixture {candidate.relative_to(ROOT)}:{lineno} carries an absolute path "
                        f"({token!r}); refusing to commit a private machine path"
                    )


def drop(dirpath: Path, name: str) -> None:
    path = dirpath / name
    if path.exists():
        path.unlink()


def committed_names(dirpath: Path) -> list[str]:
    return [
        p.relative_to(dirpath).as_posix()
        for p in sorted(dirpath.rglob("*"))
        if p.is_file() and p.name != "fixture.toml"
    ]


def layout_block(readelf: str, elf: Path, map_path: Path) -> list[tuple[str, str]]:
    """Region facts readable from the file, as opposed to claims about what the product decides.

    The two `expected_*_bytes` rows are arithmetic on this file's section table and the MAP's region
    attributes, so they can disagree with the product — and a disagreement is the finding this round
    would need, not a number copied out of the answer under test.
    """
    rows = section_table(readelf, elf)
    regions = map_regions(map_path)
    allocatable = {name: (addr, size) for name, _type, addr, size, flags in rows if "A" in flags}
    parts = [f"{name} 0x{addr:08x}+0x{size:x}" for name, (addr, size) in sorted(allocatable.items())]
    image, ram = expected_totals(elf, rows, regions)
    return [
        ("load_segments", str(load_segments(readelf, elf))),
        ("allocatable_sections", " ; ".join(parts) if parts else "none"),
        ("entry_point", f"0x{entry_point(readelf, elf):08x}"),
        ("expected_image_bytes", image),
        ("expected_live_ram_bytes", ram),
        ("acceptance", "crates/firmwaresight-artifact/tests/p5_compat_fixtures.rs"),
    ]


def fixture_toml(
    *,
    fixture_id: str,
    purpose: str,
    case: str,
    architecture: str,
    compiler: str,
    compiler_version: str,
    compiler_from_artifact: str,
    linker: str,
    commands: list[str],
    layout: list[tuple[str, str]],
    expectations: list[tuple[str, str]],
    limitations: list[str],
    files: list[str],
) -> str:
    parts = [
        "# Provenance record for a committed P5 compatibility fixture.",
        "# Generated by scripts/gen_p5_compat_fixtures.py. The bytes are tool output; this file names",
        "# the tools that produced them, the layout facts readable from them, and what they are here to",
        "# prove. The expected_* values below are arithmetic on readelf output and on this MAP's own",
        "# region table, not on the analyzer's answer; the analyzer is held to them by",
        "# crates/firmwaresight-artifact/tests/p5_compat_fixtures.rs.",
        "",
        f'fixture_id = "{fixture_id}"',
        f'purpose = "{purpose}"',
        f'layout_case = "{case}"',
        'license = "Written for this project; no third-party code copied"',
        f'generated_on = "{datetime.date.today().isoformat()}"',
        "",
        # Two identities, never one, and the compiler one repeated from the artifact's own bytes.
        "[toolchain]",
        f'compiler = "{compiler}"',
        f'compiler_version = "{compiler_version}"',
        f'compiler_in_artifact = """{compiler_from_artifact}"""',
        f'linker = """{linker}"""',
        'linker_role = "final link and the memory map"',
        "",
        "[build]",
        'rebuild_script = "scripts/gen_p5_compat_fixtures.py"',
        'relative_paths = "required: GNU ld copies the object paths it is given into the MAP"',
        "commands = [",
        "\n".join(f'  """{command}""",' for command in commands),
        "]",
        "",
        "[expectations]",
        f'architecture = "{architecture}"',
        "",
    ]
    parts += [f'{key} = """{value}"""\n' for key, value in [*expectations, *layout]]
    parts += [f'[[known_limitations]]\ndescription = """{item}"""\n' for item in limitations]
    parts += [f'[[files]]\npath = "{name}"\n' for name in files]
    return "\n".join(parts)


BASIS_EXPECTATION = (
    "expected_product_evidence_basis",
    "map-memory-configuration+elf-load for every charged section: the region attribute the MAP "
    "declares plus the address the ELF carries, never the section name",
)


def layout_summary(readelf: str, elf: Path, map_path: Path) -> str:
    """The same derivation, rendered as one line for a record that covers a second file."""
    rows = dict(layout_block(readelf, elf, map_path))
    return "; ".join(
        f"{key.replace('expected_', '').replace('_bytes', '')} {rows[key]}"
        for key in ("entry_point", "expected_image_bytes", "expected_live_ram_bytes", "load_segments")
    )


def update_manifest(entries: list[dict[str, str]]) -> None:
    """Rewrite the manifest keeping every entry that is not this generator's own namespace."""
    path = ROOT / "fixtures" / "manifest.json"
    existing = json.loads(path.read_text(encoding="utf-8"))
    kept = [e for e in existing["files"] if not e["path"].startswith("fixtures/elf/p5-compat/")]
    merged = sorted(kept + entries, key=lambda e: e["path"])
    generators = list(existing.get("generators", [existing["generated_by"]]))
    mine = "scripts/gen_p5_compat_fixtures.py"
    manifest = {
        "schema": existing["schema"],
        "generated_by": existing["generated_by"],
        "generators": generators if mine in generators else [*generators, mine],
        "note": existing["note"],
        "files": merged,
    }
    path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8", newline="\n")
    print(f"\nmanifest: {len(kept)} prior entries kept, {len(entries)} P5 entries recorded")


# ---- the fixtures --------------------------------------------------------------------------------


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--force", action="store_true", help="regenerate fixture binaries and re-record hashes")
    args = parser.parse_args()

    needed = {
        "arm-none-eabi-gcc": shutil.which("arm-none-eabi-gcc"),
        "arm-none-eabi-ld": shutil.which("arm-none-eabi-ld"),
        "arm-none-eabi-objcopy": shutil.which("arm-none-eabi-objcopy"),
        "arm-none-eabi-readelf": shutil.which("arm-none-eabi-readelf"),
        "clang": shutil.which("clang"),
    }
    missing = [name for name, path in needed.items() if path is None]
    if missing:
        sys.exit(
            f"missing {', '.join(missing)}. Parser, CLI and golden tests do NOT need any of them; they are "
            "only required to regenerate these fixtures. Nothing here installs a toolchain."
        )
    gcc, ld = needed["arm-none-eabi-gcc"], needed["arm-none-eabi-ld"]
    objcopy, readelf, clang = needed["arm-none-eabi-objcopy"], needed["arm-none-eabi-readelf"], needed["clang"]

    gcc_version, clang_version, ld_version = first_version_line(gcc), first_version_line(clang), first_version_line(ld)
    arch = "ARM (cortex-m4, thumb, 32-bit little-endian)"
    print(f"compiler  {gcc_version}\ncompiler  {clang_version}\nlinker    {ld_version}\n")

    entries: list[dict[str, str]] = []
    gcc_compile = ["-g", "-Os", "-ffunction-sections", "-fdata-sections"]

    def guard(*paths: Path) -> None:
        existing = [p for p in paths if p.exists()]
        if existing and not args.force:
            sys.exit(
                f"{', '.join(p.name for p in existing)} already exists; pass --force to regenerate and "
                "re-record hashes"
            )

    def emit(dirpath: Path, fixture_id: str, toml: str) -> None:
        build = dirpath / "build"
        if build.exists():
            shutil.rmtree(build)
        names = committed_names(dirpath)
        (dirpath / "fixture.toml").write_text(toml, encoding="utf-8", newline="\n")
        assert_no_host_paths(dirpath)
        for name in [*names, "fixture.toml"]:
            entries.append(
                {
                    "path": (dirpath / name).relative_to(ROOT).as_posix(),
                    "sha256": sha256(dirpath / name),
                    "fixture": fixture_id,
                    "purpose": "compatibility-acceptance",
                }
            )
        print(f"    committed: {', '.join(names)}\n")

    # ---- Case B: executable code resident in RAM --------------------------------------------------
    d = PARENT / "ram-exec"
    guard(d / "firmware.elf")
    prepare(d, RAM_EXEC_C, "ram-exec.ld")
    o = compile_with(gcc, d, "main", gcc_compile, clang=False)
    link_with_gnu_ld(d, ld, [o], "ram-exec.ld", "firmware.elf", "firmware.map", [])
    emit(
        d,
        "p5-ram-exec",
        fixture_toml(
            fixture_id="p5-ram-exec",
            purpose="Case B, executable-in-RAM: .fast_text is placed in writable RAM with its load "
            "address in FLASH, so VMA and LMA disagree for a section holding code.",
            case="B",
            architecture=arch,
            compiler="arm-none-eabi-gcc",
            compiler_version=gcc_version,
            compiler_from_artifact=comment_of(readelf, d / "firmware.elf"),
            linker=ld_version,
            commands=[
                "arm-none-eabi-gcc -mcpu=cortex-m4 -mthumb -g -Os -ffunction-sections -fdata-sections "
                "-c source/main.c -o build/main.o",
                "arm-none-eabi-ld -e main -Tram-exec.ld build/main.o -o firmware.elf -Map=firmware.map",
            ],
            layout=layout_block(readelf, d / "firmware.elf", d / "firmware.map"),
            expectations=[
                BASIS_EXPECTATION,
                (
                    "expected_product_unknowns",
                    "build id (no .note.gnu.build-id section is present); git context (analyze is given "
                    "one artifact and no project); and why .fast_text lives in RAM - the script says it "
                    "does, nothing in the file says for what purpose",
                ),
            ],
            limitations=[
                "one controlled linker script, not a vendor SDK overlay implementation",
                "proves the accounting rule on this evidence only, not on every toolchain",
            ],
            files=committed_names(d),
        ),
    )

    # ---- Case C: external SRAM --------------------------------------------------------------------
    d = PARENT / "extsram"
    guard(d / "firmware.elf")
    prepare(d, EXTSRAM_C, "extsram.ld")
    o = compile_with(gcc, d, "main", gcc_compile, clang=False)
    link_with_gnu_ld(d, ld, [o], "extsram.ld", "firmware.elf", "firmware.map", [])
    emit(
        d,
        "p5-extsram",
        fixture_toml(
            fixture_id="p5-extsram",
            purpose="Case C, external SRAM: a third writable region at 0x60000000 that the product can "
            "only know from the MAP region table, because no internal-RAM assumption reaches that address.",
            case="C",
            architecture=arch,
            compiler="arm-none-eabi-gcc",
            compiler_version=gcc_version,
            compiler_from_artifact=comment_of(readelf, d / "firmware.elf"),
            linker=ld_version,
            commands=[
                "arm-none-eabi-gcc -mcpu=cortex-m4 -mthumb -g -Os -ffunction-sections -fdata-sections "
                "-c source/main.c -o build/main.o",
                "arm-none-eabi-ld -e main -Textsram.ld build/main.o -o firmware.elf -Map=firmware.map",
            ],
            layout=layout_block(readelf, d / "firmware.elf", d / "firmware.map"),
            expectations=[
                BASIS_EXPECTATION,
                (
                    "expected_product_unknowns",
                    "build id (no .note.gnu.build-id section is present); git context (analyze is given "
                    "one artifact and no project); and what technology sits at 0x60000000 - the MAP "
                    "declares an address range with attributes and nothing more",
                ),
            ],
            limitations=[
                "the address is conventional for an external bus, not tied to a specific part",
                "proves a declared region is accounted from real MAP evidence, not that every vendor map is covered",
            ],
            files=committed_names(d),
        ),
    )

    # ---- Case D: custom writable region -----------------------------------------------------------
    d = PARENT / "dma-region"
    guard(d / "firmware.elf")
    prepare(d, DMA_REGION_C, "dma-region.ld")
    o = compile_with(gcc, d, "main", gcc_compile, clang=False)
    link_with_gnu_ld(d, ld, [o], "dma-region.ld", "firmware.elf", "firmware.map", [])
    emit(
        d,
        "p5-dma-region",
        fixture_toml(
            fixture_id="p5-dma-region",
            purpose="Case D, a custom-named writable region: DMARAM at 0x30000000 holds a buffer pool. "
            "The narrow claim is that an unfamiliar region name does not make its memory disappear.",
            case="D",
            architecture=arch,
            compiler="arm-none-eabi-gcc",
            compiler_version=gcc_version,
            compiler_from_artifact=comment_of(readelf, d / "firmware.elf"),
            linker=ld_version,
            commands=[
                "arm-none-eabi-gcc -mcpu=cortex-m4 -mthumb -g -Os -ffunction-sections -fdata-sections "
                "-c source/main.c -o build/main.o",
                "arm-none-eabi-ld -e main -Tdma-region.ld build/main.o -o firmware.elf -Map=firmware.map",
            ],
            layout=layout_block(readelf, d / "firmware.elf", d / "firmware.map"),
            expectations=[
                BASIS_EXPECTATION,
                (
                    "expected_product_unknowns",
                    "build id (no .note.gnu.build-id section is present); git context; and "
                    "cacheability or coherency, which a MAP region table cannot express at all - it "
                    "declares r/w/x, so no cache statement may be reported from this file",
                ),
            ],
            limitations=[
                "no cache or coherency semantics are claimed or modelled: the section name says DMA, the "
                "evidence says writable RAM, and Commit E adds no cacheability dimension",
                "one region, one layout; not a survey of DMA-capable memory maps",
            ],
            files=committed_names(d),
        ),
    )

    # ---- Case G: no debug, and post-link stripped, kept apart -------------------------------------
    d = PARENT / "no-debug"
    guard(d / "firmware-nodebug.elf", d / "firmware-stripped.elf")
    prepare(d, PLAIN_C, "plain.ld")
    stripped_from = compile_with(gcc, d, "with_debug", ["-g", "-Os"], clang=False)
    link_with_gnu_ld(d, ld, [stripped_from], "plain.ld", "with-debug.elf", None, [])
    run([objcopy, "-S", "with-debug.elf", "firmware-stripped.elf"], d)
    drop(d, "with-debug.elf")
    plain_obj = compile_with(gcc, d, "main", ["-Os"], clang=False)
    link_with_gnu_ld(d, ld, [plain_obj], "plain.ld", "firmware-nodebug.elf", "firmware.map", [])
    emit(
        d,
        "p5-no-debug",
        fixture_toml(
            fixture_id="p5-no-debug",
            purpose="Case G, two different absences kept apart: firmware-nodebug.elf was compiled without "
            "-g, firmware-stripped.elf was linked with debug and then had it removed by objcopy -S. "
            "They are not the same event and only the second one is a post-link strip.",
            case="G",
            architecture=arch,
            compiler="arm-none-eabi-gcc",
            compiler_version=gcc_version,
            compiler_from_artifact=comment_of(readelf, d / "firmware-nodebug.elf"),
            linker=ld_version,
            commands=[
                "arm-none-eabi-gcc -mcpu=cortex-m4 -mthumb -g -Os -c source/main.c -o build/with_debug.o",
                "arm-none-eabi-ld -e main -Tplain.ld build/with_debug.o -o with-debug.elf",
                "arm-none-eabi-objcopy -S with-debug.elf firmware-stripped.elf",
                "arm-none-eabi-gcc -mcpu=cortex-m4 -mthumb -Os -c source/main.c -o build/main.o",
                "arm-none-eabi-ld -e main -Tplain.ld build/main.o -o firmware-nodebug.elf -Map=firmware.map",
            ],
            layout=layout_block(readelf, d / "firmware-nodebug.elf", d / "firmware.map"),
            expectations=[
                BASIS_EXPECTATION,
                (
                    "expected_product_unknowns",
                    "build id (no .note.gnu.build-id section is present); git context; debug info, "
                    "which is unavailable in both files; and for firmware-stripped.elf the symbol "
                    "table, which is absent - an absence that must stay an absence rather than become "
                    "zero named symbols",
                ),
                (
                    "second_file_expectations",
                    "firmware-stripped.elf carries the same layout, derived the same way: "
                    + layout_summary(readelf, d / "firmware-stripped.elf", d / "firmware.map"),
                ),
            ],
            limitations=[
                "measured, not assumed: on this binutils (2.44.0.20250616) `objcopy -S` removed the "
                ".debug_* sections and also .symtab and .strtab, so firmware-stripped.elf has no symbol "
                "table while firmware-nodebug.elf still does. The two files therefore differ in two ways, "
                "and a test that reads one as the other would be reading its own assumption",
                "the intermediate with-debug.elf was deliberately not committed, so nothing here implies a "
                "third claim about a third file",
            ],
            files=committed_names(d),
        ),
    )

    # ---- Clang-produced Arm code, GNU ld final link -----------------------------------------------
    d = PARENT / "clang-arm"
    guard(d / "firmware.elf")
    prepare(d, PLAIN_C, "plain.ld")
    o = compile_with(clang, d, "main", ["-g", "-Os", "-ffunction-sections", "-fdata-sections"], clang=True)
    link_with_gnu_ld(d, ld, [o], "plain.ld", "firmware.elf", "firmware.map", [])
    clang_in_elf = comment_of(readelf, d / "firmware.elf")
    if "clang" not in clang_in_elf.lower():
        sys.exit(
            f"the Clang fixture's own bytes do not name clang (.comment reports {clang_in_elf!r}); refusing "
            "to commit a fixture whose compiler identity is a claim rather than an observation"
        )
    emit(
        d,
        "p5-clang-arm",
        fixture_toml(
            fixture_id="p5-clang-arm",
            purpose="The Clang half of the supported cohort: code produced by clang for an explicit Arm "
            "target and final-linked by GNU ld, so the ELF carries clang's own .comment while the MAP "
            "remains GNU ld output. The layout is the plain one on purpose, to keep the compiler the variable.",
            case="compiler",
            architecture=arch,
            compiler="clang",
            compiler_version=clang_version,
            compiler_from_artifact=clang_in_elf,
            linker=ld_version,
            commands=[
                "clang --target=arm-none-eabi -ffreestanding -fno-builtin -mcpu=cortex-m4 -mthumb -g -Os "
                "-ffunction-sections -fdata-sections -c source/main.c -o build/main.o",
                "arm-none-eabi-ld -e main -Tplain.ld build/main.o -o firmware.elf -Map=firmware.map",
            ],
            layout=layout_block(readelf, d / "firmware.elf", d / "firmware.map"),
            expectations=[
                BASIS_EXPECTATION,
                (
                    "expected_product_unknowns",
                    "build id (no .note.gnu.build-id section is present); git context; and the role "
                    "of .ARM.exidx.text.main, which the parser still does not recognise - it is charged "
                    "from region and load evidence, not from a name it knows",
                ),
            ],
            limitations=[
                "proves clang-compiled Arm code at this target, this optimization level and with this linker",
                "not a survey of clang -mcpu values, not an Apple or Windows-target clang, and not an lld link",
            ],
            files=committed_names(d),
        ),
    )

    # ---- Case E: a genuine long discarded-section preamble ----------------------------------------
    d = PARENT / "long-preamble"
    guard(d / "firmware.elf")
    prepare(d, discarded_c(220), "plain.ld")
    o = compile_with(gcc, d, "main", gcc_compile, clang=False)
    link_with_gnu_ld(d, ld, [o], "plain.ld", "firmware.elf", "firmware.map", ["--gc-sections"])
    offset = banner_offset(d / "firmware.map")
    print(f"    MAP banner '{BANNER}' first appears at byte {offset}")
    if offset < MIN_BANNER_OFFSET:
        sys.exit(
            f"the generated MAP places its banner at byte {offset}, below the {MIN_BANNER_OFFSET}-byte window "
            "this fixture exists to exceed; generating more dead code is the fix, weakening an assertion is not"
        )
    emit(
        d,
        "p5-long-preamble",
        fixture_toml(
            fixture_id="p5-long-preamble",
            purpose=f"Case E: 220 unreferenced functions discarded by --gc-sections push the MAP's "
            f"'{BANNER}' table to byte {offset}, past the 4096-character window a fixed head read used to "
            "stop at. The preamble is linker output, not pasted filler.",
            case="E",
            architecture=arch,
            compiler="arm-none-eabi-gcc",
            compiler_version=gcc_version,
            compiler_from_artifact=comment_of(readelf, d / "firmware.elf"),
            linker=ld_version,
            commands=[
                "arm-none-eabi-gcc -mcpu=cortex-m4 -mthumb -g -Os -ffunction-sections -fdata-sections "
                "-c source/main.c -o build/main.o",
                "arm-none-eabi-ld --gc-sections -e main -Tplain.ld build/main.o -o firmware.elf -Map=firmware.map",
            ],
            layout=layout_block(readelf, d / "firmware.elf", d / "firmware.map")
            + [("map_banner_offset", f"{offset} bytes from the start of the file")],
            expectations=[
                BASIS_EXPECTATION,
                (
                    "expected_product_unknowns",
                    "build id (no .note.gnu.build-id section is present); git context; and what "
                    "--gc-sections discarded, which is simply absent from the ELF - so the honest "
                    "answer for this image is small rather than a parse failure",
                ),
            ],
            limitations=[
                "the preamble length is a property of this source and this linker version; the generator "
                "re-measures the banner offset on every run instead of trusting the number recorded here",
            ],
            files=committed_names(d),
        ),
    )

    update_manifest(entries)
    print(f"\nwrote {len(entries)} manifest entries")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
