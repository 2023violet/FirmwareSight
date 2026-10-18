---
title: "Core Section Ranking Review"
doc_id: "FS-DEL-024"
product: "FirmwareSight"
version: "0.6.0"
status: "REVIEW_VERDICT; OPTION_2_IMPLEMENTED"
owner: "Engineering"
last_updated: "2026-10-17"
---

# Core review: `.symtab` at the top of the section growth ranking

**Follow-up:** option 2 was implemented in NEXT-CORE-01 ([25](25_CORE_SECTION_RANKING_FIX.md)). Option 1 (the meaning of `memory_size` for unloaded sections) is still open. The outputs quoted below, and `evidence/D4/CROSSCHECK.md`, are from before the fix and are kept as the evidence of the defect.

A read-only review requested in D2C-R §4, after R7 recorded that `.symtab` leads the Compare section ranking on `m2-objects` (`06_DELIVERY/21` §4). Nothing in the code was changed by this review.

## 1. Verdict

**A real Core defect in the ranking's input. It makes the ranking misleading; the budget figures are not affected.**

- The Flash and RAM footprints are correct. They exclude non-loaded sections by role.
- `Section.memory_size` is documented as "bytes occupied at runtime". For a section without `SHF_ALLOC`, the ELF reader fills it with `sh_size` instead, which is not a runtime size. That breaks the field's own contract.
- `DiffResult::top_section_growth` ranks by the `memory_size` delta of every section, loaded or not. Growth of host metadata (`.symtab`, `.strtab`, `.debug_*`, `.comment`) is therefore listed and ranked beside growth that costs Flash or RAM, under one heading.
- The desktop shows Core's first entry as the first ranking pick. The comparison's first pointer can therefore lead to a section that costs the device nothing.

## 2. Evidence (Observed)

### 2.1 Code

| Location | What it does |
|---|---|
| `crates/firmwaresight-core/src/domain/section.rs` | `memory_size: Fact<u64>`, documented "Bytes occupied at runtime"; `file_size` "Bytes occupied in the file". `SectionRole::is_device_metadata()` names Debug, SymbolTable, StringTable, Note and ArmAttribute as "not part of the device budget". |
| `crates/firmwaresight-artifact/src/elf.rs` (section loop) | `memory_size = section.size()`, that is `sh_size`, for **every** section, whether `SHF_ALLOC` is set or not. |
| `crates/firmwaresight-core/src/domain/memory.rs` `role_charge` | Nonvolatile and runtime charges are 0 unless the role contributes by default, so `.symtab` and `.strtab` are charged 0. **The budgets are right.** |
| `crates/firmwaresight-core/src/domain/diff.rs` `top_section_growth` | Ranks `section_changes` by `memory_size.delta` (> 0), with no role or `alloc` filter. `largest_added_sections` does the same for additions. |
| Consumers | Desktop Compare (`src-tauri/src/compare.rs`, `topSections`; the UI takes `topSections[0]` as the lead ranking pick and does not re-rank, D-R3-1); the detailed HTML report and the CLI text (`diff_render.rs`); the Markdown report (`markdown.rs`); the release bundle report (`project/src/bundle.rs`). |

### 2.2 Output

`fwsight diff` on `fixtures/elf/m2-objects` (GCC + GNU ld), base → target:

```text
Nonvolatile / load image 132 bytes → 196 bytes delta +64 [exact]; runtime RAM 12 bytes → 44 bytes delta +32 [exact].

Top section growth (Changed rows)
        +96  .symtab                      528 bytes  [Changed]
        +64  .text                        192 bytes  [Changed]
        +32  .bss                         40 bytes  [Changed]
        +27  .strtab                      208 bytes  [Changed]
```

The whole Flash growth (+64) is `.text`, and the whole RAM growth (+32) is `.bss`. The first row, +96, costs the device nothing.

The same happens on the D4 armlink pair (`fixtures/elf/d4-armlink`, Arm Compiler 6.24; full output in [evidence/D4/CROSSCHECK.md](../evidence/D4/CROSSCHECK.md)). There, `.symtab` +48, `.strtab` +24, `.debug_frame` +16 and `.comment` +4 rank right below `ER_IROM1` +84. So this is not a GNU ld quirk.

## 3. Why it is a defect, not intended behaviour or wording alone

- **The field contract is broken at its source.** A non-loaded section has no runtime size. Writing `sh_size` into a field documented as runtime bytes is wrong data, whatever the caption says.
- **The heading claims something the data does not support.** "Top section growth" sits under the memory answer, in the same report. A reader takes it as an explanation of that answer.
- **Renaming in the UI alone is not allowed and would not be enough.** AGENTS.md §3 forbids the UI from re-ranking. Four exports print the same Core list.

## 4. Recommended fix (not made here; needs its own task and review)

1. Keep `memory_size` as a Fact, but make it honest. For a non-`SHF_ALLOC` section it should be `Known(0)` or `Unknown("not loaded at runtime")`, with `sh_size` still available as the section's declared size. This changes the meaning of a field in `analysis:1` (`sections[].memorySize`), and the other `memory_size` readers (`memory.rs` runtime charges) must be checked with it. Per AGENTS.md §2, persisted or exported schema semantics move only with an ADR (ADR-0022 governs the schemas).
2. **Or, smaller and schema-neutral:** `top_section_growth` and `largest_added_sections` rank only budget sections, using the predicate memory accounting already uses (`flags.alloc && !role.is_device_metadata()`), so a non-alloc section whose role is `Other` (such as `.comment`) cannot leak in. Metadata growth goes to a separate, plainly labelled list ("Host metadata, not in any budget"), so nothing disappears. This touches only the ranking functions and their four renderers, plus goldens.
3. Either way, add a Core test that ranks `.symtab` growth larger than `.text` growth and asserts that `.text` leads the budget ranking. Regenerate `p2-diff.html`, the `p4-release` report and its digests on purpose.

Option 2 is the recommended first step: it is schema-neutral and fixes what the reader sees. Option 1 is the cleaner long-term model, and it belongs in an ADR.

## 5. Related observation

On the armlink pair, `armlink` names output sections after execution regions, and both the RW data and the ZI part of `RW_IRAM1` are sections named `RW_IRAM1`. Compare pairs sections by name, so these rows count as "rows whose name repeats". They show as two additions and two removals instead of two changes. This is a D4 adapter question (pair by name plus type, or by region), recorded in [evidence/D4](../evidence/D4/README.md). It does not change the verdict above.
