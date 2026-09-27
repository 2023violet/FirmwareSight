---
title: "P0 Memory Accounting Report"
doc_id: "FS-P0-007"
product: "FirmwareSight"
version: "0.5.1"
status: "EXECUTION_RECORD"
owner: "Engineering"
last_updated: "2026-09-28"
---

# P0 Memory Accounting Report

Requirement (ADR-0021, `04_TECH/23_MEMORY_ACCOUNTING_MODEL.md`): nonvolatile image footprint and
runtime RAM footprint are two different budgets; a section can be counted in both when it is
stored in flash and resident in RAM; and the strength of the load evidence must be reported
alongside the number, never hidden behind it.

Status: **LOCAL PASS**

## The fixture was built to make this falsifiable

Fixture B's linker script states the case directly:

```
MEMORY { ROM (rx):  ORIGIN = 0x08000000, LENGTH = 1024K
         RAM (rwx): ORIGIN = 0x20000000, LENGTH = 256K }
.data : { *(.data*) } > RAM AT> ROM
.bss (NOLOAD) : { *(.bss*) *(COMMON) } > RAM
.ota  : { *(.ota_staging*) } > RAM AT> ROM
```

`.data` and `.ota` are VMA-resident in RAM but LMA-stored in ROM. If the model were wrong - one
budget, or "runtime address decides everything" - these two sections would be counted once, and
the totals below would not come out.

## Independent arithmetic, from `readelf` and the MAP region table

```
$ arm-none-eabi-readelf -S -W fixtures/elf/p0-dual-region/firmware.elf
  [1] .text     PROGBITS 08000000  size 60
  [2] .rodata   PROGBITS 0800003c  size 32
  [3] .data     PROGBITS 20000000  size  4
  [4] .bss      NOBITS   20000004  size  4
  [5] .ota      PROGBITS 20000008  size 64

$ grep -A5 "Memory Configuration" fixtures/elf/p0-dual-region/firmware.map
ROM   0x08000000   0x00100000   xr
RAM   0x20000000   0x00040000   xrw
```

By hand:

- Runtime RAM = sections whose VMA lies in `RAM (xrw)` = 4 + 4 + 64 = **72**
- Nonvolatile image = ROM-resident bytes = `.text` 60 + `.rodata` 32, plus the flash-stored load
  images of `.data` (4) and `.ota` (64) = **160**
- 92 + 72 = 164 ≠ 160, and that is the dual-accounting signature: 68 bytes are counted in both
  budgets because they genuinely occupy both.

The tool's own output reproduces exactly these numbers:

| Field | Value |
| --- | --- |
| `nonvolatileImageFootprint` | `{ state: "exact", classification: "observed", bytes: 160 }` |
| `runtimeRamFootprint` | `{ state: "exact", classification: "observed", bytes: 72 }` |
| `dualAccountedSections` | `elf.section_header[3]` (.data), `elf.section_header[5]` (.ota) |
| `excludedMetadataBytes` | 1,975 (`.debug_*`, `.symtab`, `.strtab`, `.comment`, `.ARM.attributes`) |
| `layoutSource` | `map` |
| `weakestEvidenceBasis` | `map-memory-configuration+elf-load` |
| `admissibleForHardBlock` | `true` |

`.bss` contributes 0 bytes to the image and 4 to RAM - it is NOBITS, so it has no flash footprint
(`.bss` never appears in `dualAccountedSections`; `bss_costs_ram_and_nothing_in_the_image`
asserts exactly that).

## The same source without load evidence

Fixture A compiles the identical `source/main.c` with no MAP.

| Field | Fixture A | Fixture B |
| --- | --- | --- |
| Nonvolatile | 160, `exact`, observed | 160, `exact`, observed |
| Runtime RAM | 72, **`partial`, derived** | 72, `exact`, observed |
| Layout source | `none` | `map` |
| Weakest basis | `elf-address-and-flags` | `map-memory-configuration+elf-load` |
| Admissible for a hard block | **false** | true |

Two `.noinit`-class sections carry neither a declared size nor a payload, so the runtime total can
only be a floor: `unattributed: ["elf.section_header[5]", "elf.section_header[7]"]` with the
reason text attached. The same numbers, honestly labelled as weaker.

## The evidence ladder

`MemoryEvidenceBasis` orders five rungs; each carries its own evidence class, and the ceiling is
a property of the rung, not of the caller's confidence.

| Rung | Source | Class | May support a hard block |
| --- | --- | --- | --- |
| `RegionConfigAndElfLoad` | project region configuration + ELF load | Declared/Observed | yes |
| `MapRegionAndElfLoad` | GNU ld `Memory Configuration` + ELF load | Observed | yes |
| `ElfAddressAndFlags` | `sh_addr` + `sh_flags` | Derived | **no** |
| `SectionNameHeuristic` | `.text` / `.bss` naming | Derived, low confidence | **no** |
| `Insufficient` | nothing attributable | Unknown | **no** |

`MemoryFootprint::admissible_for_hard_block()` is the only place that ceiling is read, and the
gate layer of later phases is the only consumer. Nothing in P0 turns an inadmissible total into a
BLOCK - `flags_only_mapping_cannot_support_a_hard_block`,
`name_heuristic_alone_is_derived_and_offers_no_number` and
`without_a_map_the_model_reports_weaker_evidence_not_false_confidence` are the assertions.

## Tests and commands

```
$ RUSTUP_TOOLCHAIN=stable cargo test -p firmwaresight-core        # 39 tests, 18 of them in domain/memory.rs
$ RUSTUP_TOOLCHAIN=stable cargo test -p firmwaresight-artifact    # 19 acceptance tests
$ RUSTUP_TOOLCHAIN=stable cargo test -p fwsight --test golden     # memory golden
```

Named evidence in `p0_acceptance.rs`:
`.data` dual-accounted 4/4 and `.ota` 64/64 at `MapRegionAndElfLoad`; `.bss` contributing 0 to
the image; debug sections excluded from both budgets; totals exact and admissible; the two
budgets never collapsing into one number; and the same source degrading to a weaker basis when
the MAP is withheld.

`golden/core/p0-basic-memory.json` and `golden/core/p0-dual-region-memory.json` pin the whole
memory projection, so a future change to the rule table cannot silently move a number -
`memory_accounting_golden_passes_on_both_fixtures` fails, and `scripts/check.py --only drift`
fails on any unreviewed golden rewrite.

## What this does not prove

- It proves the rule on two linker layouts built by one toolchain, not on every vendor script.
  Executable-region-in-RAM, external RAM, DMA pools and overlay regions are untested.
- It measures footprint, not headroom: stack, heap and allocator behaviour remain unknown
  (`Fact::Unknown`), and P0 has no device RAM usage data to consult.
- It reports `exact` only for what the ELF and MAP declare. A linker script that lies about its
  regions would be faithfully reproduced as a wrong-but-well-labelled number.
