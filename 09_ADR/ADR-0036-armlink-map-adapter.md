---
title: "ADR-0036: armlink MAP adapter"
doc_id: "FS-ADR-0036"
product: "FirmwareSight"
version: "0.6.0"
status: "BASELINE"
owner: "Engineering"
last_updated: "2026-10-18"
---

# ADR-0036: Read the Arm linker (armlink) image map

## Status

**BASELINE** (2026-10-18). Owner direction in chat on 2026-10-18 (`.ai/DECISIONS.md` OS3) was to add the features most worth adding, armlink first. This is route batch D4 (`06_DELIVERY/12`). It follows ADR-0033's rules for object attribution and `04_TECH/39`'s rule that each linker gets its own grammar.

## Context

- Keil MDK and Arm Compiler (armclang + armlink) build a large share of Cortex-M firmware. Until now FirmwareSight read their ELF (`.axf`) but refused their MAP (`ERR-MAP-3001`). Without it there are no region capacities, no MAP-backed accounting and no object or library attribution.
- `fixtures/elf/d4-armlink` holds a same-origin armlink sample: the M2 sources, Arm Compiler 6.24, a scatter file, and `--map --load_addr_map_info --symbols --info=sizes,totals,unused`. Its totals agree with GNU readelf (`evidence/D4/CROSSCHECK.md`).

## Decision

1. **A separate adapter** (`map_armlink.rs`, adapter id `armlink`, parser id `map-adapter/armlink`).
   - It produces the same `MapEvidence` as GNU ld: regions, placements, object rows and linker bytes.
   - `map::parse_any` dispatches by banner: armlink's `Memory Map of the image` with an `Execution Region` goes to this adapter; GNU ld goes to the GNU adapter.
   - Anything else, including IAR and an armlink listing without an image map, is refused as before (`ERR-MAP-3001`).
   - The GNU adapter's `detect` keeps refusing armlink text, so neither grammar is ever read as the other.
2. **Regions are the execution regions:** `Exec base` (or `Base`) and `Max`.
   - A region holding any RW or ZI row is RAM; a region of RO/XO rows is Flash.
   - A region without `Max` is a parse error, because a capacity is never guessed.
   - Load regions are not separate budget regions. In the usual scatter file the RO execution region spans the load region. This is recorded as a known limit for scatter files whose load region is larger than its first execution region.
3. **Load addresses are only what armlink printed.**
   - With `--load_addr_map_info` (which µVision passes by default), the region header's `Load base` is used.
   - Without it, a region that executes inside its load region loads where it runs.
   - One that executes elsewhere has an **unknown** load address, with the reason "link with --load_addr_map_info". The address is never inferred.
4. **Compressed RW data** (`COMPRESSED[...]`) is stored at a size that is not the section's. That region's load address is reported as unknown, so its Flash is not charged at the wrong size.
5. **Object rows** follow ADR-0033:
   - the key is the Object column: `libx.lib(y.o)` for a library member, normalized by the same rule as GNU ld (never a host path);
   - `PAD` is linker fill and veneers are linker stubs, never an object's bytes;
   - `anon$$obj.o` (armlink's object for common symbols) is kept as armlink names it.
6. **Two output sections can share a name.** armlink names ELF sections after execution regions, so the RW data and the ZI part of one region are both `RW_IRAM1`. Core therefore keeps each section's runtime span (`MemoryContribution::runtime_span`):
   - `attribution::charge` picks the section of that name whose span holds the row;
   - a row that no candidate holds is Unknown;
   - a unique name behaves exactly as before.
7. **Not changed:** the analysis, diff and gate schemas; storage; the GNU adapter's grammar; the `LayoutSource` vocabulary (`map` means "a supported MAP adapter produced the region table").

## Consequences

- **Support per layer** (`04_TECH/39`): armlink ELF read, MAP regions, MAP-backed accounting, object and library attribution, and Compare are supported, backed by the synthetic same-origin fixture and the cross-check.
- **Still open, and gaps a real project should confirm:**
  - Compare pairs sections by name, so armlink's repeated `RW_IRAM1` rows stay unpaired (a D4 follow-up);
  - µVision IDE builds, the Arm C library or microlib, RTOS projects, multiple load regions and compressed RW data from real projects.
- User-visible text says "linker MAP (GNU ld or armlink)" where it used to say "GNU ld MAP".

## Alternatives rejected

- **Teach the GNU parser armlink rows:** rejected by `04_TECH/39`, because one grammar per linker keeps a refusal from turning into a misread.
- **Infer load addresses from the load-region layout:** it is plausible, but it is a guess the MAP does not state; `--load_addr_map_info` states it.

## Review (2026-10-18)

The independent review returned PASS_WITH_FIXES with no S1. Every finding was fixed:

**S2:**
- the third-party list covers only the four shipping targets of `deny.toml` (377 crates; r-efi, wasm and redox are gone);
- the load-address offset uses `checked_add` and applies only when the placement holds the section, so GNU behaviour is unchanged;
- the current-state docs and the error hint are updated;
- the Core and IPC wording is "linker MAP", and the three goldens that print the Compare reason are regenerated.

**S3:**
- an empty execution region is of Unknown kind;
- a load region without `Max` proves no containment;
- GNU ld's banner wins the dispatch;
- the README notes that the real-project cohort is UNVERIFIED.

**Kept:**
- an object name containing a space is a known parsing limit;
- the `p3_execution.license_gap` line in `BASELINE.yaml` records the P3 round and is history.


## Amendment A1 (2026-10-18): compressed RW data, from real Keil projects

Two real Keil MDK projects ([evidence/D4-REAL](../evidence/D4-REAL/README.md)) showed that decision 4 was wrong in both directions. The Flash total left out the compressed bytes, and RAM was the compressed size reported as exact. Decision 4 is replaced:

1. **The ELF section of a compressed region holds the compressed bytes.** That is what Flash stores, so it is charged at its file size from the region's `Load base`.
2. **The runtime size comes from the MAP.** The region's `Size` is the decompressed size (`MapPlacement::runtime_size`). The data section's runtime size is that `Size` less the region's ZI sections. If the region holds more than one data section, each one's runtime size is Unknown, because the MAP gives only their total. The value is computed from two observed numbers: the MAP's region `Size` and the ELF's ZI sizes. Alignment padding inside the region would be counted in the data section. On `dsp-fir` the result equals armlink's own `RW-data=2572`.
3. **Rows in a compressed region:** each row's RAM is its size; its Flash is **Unknown**, because only the region's total stored size is known (`attribution::charge`).
4. **Without a `Load base`** the compressed region's load address stays unknown, with the reason "link with --load_addr_map_info".
5. **ELF only:** if the image contains armlink's decompressor (`__decompress*`), every initialized writable section's runtime size is Unknown, so RAM is a lower bound (derived), never exact. With a MAP, the MAP decides.
6. **Known limits of the ELF-only rule:**
   - a stripped armlink image has no symbols, so the decompressor cannot be seen and the stored size stays as the runtime size; analyze it with its MAP;
   - a non-armlink image that defines a `__decompress*` symbol loses its RW runtime sizes to Unknown. This errs toward Unknown, never toward a wrong number.
7. **Zero-size rows are not contributions.** armlink lists the C library's `.ARM.Collect$$$$...` markers at size 0. Storage refuses a zero-byte contribution, so the adapter skips them, as the GNU adapter does.
8. **The load region's `Size` is authoritative for Flash.** armlink's "Total ROM Size" can be a few bytes smaller, because it leaves out alignment padding between execution regions (4 bytes in `rtx-blinky`).

## Amendment A2 (2026-10-18): Compare pairs a repeated section name by role

This closes the follow-up in §Consequences. When a section name repeats in a build, its rows are paired by role (`InitializedData`, `UninitializedData`, ...) if each role occurs at most once per side under that name. armlink's RW data and ZI parts of `RW_IRAM1` then pair as two ordinary rows. A role that repeats under one name still leaves all of that name's rows unpaired and ambiguous, as before; nothing is paired by position. The rule also applies when the name exists on one side only: rows with distinct roles are then plain additions or removals, not ambiguous ones. A section whose role changed under a repeated name shows as one removal and one addition. No GNU ld golden changed under A2. The diff schema is unchanged.
