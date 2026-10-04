---
title: "P5 Commit E compatibility fixture report"
doc_id: "FS-P5-COMMIT-E-FIXTURES"
product: "FirmwareSight"
version: "1.0"
status: "MEASURED"
stage: "P5_PRODUCTIZATION"
owner: "Engineering"
last_updated: "2026-10-04"
---

# P5 Commit E — compatibility fixtures (prompt §24, with §25 depth and §26 safety)

Six new fixture sets under `fixtures/elf/p5-compat/`, one existing fixture pressed into service for
case A, 31 new `fixtures/manifest.json` entries (25 → 56), and 14 acceptance tests in
`crates/firmwaresight-artifact/tests/p5_compat_fixtures.rs`. Every number below was measured on this
host at start HEAD `bfbc1aa` and is reproduced by a test.

Two rules govern the whole file. §9: fixtures are real tool output, never hand-edited bytes. §49: every
support statement names its fixture, compiler, linker, architecture, layout and test — so this report
says what the evidence covers and, at the end, what it does not.

## 1. Toolchain identities actually used

| tool | version as measured | where the identity is provable |
| --- | --- | --- |
| `arm-none-eabi-gcc` | `Arm GNU Toolchain 14.3.Rel1 (Build arm-14.174)) 14.3.1 20250623` | the ELF's own `.comment` (`readelf -p .comment`), repeated in each `fixture.toml` as `compiler_in_artifact` |
| `arm-none-eabi-ld` | `GNU ld (Arm GNU Toolchain 14.3.Rel1 (Build arm-14.174)) 2.44.0.20250616` | the MAP's first lines, and `linker =` in each record |
| `clang` | `clang version 22.1.8 (https://github.com/llvm/llvm-project ca7933e47d3a…)`, host default target `x86_64-pc-windows-msvc`, cross-invoked with `--target=arm-none-eabi` | the ELF's `.comment` — `the_clang_fixture_names_clang_in_its_own_bytes_and_the_gcc_fixtures_name_gcc` |

Compiler identity is read out of the artifact rather than asserted by the record. That is what makes a
relabelled fixture fail: a GCC-built ELF names `GCC:` in its own bytes whatever its `fixture.toml`
claims, and mutation proof **B** (handing the GCC binary the clang record) reddened that test.

## 2. The derivation the records carry, and why it is independent

Each `fixture.toml` now carries (prompt §11): `entry_point`, `expected_image_bytes`,
`expected_live_ram_bytes`, `expected_product_evidence_basis`, `expected_product_unknowns`, plus
`load_segments` and `allocatable_sections`.

Those figures come from `scripts/gen_p5_compat_fixtures.py`, which computes them from **two tool
outputs and nothing else**:

- image bytes = every `SHF_ALLOC` section that is not `SHT_NOBITS`, from `readelf -S -W` on the file;
- live RAM bytes = every `SHF_ALLOC` section whose VMA falls inside a region the **MAP's own
  `Memory Configuration` table** declares writable (`w` in its attribute column).

The rule is the one `04_TECH/23_MEMORY_ACCOUNTING_MODEL.md` states, and the `*default*` row is skipped
because it spans the whole address space and would make every section look at home everywhere. No
section name participates in either sum, so a record cannot inherit the analyzer's assumptions.

Three checks that this derivation is not fitted to the new fixtures — it reproduces figures that
*pre-date* it and are asserted by tests this round never edited:

| pre-existing fixture | rule derives | already asserted at |
| --- | --- | --- |
| `p0-dual-region` | image `160 = 60 .text + 32 .rodata + 4 .data + 64 .ota`, live RAM `72 = 4 .data + 4 .bss + 64 .ota` | `crates/firmwaresight-artifact/tests/p0_acceptance.rs` (`the_classic_flash_and_ram_case_needs_no_new_fixture` repeats it) |
| `p2-diff/base` | image `256 = 124 .text + 32 .rodata + 64 .settings + 4 .data + 32 .calib`, live RAM `8` | `p2_fixture_pair.rs:336,338` |
| `p2-diff/target` | image `376 = 212 .text + 32 .rodata + 64 .settings + 4 .data + 64 .ota`, live RAM `76 = 4 .data + 8 .bss + 64 .ota` | `p2_fixture_pair.rs:337,339` |

And `the_provenance_record_and_the_product_answer_the_same_numbers` reads each record's numbers back out
of the TOML at test time and holds the analyzer to them, so a record that drifts from the product reddens
a test in either direction.

## 3. Per-fixture record (§24's field list)

All six are ELF32 little-endian ARM, `e_machine = 40`, entry `0x08000001` (`-e main`, thumb bit set by
the ABI's mapping-symbol convention), linked by GNU ld 2.44.0.20250616 with a controlled linker script.
"Expected" is §2's derivation; "actual" is `fwsight analyze <elf> --map <map> --json` on the rebuilt CLI.

### 3.1 `p5-ram-exec` — case B, executable resident in RAM

| field | value |
| --- | --- |
| files | `firmware.elf`, `firmware.map`, `ram-exec.ld`, `source/main.c`, `fixture.toml` |
| compiler / version | `arm-none-eabi-gcc` / Arm GNU Toolchain 14.3.Rel1, 14.3.1 20250623 |
| linker / version | `GNU ld (Arm GNU Toolchain 14.3.Rel1 (Build arm-14.174)) 2.44.0.20250616` |
| target | `-mcpu=cortex-m4 -mthumb -g -Os -ffunction-sections -fdata-sections` |
| generation command | `arm-none-eabi-gcc … -c source/main.c -o build/main.o`, then `arm-none-eabi-ld -e main -Tram-exec.ld build/main.o -o firmware.elf -Map=firmware.map` |
| ELF SHA-256 | `54e77ae42cb17c7d17366ef2461aaa7b78337eb7c9edb84e0b8945828400642f` |
| MAP SHA-256 | `2891c91004f30cb104b9face1f617685a69ae59146ebf7d77535885423021b41` |
| expected architecture | ARM (cortex-m4, thumb, 32-bit little-endian) |
| entry | `0x08000001` |
| PT_LOAD | 2 |
| expected memory | image `72 = 24 .text + 32 .rodata + 4 .data + 12 .fast_text`; live RAM `20 = 4 .data + 12 .fast_text + 4 .bss` |
| expected basis | `map-memory-configuration+elf-load` |
| expected Unknowns | build id (no `.note.gnu.build-id`); git context (one artifact, no project); why `.fast_text` is in RAM — the script places it, the file does not say for what |
| actual result | nv 72, runtime RAM 20, basis `MapRegionAndElfLoad`, `.fast_text` VMA `0x20000004` / LMA `0x0800003c` with `execute` set, 18 sections, 29 symbols — all matched |
| test | `code_resident_in_ram_is_charged_to_ram_by_load_evidence_not_by_its_name` |
| limitations | one controlled linker script, not a vendor SDK overlay; proves the rule on this evidence only, not on every toolchain |

The point of the case: `.fast_text` carries `SHF_ALLOC|SHF_EXECINSTR` and **no** `SHF_WRITE`, so a
section-attribute rule alone would charge it to nothing. It is live RAM because the region its VMA
sits in is declared `xrw`.

### 3.2 `p5-extsram` — case C, external SRAM

| field | value |
| --- | --- |
| files | `firmware.elf`, `firmware.map`, `extsram.ld`, `source/main.c`, `fixture.toml` |
| compiler / linker / target | as 3.1, `-T extsram.ld` |
| ELF SHA-256 | `cd6c269e4586c88bbe934784437b96bf57ffd9c6c55fc286b3ab5a049c7bae43` |
| MAP SHA-256 | `e31912d757cd2bd3591ff5b3ec03299583711828ebf7857f6aa61a823b2fa0f4` |
| entry / PT_LOAD | `0x08000001` / 4 |
| expected memory | image `184 = 20 .text + 32 .rodata + 4 .data + 128 .extsram_pool`; live RAM `136 = 4 .data + 128 .extsram_pool + 4 .bss` |
| expected basis / Unknowns | as 3.1, plus: what technology sits at `0x60000000` — the MAP gives an address range and attributes, nothing more |
| actual result | nv 184, runtime RAM 136, `EXTSRAM` observed at `0x60000000` length `0x80000`, pool charged `128/128` with basis `MapRegionAndElfLoad`, 17 sections, 27 symbols |
| test | `an_external_sram_region_is_observed_from_the_map_and_charged_to_runtime_ram` |
| limitations | the address is conventional for an external bus, not tied to a specific part; proves a declared region is accounted from real MAP evidence, not that every vendor map is covered |

No internal-RAM assumption reaches `0x6000_0000`, so this case fails closed if the model ever starts
guessing RAM by address range.

### 3.3 `p5-dma-region` — case D, custom-named writable region

| field | value |
| --- | --- |
| files | `firmware.elf`, `firmware.map`, `dma-region.ld`, `source/main.c`, `fixture.toml` |
| ELF SHA-256 | `31c9c638b1787495430fa64e8a788f794b27ba27a8b0b2250e61e5e7ad698214` |
| MAP SHA-256 | `1bc4c628d34990edf7d16812e6c5505a4ee8a262f112ff4fa9b779683433ce94` |
| entry / PT_LOAD | `0x08000001` / 4 |
| expected memory | image `120 = 20 .text + 32 .rodata + 4 .data + 64 .dma_buffer`; live RAM `72 = 4 .data + 64 .dma_buffer + 4 .bss` |
| expected basis / Unknowns | as 3.1, plus: cacheability and coherency, which a MAP region table cannot express at all — it declares `r/w/x` |
| actual result | `DMARAM` observed at `0x30000000` length `0x4000`; buffer charged `64/64`; nv 120, runtime RAM 72, 17 sections, 27 symbols |
| test | `a_custom_named_writable_region_is_not_lost_and_no_cacheability_is_claimed` — which also greps the whole memory model for `cache`, `coherent`, `non-cacheable` and fails if any appears |
| limitations | Commit E adds no cacheability dimension (§17): the section name says DMA, the evidence says writable RAM, and no report may upgrade one into the other; one region, one layout, not a survey of DMA-capable maps |

### 3.4 `p5-long-preamble` — case E, a GNU ld MAP whose banner starts late

| field | value |
| --- | --- |
| files | `firmware.elf`, `firmware.map`, `plain.ld`, `source/main.c`, `fixture.toml` |
| recipe | 220 unreferenced functions, discarded by `--gc-sections`; the preamble is linker output, not pasted filler |
| ELF SHA-256 | `8552c799084594e87d438ccce5691cdda40c65b79f3637196d4de3187ca14e74` |
| MAP SHA-256 | `ef65bd002673c78897dfca148ad2487c4b68da3faa3479608ab0b7bd16f13316` |
| banner offset | `Memory Configuration` first appears at byte **16571**; re-measured on every regeneration, and the generator refuses to emit below a 4096-byte floor |
| entry / PT_LOAD | `0x08000001` / **1** (only `main` survives `--gc-sections`, so this is the one single-segment fixture; case H is carried by the others) |
| expected memory | image `4 = 4 .text`; live RAM `0 = none` |
| actual result | `map::detect` true, `adapter_id = "gnu_ld"`, both `FLASH` and `RAM` regions parsed from past the preamble, object contributions non-empty, nv 4, runtime RAM 0, 14 sections, 15 symbols |
| test | `a_map_whose_banner_sits_past_every_fixed_window_still_parses_and_analyzes` |
| limitations | preamble length is a property of this source and this linker version; it is a regression input for the F003 class (fixed-head reads missing the table), not a claim about arbitrary MAP files |

### 3.5 `p5-clang-arm` — the Clang half of the cohort, and the defect it found

| field | value |
| --- | --- |
| files | `firmware.elf`, `firmware.map`, `plain.ld`, `source/main.c`, `fixture.toml` |
| compiler / version | `clang version 22.1.8`, cross-invoked `--target=arm-none-eabi -ffreestanding -fno-builtin -mcpu=cortex-m4 -mthumb -g -Os -ffunction-sections -fdata-sections` |
| linker / version | GNU ld 2.44.0.20250616 (the Arm toolchain's `arm-none-eabi-ld`) — the final link and the MAP are GNU ld, so the MAP stays inside the supported adapter |
| ELF SHA-256 | `722bf7e20f1d731a35bcd111e5d10f49a70a882a9fc3187b7dd0765d667b85f4` |
| MAP SHA-256 | `39a7eb5492d881cf55ef0a64e56af58982baf54bb70d7d367644dea9d23831c1` |
| entry / PT_LOAD | `0x08000001` / 2 |
| expected memory | image `58 = 14 .text + 32 .rodata + 8 .ARM.exidx.text.main + 4 .data`; live RAM `8 = 4 .data + 4 .bss` |
| expected Unknowns | build id; git context; the role of `.ARM.exidx.text.main`, which the parser still does not recognise — it is charged from region and load evidence, not from a name it knows |
| actual result | nv 58, runtime RAM 8, exidx charged `8/0` with basis `MapRegionAndElfLoad` and `role = Unknown`, 8 `.debug_*` sections excluded as metadata (`excludedMetadataBytes` 1633), 18 sections, 25 symbols |
| tests | `an_allocated_section_the_parser_does_not_recognise_is_still_charged`, `debug_sections_cost_real_bytes_and_enter_neither_budget`, `the_clang_fixture_names_clang_in_its_own_bytes_and_the_gcc_fixtures_name_gcc` |
| limitations | proves clang-compiled Arm code at this target, this optimization level and with this linker; not a survey of `clang -mcpu` values, not an Apple or Windows-target clang, not an `lld` link |

**This fixture is how the round found the undercount.** clang emits `.ARM.exidx.text.main` for `main`;
GNU ld keeps it inside the declared `FLASH` region with `SHF_ALLOC` set; the model inferred
"not allocated" from "unrecognised section kind", so eight real image bytes entered neither budget
while the total still reported `Exact` with nothing in `unattributed`. Fixed narrowly in
`crates/firmwaresight-artifact/src/elf.rs` by reading `SHF_ALLOC` from the section header, with
mutation proofs in §5 below and the disposition row in `P5_SUPPORTABILITY_REPORT.md`.

### 3.6 `p5-no-debug` — case G, two different absences kept apart

| field | value |
| --- | --- |
| files | `firmware-nodebug.elf`, `firmware-stripped.elf`, `firmware.map`, `plain.ld`, `source/main.c`, `fixture.toml` |
| recipe | `-g -Os` build linked to `with-debug.elf`, then `arm-none-eabi-objcopy -S with-debug.elf firmware-stripped.elf`; separately a `-Os` (no `-g`) build linked to `firmware-nodebug.elf` with the MAP. The intermediate is deliberately not committed |
| ELF SHA-256 (nodebug) | `9f50445b7f2316e187c9f298e83cc1fbe120794a5516d24920438694bd9e218c` |
| ELF SHA-256 (stripped) | `f7a72eddabb9fdd24eba0bc4240eb763896227958d54b2fb2058f94771760711` |
| MAP SHA-256 | `616c0e35544efd586f7a11534c5ea708c914e95f5b645cb40d859fd382a4de35` |
| entry / PT_LOAD | `0x08000001` / 2 for both files |
| expected memory | nodebug: image `48 = 12 .text + 32 .rodata + 4 .data`, live RAM `8 = 4 .data + 4 .bss`; the record's `second_file_expectations` row carries the stripped file's same derivation |
| actual result | nodebug: 0 `.debug_*` sections, **17 symbols**, `.symtab` and `.strtab` present; stripped: 0 `.debug_*`, **0 symbols**, only `.shstrtab` left. Both report Elf32Le, nv 48, runtime RAM 8, exact totals, `.text` VMA `0x08000000` |
| test | `a_build_without_debug_and_a_post_link_strip_are_two_different_facts` |
| limitations | measured, not assumed: on this binutils `objcopy -S` removed the `.debug_*` sections **and** `.symtab`/`.strtab`, so the pair differs in two ways and a test that reads one as the other would be reading its own assumption |

### 3.7 Case A, classic FLASH + RAM — no new fixture

`fixtures/elf/p0-dual-region/firmware.elf` (`c6d0feed…4e62`) already is this case, with its regions
named `ROM (rx)` / `RAM (rwx)` rather than `FLASH`/`RAM`. It is recorded here rather than duplicated:
nv 160, runtime RAM 72, `.data` dual-charged `4/4`, `.bss` `4`/`0`, `.ota` `64/64`, all with basis
`MapRegionAndElfLoad`, asserted by `the_classic_flash_and_ram_case_needs_no_new_fixture` against §14's
six bullets. Adding a second classic fixture would have grown the cohort without growing the evidence.

## 4. §13 A–H disposition

| case | disposition | evidence |
| --- | --- | --- |
| A classic FLASH + RAM | covered, no new fixture | `p0-dual-region`, §3.7 |
| B executable-in-RAM | **new** | `p5-ram-exec` |
| C external SRAM | **new** | `p5-extsram` |
| D DMA / custom writable region | **new**, cacheability explicitly not claimed | `p5-dma-region` |
| E long MAP preamble | **new**, real discarded sections | `p5-long-preamble` |
| F ELF with debug info | already covered (`-g` on every fixture); recognized, not consumed | `p5-clang-arm` deepens it: 8 debug sections, excluded, no DWARF claim |
| G stripped / no debug | **new** | `p5-no-debug` (two files, two absences) |
| H multiple `PT_LOAD` | **new and named** | counts recorded per fixture (2, 4, 4, 1, 2, 2, 2) and asserted by `real_linkers_emit_several_load_segments_and_the_fixtures_say_how_many`; `p0-dual-region` = 3 and `p2-diff` pair = 5 and 4 |

## 5. Acceptance-test depth (§25) and the mutation proofs (§44)

`p5_compat_fixtures.rs` holds 14 tests. Coverage of §25's list: artifact kind, architecture, bitness and
endianness and entry point for **every** committed ELF (`the_identity_the_product_reports_is_the_identity_
the_file_declares`, read against the header bytes, not the model); section facts (VMA, LMA, flags, role,
`has_file_payload`); symbol counts; memory-region facts (name, origin, length); both totals; evidence
basis; the Unknown ledger (build id reason, absent symbol table, unrecognized role); MAP adapter
identity; `PT_LOAD` counts; the debug / no-debug distinction. Image footprint is checked against the
file's own `SHF_ALLOC` payload for every manifest ELF
(`the_reported_image_footprint_equals_the_bytes_the_file_claims_are_loaded`). No test asserts
incidental ordering: contributions are always found by section name.

| proof | mutation | reddened | restored |
| --- | --- | --- | --- |
| A | undo the `SHF_ALLOC` fix in `elf.rs` | `an_allocated_section_…` and `the_reported_image_footprint_…`, with `left: 50, right: 58` | byte-exact |
| B | give the clang ELF the GCC binary while keeping the clang record | `the_clang_fixture_names_clang_in_its_own_bytes_…` | byte-exact |
| C | perturb one recorded total (184 → 185) | `an_external_sram_region_is_observed_…` | byte-exact |
| E | read `SHF_WRITE` where the fix reads `SHF_ALLOC` | 8 tests across the workspace | byte-exact |

Restoration was checked by digest, and the suite re-run green afterwards.

## 6. Host-path and hash safety (§26)

- Build paths are **relative** (`source/main.c` → `build/main.o`) because GNU ld copies the object paths
  it was given into the MAP; the recipe refuses to emit an absolute path token anywhere in a committed
  file (`assert_no_host_paths` scans every textual output, and it fired during development on an earlier
  draft of this generator).
- No username, no temp directory, no build-machine drive path: `grep` over the six MAPs and five `.ld`
  files returns nothing matching `^[A-Za-z]:[\\/]`, `/Users/`, `/home/` or `/tmp/`.
- Every committed path is in `fixtures/manifest.json` (31 new entries, 56 total) and every one is
  `git add`-tracked; `drift/fixtures tracked` is green, which is the guard that exists because
  `.gitignore` once ate half of the `p2-diff` pair.
- `.gitattributes` needed no change: `*.elf binary`, `*.map -text` and `text eol=lf` for `.ld`, `.c`,
  `.toml` already make a fresh checkout reproduce these bytes on any platform, which is what the
  manifest's "hash of the committed blob" note promises.

## 7. Regeneration reproducibility (§51)

All required tools are present on this host, so the check was run rather than deferred:

1. `python scripts/gen_p5_compat_fixtures.py --force` over the six sets, twice (once before and once
   after extending the metadata template).
2. **Result: all 13 ELF and MAP files came back byte-identical** (`identical=13 differing=0 missing=0`
   against the digests recorded before the run), including the 16,571-byte banner offset, which the
   generator re-measures instead of trusting.
3. The compiler embeds nothing volatile in these outputs at this recipe: no build-id (there is no
   `.note.gnu.build-id` section to change), and the DWARF line paths are the relative ones the recipe
   passed. No deterministic-prefix flag was needed, and no final artifact byte was patched.
4. After the template extension, the six `fixture.toml` records and their manifest digests moved and
   **no binary or MAP byte moved** — checked with `git diff --name-only` over the fixture tree.

## 7a. Clean detached worktree (§52), and what it did not need

Run against the candidate head before the push, in `target/wt-p5e` created by
`git worktree add --detach target/wt-p5e <candidate>` and removed afterwards:

| check | result |
| --- | --- |
| checkout state | detached at the candidate SHA, `git status --short` empty |
| manifest paths present | **56 of 56 exist**, `missing=0` — no generator was run in the worktree |
| manifest paths tracked | `untracked=0` against the worktree's own `git ls-files` |
| committed hashes | `hash_mismatch=0`: each file's SHA-256 equals the manifest's `sha256` field, byte for byte, from a fresh checkout |
| no local compiler needed | `cargo test -p firmwaresight-artifact --test p5_compat_fixtures` ran with `arm-none-eabi-gcc`, `clang`, `arm-none-eabi-readelf` and `ld.lld` all **absent from `PATH`**: the PATH was built by filtering the Arm toolchain and LLVM directories out, and `command -v` in that same environment resolves all four to nothing while `cargo`, `rustc`, `node`, `corepack` and `python` still resolve. The test command's own first log line is `toolchain absent, run valid`, so the precondition sits next to the result instead of being assumed. **14 passed / 0 failed** |
| full gate in the worktree | `python scripts/check.py` **18 of 18** — the same 16 steps as the main tree plus the two `rust/frontend assets (install)` and `(build)` preparatory steps a cold checkout needs before the desktop crate compiles. `cargo test --workspace` there reported **868 passed across 47 targets** and the UI suite **217 in 8 files**, identical to the main tree |
| package group | **NOT_RUN in the worktree**, stated rather than glossed. Rebuilding the installer in a second tree would re-measure a step already measured on identical bytes; the package authority for this head is the local `--only package` **4 of 4 with no `SKIP`** plus the three `Package` jobs of Run `37228929762` at head `59d85c3` — `Build and verify the …package` and `Upload the artifact set` green on Windows, Ubuntu and macOS — read in `P5_CI_AUTHORITY.md` |

Why this section exists at all: `.gitignore` once ate half of the `p2-diff` fixture pair, the manifest
recorded both halves, every local run digested the bytes the generator had left on disk, and CI — which
clones — was the only thing that disagreed. `drift/fixtures tracked` closes the "listed but untracked"
case, and this worktree check is the stronger form: it shows the tracked bytes are also the *right* bytes
with nothing on disk but what a clone produces.

Note the boundary §57 draws around the other artifact: nothing here claims that the **root `SHA256SUMS`**
verifies in a clean checkout. That is L26, deliberately untouched by this round, and it concerns working-copy
bytes of CRLF-affected text files, not these fixtures.

## 8. What an ordinary test run needs

**Nothing beyond the repository.** The fixtures are committed binaries, so
`cargo test -p firmwaresight-artifact` (and therefore `python scripts/check.py`) runs all 14 acceptance
tests with no compiler, no linker, no `readelf` and no `arm-none-eabi-*` installation. Only
*regenerating* them needs the toolchain, and `scripts/gen_p5_compat_fixtures.py` says so in its own
error message when a tool is missing.

## 9. What this evidence does not support (§49)

- Not "Clang supported" in general: one clang 22.1.8 Arm ELF, one target, `-Os`, final-linked by GNU ld.
  `lld` produced nothing here and is not claimed.
- Not "all linker scripts supported": four controlled scripts (`ram-exec`, `extsram`, `dma-region`,
  `plain`) with every allocatable section explicitly placed.
- Not "all Cortex-M parts supported": `-mcpu=cortex-m4` only, and architecture coverage remains ARM.
- Not "DWARF understood": debug sections are recognized, counted and excluded; nothing reads them.
- Not a Keil or IAR claim, and not a Windows/macOS/Linux runtime-support claim — those live in
  `P5_COMPATIBILITY_MATRIX.md` with their own evidence, and macOS/Linux stay `CI_BUILD_ONLY`.
- Not `unattributed == ∅` as a general property: it is checked for these fixtures, where every
  allocated section lands inside a declared region. A real image that violates that is a finding, and
  the generator now refuses to record such a case silently.
