---
title: "P0 Parser Results"
doc_id: "FS-P0-006"
product: "FirmwareSight"
version: "0.5.1"
status: "EXECUTION_RECORD"
owner: "Engineering"
last_updated: "2026-09-28"
---

# P0 Parser Results

Requirement: real ELF and real MAP input must produce normalized facts, every fact must carry
its evidence class, and nothing may be invented where a fact is absent.

Status: **LOCAL PASS**

## ELF reader (`object-elf/0.40`)

Read from the committed goldens, which are the CLI's own `--json` output:

| Fact | Fixture A `p0-basic` | Fixture B `p0-dual-region` |
| --- | --- | --- |
| kind / parser | `elf` / `object-elf/0.40` | same |
| byte size | 7,288 | 10,960 |
| architecture | `Arm`, 32-bit, little | same |
| entry point | `0x00008001` | `0x08000039` |
| build id | none present (`Fact::Unknown` with reason) | none present |
| sections | 19 | 17 |
| symbols | 42 | 32 |
| evidence items | 10 (9 observed, 1 derived) | 11 (all observed) |
| ELF capability | `supported` | `supported` |
| sections / symbols capability | `available` / `available` | `available` / `available` |
| debug info | `available` | `available` |
| MAP capability | `not-provided` | `provided` |
| object attribution | `unavailable` (no MAP) | `available` |
| Git capability | `unknown` | `unknown` |

Entry point `0x00008001` on fixture A is reported exactly as linked: the low bit is the Thumb
indicator the linker emitted. The parser does not silently mask it off, because "what the file
says" and "what we think it means" are different evidence classes and only the first is Observed.

A missing build id is a `Fact::Unknown { reason }`, not a zero-length id: the golden records
`no .note.gnu.build-id section is present`, and the desktop renders the reason rather than a blank.

## GNU ld MAP adapter (`adapter_id = gnu_ld`)

Fixture B's `firmware.map` (3,249 bytes) yields facts the ELF alone cannot support:

| MAP output | Value |
| --- | --- |
| Memory Configuration regions | `ROM (rx) 0x08000000 len 1024K`, `RAM (rwx) 0x20000000 len 256K` |
| Output format line | `elf32-littlearm` |
| Placement records | per input section: address, size, load address where it differs |
| Object contributions | per `.o`, bytes attributed |
| Resulting layout source | `map` (`LayoutSource::MapMemoryConfiguration`) |

GNU ld prints `load address 0x...` only when it differs from the virtual address. An absent
clause is therefore read as `lma == vma`, recorded in `map.rs` next to the parse rather than
left as a silent default.

Foreign maps are refused with `MapUnsupported` and no fallback parser: an ArmClang or IAR banner
is detected and rejected rather than approximately interpreted
(`foreign_toolchain_maps_are_rejected_without_falling_back`).

## Two fixtures, one source - what the parser must not do

Both ELF files compile the identical `source/main.c`. The only difference is whether a MAP was
supplied. The reported budgets are identical in size but differ in strength:

| | Fixture A | Fixture B |
| --- | --- | --- |
| Nonvolatile footprint | 160 bytes, `exact`, **observed** | 160 bytes, `exact`, **observed** |
| Runtime RAM footprint | 72 bytes, **`partial`, derived** | 72 bytes, `exact`, **observed** |
| Layout source | `none` | `map` |
| Weakest evidence basis | `elf-address-and-flags` | `map-memory-configuration+elf-load` |
| Admissible for a hard block | **false** | true |
| Excluded metadata bytes | 2,227 | 1,975 |

Identical numbers with different provenance is the desired outcome. A parser that produced the
same claim in both cases would be reporting confidence it does not have.

## Panic discipline on hostile input

```
$ RUSTUP_TOOLCHAIN=stable cargo test -p firmwaresight-artifact
test result: ok. 19 passed; 0 failed     # tests/p0_acceptance.rs
```

Coverage inside that run: all four `fixtures/malformed/` files, a truncated ELF whose table
offsets point past EOF, a zero-filled header whose declared sizes are all zero, a foreign MAP,
a MAP with no region table, and the size guard firing before any allocation. Every case returns
a typed `ArtifactError`; none panics.

## Commands

```
$ cargo run -q -p fwsight -- analyze fixtures/elf/p0-dual-region/firmware.elf \
      --map fixtures/elf/p0-dual-region/firmware.map
$ cargo run -q -p fwsight -- analyze fixtures/elf/p0-basic/firmware.elf --json
$ RUSTUP_TOOLCHAIN=stable cargo test -p firmwaresight-artifact
```

## Known parser limits carried forward

- No DWARF consumption: debug sections are recognized and excluded from budgets, but no
  source-line or compile-unit facts are produced.
- Program headers are read for identity only; segment-level accounting is not implemented.
- Only GNU ld maps are supported. ArmClang, IAR and linker scripts from vendor IDEs remain
  refused rather than partially understood.
