---
title: "ADR-0038: IAR ILINK MAP adapter"
doc_id: "FS-ADR-0038"
product: "FirmwareSight"
version: "0.7.0"
status: "BASELINE"
owner: "Engineering"
last_updated: "2026-10-18"
---

# ADR-0038: Read the IAR ILINK map (IAR Embedded Workbench for Arm)

## Status

**BASELINE** (2026-10-18).
- The Owner asked for IAR support and, on 2026-10-18, directed that real samples be taken from public open-source projects instead of installing IAR.
- Standing Owner rule 3 ("no Keil or IAR support claim without real vendor samples") is met by nine same-origin builds from GitHub ([evidence/D4-IAR](../evidence/D4-IAR/README.md)).
- The adapter follows `04_TECH/39`, under which each linker gets its own grammar, and ADR-0033 for object attribution.

## Context

- IAR Embedded Workbench for Arm is the third common Cortex-M toolchain. Its linker, ILINK, writes standard ELF (`.out`), which FirmwareSight already reads. Its map was refused with `ERR-MAP-3001`.
- Many open-source projects commit their `EWARM/<config>/Exe/*.out` and `List/*.map` to Git. Nine pairs were found from permissively licensed repositories, each changed in one commit. They cover ILINK 7.50, 8.11, 8.20, 8.30, 8.40, 9.10 and 9.20.
- The ILINK map has three parts this adapter reads:
  - the `PLACEMENT SUMMARY` directives, such as `"P1": place in [from 0x800'0000 to 0x807'ffff] { ro };`;
  - the table of placed sections (`Section  Kind  Address  Size  Object`);
  - the `[n] = ...` index of object directories and libraries.
  ILINK 8 and later group hex digits with `'`.

## Decision

1. **A separate adapter** (`map_iar.rs`, adapter id `iar_ilink`, parser id `map-adapter/iar_ilink`).
   - `map::parse_any` sends any text that names IAR's linker to it.
   - The adapter reads a map with ILINK's banner and a placement summary.
   - An ILINK map for another target (STM8, RISC-V, RX…) is refused with that target named.
   - An IAR file without a placement summary is refused as before.
2. **Regions are the `place in` address ranges** (`[from A to B]` or `[from A size N]`):
   - a range holding an `inited`, `zero`, `uninit`, `noinit` or `rw code` row is RAM;
   - a range of only `ro code` and `const` rows is Flash;
   - an empty range is of Unknown kind;
   - the kind is derived from what was placed in the range, as armlink's is (ADR-0036 decision 2);
   - ranges that share any address (two directives over one range, or one range nested in another) are one region over their union, named after every directive in it (`P1+P3`), so no capacity is counted twice;
   - disjoint ranges of one directive are numbered (`P2`, `P2 (2)`);
   - `place at` fixes an address inside a range and is not a region;
   - a map with no `place in` range, with no section table, or with more than 4096 ranges is a parse error, because a capacity is never guessed.
3. **No placements.** IAR copy-initializes RW data from `Initializer bytes` stored in the read-only block, so the ELF's program headers already state what Flash holds. The map adds no load address.
4. **Object rows** follow ADR-0033:
   - a library member is `lib.a(member.o)`, taken from the `[n]` index;
   - an object from an object directory is its bare name, never a host path;
   - `Initializer bytes`, `- Linker created -` rows, `<Block>` headers and zero-size rows are not objects' bytes.
5. **Output sections by address.** IAR names its ELF sections after placement blocks (`P1 ro`, `P2 zi`, `Absolute sections 1`), and the map does not print those names. After an IAR map is parsed, and only then, the pipeline gives each row the smallest allocated ELF section that holds the whole row (`map_iar::resolve_output_sections`). A row that no section holds keeps no output section, and its charge is Unknown. On the nine samples every row resolved.
6. **Not changed:**
   - the analysis, diff and gate schemas;
   - storage;
   - the GNU and armlink adapters;
   - the `LayoutSource` vocabulary. `map` means "a supported MAP adapter produced the region table".

## Consequences

- **Support per layer** (`04_TECH/39`): the IAR (ILINK, for Arm) ELF, MAP regions, MAP-backed accounting, object and library attribution, and Compare are supported, backed by nine real samples.
  - Flash matches IAR's totals on all nine.
  - RAM is 0–3 bytes higher, from the alignment padding at the end of zero-init sections.
  - Every module IAR lists is attributed. Module RAM matches.
  - Module Flash differs only by IAR's per-module share of the initializer bytes, which FirmwareSight keeps with the linker's bytes.
- **Limits:**
  - Region names are placement labels, not `.icf` region names.
  - Absolute-address sections outside every range have no capacity.
  - Object keys are not qualified by object directory.
  - The samples are STM32 builds with IAR's default runtime.
- **Wording:** user-visible text names "GNU ld, armlink or IAR ILINK", where it used to say that IAR was not supported.

## Review (2026-10-18)

The independent review returned PASS_WITH_FIXES with no S1. Every finding was fixed:

**S2:**
- region kind is stated as derived;
- section resolution runs only for the IAR adapter and needs the whole row inside the section;
- overlapping and nested ranges merge, and repeated labels are numbered.

**S3:**
- an unterminated directive is still read;
- a missing section table is an error;
- ranges are capped at 4096;
- the loose "IAR Systems" marker no longer routes a file to this adapter;
- tests now cover non-ASCII text, overlap and overrunning rows.

## Alternatives rejected

- **Install the IAR evaluation and build our own samples:** the download requires a personal registration, and the licence is non-commercial and 14 days long. Committed public builds are the same linker's real output, and anyone can fetch them again by commit.
- **Commit the samples as fixtures:** they are third-party binaries. Their licences would allow it with notices, but the repository's fixtures stay first-party. The evidence names each sample by repository, commit, path and SHA-256, and the adapter's own tests use a synthetic map in ILINK's layout.
- **Charge initializer bytes to modules:** the map gives no per-module share once the initializers are compressed (lz77). Splitting them would be a guess.
