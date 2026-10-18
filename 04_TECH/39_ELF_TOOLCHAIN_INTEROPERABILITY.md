# ELF interoperability foundation — NEXT-46, 2026-10-14

This matrix describes measured format evidence, not a vendor endorsement. A filename ending in `.axf` or `.out` does not establish its container, compiler or linker. Analyze accepts ELF magic/class/endianness; an explicitly supplied unsupported MAP fails with `ERR-MAP-3001`. Remove that MAP to analyze compatible ELF alone. It is never silently ignored.

## Evidence levels

| Level | Meaning | Current evidence |
|---|---|---|
| RECOGNIZED_CONTAINER | Magic, ELF class and byte order accepted by bounded intake | ELF32/ELF64, little/big endian |
| PARSED_BASIC_METADATA | `object` parses headers, entry, sections, file offsets/payload, symbols and build-id where present | Four Clang relocatable fixtures in `fixtures/elf/interop-clang`, plus existing linked ARM fixtures |
| VERIFIED_MEMORY_ACCOUNTING | Existing Core model tested against explicit supported placement evidence | Existing ARM GCC/GNU ld and Clang/GNU ld cohorts; new relocatable fixtures do **not** establish executable accounting |
| VERIFIED_COMPILER_COHORT | Actual toolchain output, recipe/provenance and independent inspection | GCC + GNU ld; Clang + GNU ld. Keil/armlink and IAR/ILINK: **UNVERIFIED** as a cohort. One synthetic Arm Compiler 6.24 sample reads and accounts correctly from the ELF alone ([evidence/D4](../evidence/D4/README.md)); since ADR-0036 A1, two real µVision projects agree with armlink's totals ([evidence/D4-REAL](../evidence/D4-REAL/README.md)). Three samples are broader evidence, still not a large cohort |
| LINKER_MAP_SUPPORTED | Dedicated grammar supplies required fields with independently validated fixtures | GNU ld; **armlink** since ADR-0036 (fixture `fixtures/elf/d4-armlink`, cross-checked against armlink `--info=totals` and GNU readelf). IAR ILINK: **NOT_SUPPORTED / FIXTURE_BLOCKED** |

## Parser facts and limits

| Fact | Measured behavior | Boundary |
|---|---|---|
| ELF class / byte order | Clang 22.1.8 produced ARM32 LE/BE and AArch64 LE/BE relocatable objects; `llvm-readobj` and raw header checks agree with parser | These are genuine Clang objects, not renamed vendor output |
| Architecture | ARM, AArch64, x86-64 and MIPS map to existing identity variants; other architectures retain `Other` | This round verifies ARM/AArch64 class and endian combinations; other mappings are code presence, not new cohort verification |
| Sections | Name, role, alloc/write/execute flags, VMA, file offset, file size, memory size | A section header has no LMA; absent load address stays Unknown |
| Symbols | Present tables supply name, address, size, kind, binding and section reference | Missing/stripped symbols degrade capability; source/module attribution is not inferred from names |
| Debug | Known debug-section names produce presence/availability metadata | No DWARF unit/source/type decoder is implemented by `elf.rs`; presence is not decoded attribution |
| RAM initialization | Supported GNU MAP supplies explicit load address; Core keeps nonvolatile bytes and runtime bytes separate | ELF-only address/flags basis is qualified. No target region capacity, allocator headroom or stack/heap guarantee is invented |
| Relocatable objects | Basic metadata parses; the new fixture `.data` is 4 stored bytes and `.bss` is 4 memory bytes / 0 stored bytes | Address-zero objects have no linked deployment layout and no vendor/executable utilization claim |
| Loading | Stat/size guard then bounded same-handle buffer and hash | Default 512 MiB ceiling; bad encoding, oversized, malformed and truncated inputs are tested |

Recipe: `python scripts/gen_interop_fixtures.py <new-fixture-directory> <new-external-evidence-directory>`. The committed provenance records compiler identity, relative commands, bytes and SHA-256. Independent readout logs remain outside Git in the delivery evidence. Tests need no installed toolchain; regeneration requires existing Clang and llvm-readobj. The generator refuses to replace either existing directory.

## MAP admission design and refusal harness

Since ADR-0036 two adapters exist, GNU ld and armlink, chosen by `map::parse_any` from the banner; the GNU adapter's own detection still refuses armlink text. Before ADR-0036 the GNU adapter was the only parser. Detection returns the existing typed `MapUnsupported` error for foreign banners **before** evaluating GNU markers, including mixed/concatenated text and long preambles. Supported GNU positives and frozen fixtures remain unchanged. Detection/refusal is not an ArmLink or IAR parser.

Required normalization facts are memory-region origin and capacity, output-section VMA/size, explicit LMA where provable, and optional object/symbol attribution. A future adapter must validate each field against an independent companion ELF or vendor inspection result. Missing fields must preserve Unknown. No public schema, domain vocabulary or memory-accounting semantic change is authorized here.

| Grammar | Independent semantics | Adapter prerequisite / decision |
|---|---|---|
| GNU ld | `Memory Configuration`, column-zero output section, optional `load address`, input-object contributions | Existing `map.rs` retained; positive fixtures/regressions remain authoritative |
| Arm armlink | Scatter load region contains execution regions; load and execution addresses can differ; compressed initialization prevents equating execution byte size with stored payload | Separate future grammar; pair genuine full MAP + ELF with independent load/execute inspection. **FIXTURE_BLOCKED**, no parser enabled | **Since ADR-0036:** supported. Execution regions are the region table, `Load base` (from `--load_addr_map_info`) gives load addresses, a region without it that runs outside its load region is Unknown, compressed RW is charged from its `Load base` at its stored size and sized at runtime by the MAP region (A1; ELF only: a lower bound), Compare pairs repeated region names by role (A2), and object rows follow ADR-0033 |
| IAR ILINK | `.icf` placement directives, sections/blocks, initialization table and potentially compressed/shared data | Separate future grammar; pair genuine full MAP + ELF, distinguish initializer ranges from runtime ranges. **FIXTURE_BLOCKED**, no parser enabled |

Primary semantic references: [Arm load/execution region explanation](https://developer.arm.com/community/arm-community-blogs/b/tools-software-ides-blog/posts/building-an-elf-image-for-an-armv8-fixed-virtual-platform-2065554406), [IAR ILINK overview](https://docs.iar.com/ewarm/9.7x/en/iar-c-c---development/linking-using-ilink/linker-overview.html), [IAR MAP option](https://docs.iar.com/ewarm/10.1x/en/iar-c-c---development/linker-options/descriptions-of-linker-options/--map.html). These guide design; they do not replace fixture proof.

Public downstream examples were inspected at [ARMmbed/mbed-os tools/test/memap](https://github.com/ARMmbed/mbed-os/tree/d723bf9e55415433e108124ee6d36337feddf1b8/tools/test/memap), pinned commit `d723bf9e55415433e108124ee6d36337feddf1b8`. They have vendor-style banners but generic substituted paths, abbreviated content and no matching ELF/build recipe proving this cohort. `arm.map` contains compressed load data without a complete RAM execution region; `iar.map` has an incomplete `0x` group total and empty initialization table. These are downstream parser test examples, **not independently validated vendor-generated cohorts**. The project's Apache license was inspected; their exact research copies and hashes remain outside Git, with no redistribution or product-license change. No adequate paired vendor fixture was obtained.

Tests: `crates/firmwaresight-artifact/tests/toolchain_interop.rs` proves metadata, foreign-banner isolation, filename independence, explicit-MAP refusal, ELF-only recovery, UTF-16/invalid bytes, Unicode filenames, long headers, truncation and size guards. `apps/cli/tests/export_privacy.rs` proves actionable refusal on both output modes. Future adapter admission still requires credible samples and any necessary architecture decision; schematic test success cannot promote either vendor row.
