---
title: "P5 Commit E compatibility matrix"
doc_id: "FS-P5-COMMIT-E-MATRIX"
product: "FirmwareSight"
version: "1.1"
status: "MEASURED"
stage: "P5_PRODUCTIZATION"
owner: "Engineering"
last_updated: "2026-10-09"
---

# P5 Commit E — compatibility matrix (prompt §23, claim discipline §49, vocabulary §5 of the closure normalization)

Every row states the evidence class behind its word. `SUPPORTED` needs runtime or parser evidence on a
committed artifact; building a package is not running an app, so `CI_BUILD_ONLY` is a real and separate
status, and `NOT_TESTED` is not a failure — it is the honest absence of evidence.

**The status column uses exactly five values and no sixth: `SUPPORTED`, `SUPPORTED_WITH_LIMITS`,
`CI_BUILD_ONLY`, `NOT_TESTED`, `UNSUPPORTED`.** That is the closure normalization round's contract (§5/§6), and
it was not what the first version of this file did: it carried `BEST_EFFORT_NO_CLAIM`, `DEFERRED_NO_MVP`,
`NOT_CLAIMED`, `MEASURED, NOT INFERRED`, `BUILT_AND_VERIFIED_IN_CI` and half-sentences such as "`SUPPORTED` for
the region" in the status cell. Every one of those is now a canonical value, and what it said has moved to the
evidence column unchanged — the re-labeling changes no fact, and no row was upgraded or downgraded except where
the row's own evidence already described limits. Release-readiness states (`READY_NOT_EXECUTED`,
`UPDATE_READY_MANUAL`) are not compatibility claims, so §7b holds them in a column that is not called status.

Frozen boundary (§6): the supported cohort stays **GCC/Clang ELF + GNU ld MAP**. Keil and IAR remain
unsupported and nothing in this file widens that.

## 1. Platforms

| platform | status | evidence, named |
| --- | --- | --- |
| Windows x64, the measured host (Windows 10 19045) | `SUPPORTED_WITH_LIMITS` | Real install, real window, real uninstall and reinstall of the packaged NSIS build — `P5_INSTALL_RECOVERY_REPORT.md` §38/§39, and Commit D's §27 walk against the installer. The limits are the row's own: one host, one WebView2 (`Edg/154.0.4258.48`), 100 % DPI, which is L4's residue. The OS family and build are what this machine reported; nothing here says Windows 11. |
| Windows 10 x64 as a distinct configuration | `NOT_TESTED` | One host, one machine. `04_TECH/20:18` puts it at Tier 1 "Yes if Tauri/WebView support validated"; the validation this row asks for is a second machine, which does not exist here. |
| Windows 11 x64 | `NOT_TESTED` | Never run on it. The measured host is 19045. |
| macOS (Apple Silicon and Intel) | `CI_BUILD_ONLY` | A `.dmg` and a `.app` are built and indexed by the `Package macOS` job and verified file by file (`P5_PACKAGING_REPORT.md` §5d/§5e). No macOS window has ever been opened by this project. The owner fixed this row's status in the §5 checkpoint; §39 forbids upgrading it. |
| Ubuntu LTS x64 | `CI_BUILD_ONLY` | Same reasoning: the `Package Ubuntu` job builds and verifies a package; there is no Linux window evidence. |
| Other Linux distributions | `NOT_TESTED` | `04_TECH/20` Tier 3, which is a *no-claim* tier: no packaging job, no testing, no blanket statement. The absence of a claim is the claim this row makes. |
| Windows ARM64, Linux ARM64 | `UNSUPPORTED` | `04_TECH/20:23-24`. Deferred, outside the current MVP, not inferred, not tested, not claimed — the roadmap position belongs here, not in the status word. |

## 2. Compilers

| compiler | version as measured | status | evidence |
| --- | --- | --- | --- |
| `arm-none-eabi-gcc` (Arm GNU Toolchain 14.3.Rel1, build arm-14.174) | `14.3.1 20250623` | `SUPPORTED` | Ten committed ELF files: `p0-basic`, `p0-dual-region`, the `p2-diff` pair, and the new `ram-exec`, `extsram`, `dma-region`, `no-debug` (both files) and `long-preamble` fixtures. Each carries a `GCC:` banner in its own `.comment` section, asserted by `the_clang_fixture_names_clang_in_its_own_bytes_and_the_gcc_fixtures_name_gcc`. |
| `clang` with an explicit Arm target | `22.1.8` (git main `ca7933e47d3a`), built with `--target=arm-none-eabi -mcpu=cortex-m4 -mthumb` | `SUPPORTED_WITH_LIMITS` | One target, one optimization level, one linker: the limit is the cohort, not the compiler's identity. `fixtures/elf/p5-compat/clang-arm/`: one ELF and one GNU ld MAP. The compiler identity is proven from the artifact — its `.comment` names `clang version 22.1.8`, and the acceptance test refuses the fixture if it does not. Before this round the Clang half of the cohort claim had **zero** committed compiler-produced evidence (`P5_PRODUCTIZATION_AUDIT.md` §F); it now has one, green. |
| `clang` for other targets (`-mcpu` values, Apple, Windows-host clang) | — | `NOT_TESTED` | One target, one optimization level. §49 forbids promoting that to a family claim. |
| Keil ARM Compiler, IAR C/C++ | — | `UNSUPPORTED` | Frozen out by §6. No fixture, no adapter, no claim. |
| `gcc`/`g++` native x86_64 (host MinGW) | `16.2.0` | `NOT_TESTED` | Present on this host and deliberately never used for a fixture: it produces PE/COFF or native x86 ELF, and the desktop product is not asked to claim either. "Not tested for the product" is the whole of this row. |

## 3. Linkers and MAP producers

| linker | version | status | evidence |
| --- | --- | --- | --- |
| GNU ld (same Arm toolchain) | `2.44.0.20250616` | `SUPPORTED` | Nine committed MAP files across four distinct region shapes — `ROM`/`RAM`, `FLASH`/`RAM`, `FLASH`/`RAM`/`DMARAM`, `FLASH`/`RAM`/`EXTSRAM` — all parsed by the `gnu_ld` adapter, including one whose recognizable table starts at byte 16,571. |
| `ld.lld`, LLVM link | available on this host | `NOT_TESTED` | No committed lld-produced MAP. "GNU ld MAP" is the cohort; an lld banner is a different grammar and would be a scope change, not a compatibility win. |
| Keil/IAR map formats | — | `UNSUPPORTED` | §6. `foreign_toolchain_maps_are_rejected_without_falling_back` is the regression that keeps a foreign MAP from being quietly accepted. |

## 4. Architectures and artifact classes

| architecture / class | status | evidence |
| --- | --- | --- |
| Arm, Cortex-M class, Thumb, 32-bit little-endian | `SUPPORTED` | All eleven committed ELF files report `Machine: ARM`, `Class: ELF32`, little-endian, and the product's own artifact facts agree (`Arm` / `32` / `little`). |
| Cortex-M0/M3/M7, Cortex-A, Arm 64 (`AArch64`), RISC-V, MIPS, x86 | `NOT_TESTED` | One `-mcpu=cortex-m4` recipe. Recognition of an enum name in the domain model is not evidence that a fixture parses. |
| ELF (32-bit LE ARM here) | `SUPPORTED` | The whole fixture cohort, plus `fixtures/malformed/*` proving the refusals. |
| GNU ld MAP | `SUPPORTED` | Section 3 above. |
| HEX / BIN / UF2 / Mach-O / PE | `UNSUPPORTED` | Not in the cohort, not claimed. |
| DWARF semantic analysis | `UNSUPPORTED` | Not implemented, and not this round's to implement. Debug sections are **recognized and excluded**, never read for meaning. §49 forbids writing "DWARF supported" because a debug section parses, and `debug_sections_cost_real_bytes_and_enter_neither_budget` asserts that no capability claims DWARF. |

**Dated 2026-10-09, and this row's status is unchanged by that date.** `ADR-0030` was accepted on 2026-10-09 as product
direction and design for BIN and Intel HEX as **release-attached byte evidence** — raw bytes, `byte_size`, SHA-256 and a
declared kind, bound into the Gate required-artifact verdict and the release identity. It approves no code, so nothing in
this table moved and the `HEX / BIN` entry above stays `UNSUPPORTED` with its evidence sentence intact: there is still no
fixture, no attachment observation path and no runtime evidence. The status column keeps the five-value vocabulary the
closure normalization fixed it to; `SUPPORTED_WITH_LIMITS` is the word a later round would use, and only with an
attached-file fixture plus a measured run. Governance state words such as `DESIGN_APPROVED / NOT_IMPLEMENTED` belong to
`.ai/` and `BASELINE.yaml`, not to this column. Design: `04_TECH/28_RELEASE_ATTACHED_BYTE_EVIDENCE.md`.

## 5. Memory layout shapes (§26 cases A–H)

| case | status | evidence |
| --- | --- | --- |
| A classic FLASH + RAM | `SUPPORTED` | `fixtures/elf/p0-dual-region/` — regions named `ROM`/`RAM`, `.data`/`.ota` VMA-in-RAM, LMA-in-ROM; totals 160 / 72 asserted. Re-recorded as a Commit E assertion rather than duplicated as a fixture. |
| B executable code resident in RAM | `SUPPORTED` | `p5-ram-exec`: `.fast_text` VMA `0x20000004`, LMA `0x0800003c`, charged `12/12` from load evidence, totals 72 / 20. |
| C external SRAM | `SUPPORTED` | `p5-extsram`: declared `EXTSRAM 0x60000000 +0x80000`, `.extsram_pool` charged `128/128`, totals 184 / 136. |
| D DMA / custom writable region | `SUPPORTED_WITH_LIMITS` | Region and memory accounting is what is supported; cacheability semantics are explicitly **not** claimed and nothing in the model asserts them. `p5-dma-region`: `DMARAM 0x30000000 +0x4000`, `.dma_buffer` charged `64/64`, totals 120 / 72, and a test that fails if the model ever says `cache`, `coherent` or `non-cacheable`. |
| E long MAP preamble | `SUPPORTED` | `p5-long-preamble`: 220 dead functions discarded by `--gc-sections` push the banner to byte 16,571; detection, region table and object lines all still parse. |
| F ELF with debug info | `SUPPORTED_WITH_LIMITS` | Presence and acceptance are supported — debug-section bytes are measured, costed and excluded; DWARF **semantics** are not consumed, which is its own `UNSUPPORTED` row in section 4. Every GCC/Clang fixture is a `-g` build except `firmware-nodebug.elf`; the clang fixture carries `.debug_*` sections with real bytes that enter neither budget. |
| G no debug, and post-link strip | `SUPPORTED` | `p5-no-debug`: two files, two different absences — 17 symbols versus 0, both analyzed to the same exact 48 / 8 totals. |
| H multiple `PT_LOAD` segments | `SUPPORTED` | `2, 4, 4, 1, 2, 2, 2` for the new fixtures, `3` for `p0-dual-region`, `5` and `4` for the `p2-diff` pair — counted from the ELF program headers in `real_linkers_emit_several_load_segments_and_the_fixtures_say_how_many`. |
| Unusual-but-allocated section kinds | `SUPPORTED` | `p5-clang-arm`'s `.ARM.exidx.text.main` (8 bytes, `SHF_ALLOC`, unrecognized role) is now charged to the image. This is the case the round found and fixed; see `P5_COMMIT_E_DESIGN.md`. |

## 6. Debug, symbols and the states between them

| condition | status | evidence |
| --- | --- | --- |
| Built with `-g` | `SUPPORTED` | Every GCC and Clang fixture except `firmware-nodebug.elf`. |
| Built without `-g` | `SUPPORTED` | `firmware-nodebug.elf`, symbol table intact. |
| Post-link stripped | `SUPPORTED` | `firmware-stripped.elf`, no symbol table; 0 symbols reported honestly rather than as zero-and-silence. |
| Stripped with `objcopy -S` | `SUPPORTED` | **Measured on this host, not inferred from the tool's documentation**: on this binutils, `-S` removed `.debug_*` **and** `.symtab`/`.strtab`, leaving only `.shstrtab`. Recorded because the difference between "no debug" and "no symbols" is what case G exists to keep apart. |

## 7. Runtime evidence versus CI evidence

| surface | status | why |
| --- | --- | --- |
| Desktop app, installed on Windows x64 | `SUPPORTED_WITH_LIMITS` | Install/uninstall/reinstall walked on this host against the packaged NSIS build; the limits are one host, one WebView2 and 100 % DPI (L4's carried residue), so the row cannot be a bare `SUPPORTED`. |
| CLI `fwsight`, this host | `SUPPORTED` | Every fixture-level claim above is reproducible through it; the golden set is generated from it. No GUI, so no DPI or WebView surface to bound. |
| CLI and desktop on macOS / Ubuntu | `CI_BUILD_ONLY` | The CI jobs compile and test the headless crates and build packages; §27/§39 forbid calling that a window. |
| Application package on macOS / Ubuntu | `CI_BUILD_ONLY` | Built and checksum-verified in CI (`P5_PACKAGING_REPORT.md` §5d/§5e), **never installed by anyone**: a package build is not runtime support, which is exactly why this file keeps the two rows apart. |

### 7b. Release readiness — states, not compatibility statuses

These are the fixed P5 §11/§12 dispositions. §18 of the Commit E prompt froze the vocabulary and this round
does not change it; they are listed here rather than in the table above because a readiness state is not a
support status and §6 forbids a sixth status value.

| dimension | state | what would change it |
| --- | --- | --- |
| Code signing | `READY_NOT_EXECUTED` | A key-management and signing decision, not a code change in this round. |
| Notarization | `READY_NOT_EXECUTED` | Same: nothing has been executed, and no certificate exists here. |
| Update channel | `UPDATE_READY_MANUAL` | Manual upgrade only; §12 forbids an updater, an endpoint and a manifest until the signing ADR is complete. |

## 8. What this matrix refuses to say

- Not "Clang supported" whole-cloth: Clang **for Arm at this target and this optimization level, linked by
  GNU ld** — one fixture, named.
- Not "macOS supported" or "Ubuntu supported": `CI_BUILD_ONLY`, with the package build as the only evidence.
- Not "DWARF supported": debug sections are recognized, measured and excluded; nothing reads their meaning.
- Not "all linker scripts" or "all Cortex-M parts": four controlled scripts, one `-mcpu`.
- Not "all GNU ld MAP shapes": the four layouts here plus one long-preamble MAP, and `foreign_toolchain_maps_are_rejected_without_falling_back` still refuses what it should.
- Not a Gate or release claim: P5 is `IN_PROGRESS`, `COMMIT_E` has no verdict written here, and the product
  state remains **MVP CANDIDATE** on baseline `0.6.0`.
