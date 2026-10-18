---
title: "Core Section Ranking Correctness"
doc_id: "FS-DEL-025"
product: "FirmwareSight"
version: "0.6.0"
status: "CANDIDATE"
owner: "Engineering"
last_updated: "2026-10-17"
---

# NEXT-CORE-01: section growth ranking correctness

**Authorized by:** the Owner's execution contract in chat on 2026-10-17 ("NEXT-CORE-01"). Development, tests and independent review on `feature/core-section-ranking-correctness`. **A push to `main` needs separate Owner approval.**

**Base:** `main` `1a7f295` (CI run `39099224047`, 10/10), verified live at the start.

**Source:** [24_CORE_SECTION_RANKING_REVIEW.md](24_CORE_SECTION_RANKING_REVIEW.md), option 2.

## 1. What changes

| ID | Change |
|---|---|
| K1 | `SectionRole::from_name` reads the role word a stored section row carries (`"Code"`, `"SymbolTable"`, …). An unrecognized word gives `None` and is never guessed. `SectionRole::admitted_to_budget(alloc)` is memory accounting's existing admission rule, `alloc && !is_device_metadata()`. `memory.rs` calls it instead of restating it, so there is one rule. |
| K2 | `SectionChange::budget_admission()` gives `Budget`, `HostMetadata`, `NotLoaded` or `Undecided`. A row whose two sides disagree, or whose role word is not recognized, is `Undecided`. Both sides are checked, and a side that is absent is not guessed. |
| K3 | `top_section_growth` ranks only `Budget` rows. `largest_added_sections` ranks only `Budget` rows that are not ambiguous, as `largest_added_objects` already does. The size compared is unchanged: the section's size delta (`memory_size`). It is labelled as a section size change, not as Flash. |
| K4 | `section_ranking_exclusions()` counts what the rankings leave out and why: host metadata, not loaded, undecided, and added rows whose name repeats. Every renderer prints that note, so leaving a row out of the ranking never reads as "it did not happen". The full section table, the counts and the change kinds do not change. |
| K5 | The renderers' headings say what the ranking is: CLI text, HTML, Markdown, the release bundle report, and the desktop Compare ranking label. No renderer re-ranks. |

**Not changed:**
- Flash and RAM accounting;
- `Section.memory_size` and its ELF source;
- `analysis:1`, `diff:1` and every JSON schema;
- the manifest format and bundle membership;
- storage and migrations;
- the armlink MAP (still refused, `ERR-MAP-3001`).

**Recorded, not fixed:** for a section without `SHF_ALLOC`, `Section.memory_size` still holds `sh_size`. Fixing that field's meaning needs its own ADR (`24` §4 option 1).

## 2. Acceptance matrix

| AC | Check | How |
|---|---|---|
| AC01 | m2-objects: Flash 132→196 (+64), RAM 12→44 (+32) unchanged; `.text` +64 leads the section ranking; `.bss` +32 is listed; `.symtab` is not | CLI `fwsight diff`; Core test |
| AC02 | `.symtab` +96 and `.strtab` +27 are still in the full section table, and the exclusion note counts them | CLI full output; Markdown/HTML tables |
| AC03 | d4-armlink: Flash 108→192 (+84), RAM 12→44 (+32) unchanged; `ER_IROM1` +84 leads; `.symtab`, `.strtab`, `.debug_frame` and `.comment` are not ranked; the armlink MAP is still `ERR-MAP-3001` | CLI; cross-check script |
| AC04 | Section counts, change kinds and the ambiguous count are unchanged; the unpaired `RW_IRAM1` rows are not in the largest-added list | CLI counts before and after; Core test |
| AC05 | Core tests for: non-alloc metadata, non-alloc `Other`, alloc `.bss`, alloc with role `Unknown`, an unrecognized role word, an unknown size, admission that differs between sides, ambiguous additions. They fail on the old code and pass on the new | `cargo test`, run before and after |
| AC06 | Desktop Compare, CLI text, HTML, Markdown and the bundle report agree; none recommends host metadata | Renderer tests; native smoke |
| AC07 | Goldens regenerated with `update_goldens.py --confirm`; every changed line explained | Golden diff review |
| AC08 | `check.py` passes every step; QA-identifier native smoke recorded with an AC matrix | Gate, smoke |
| AC09 | Independent review: no S1 or S2; S3 items dispositioned; the final SHA is reviewed | Review |
| AC10 | The Owner's checkout (`e34f9e6`, 9 staged files), the 23 Owner app-data files, the QA stores and the other branches are unchanged | Read-only checks |

## 3. Results

Evidence: [evidence/NEXT-CORE-01](../evidence/NEXT-CORE-01/README.md).

- **Core:**
  - Five ranking tests and a role read-back test were added.
  - The first version of the ranking tests failed on the unchanged code.
  - The final tests fail when the old ranking is restored by mutation, and pass on the fix. Core: 170 passed.
- **Numbers, before and after:**

  | Pair | Flash | RAM | Ranking before | Ranking after | Left out |
  |---|---|---|---|---|---|
  | m2-objects | 132→196 (+64), unchanged | 12→44 (+32), unchanged | `.symtab` +96, `.text` +64, `.bss` +32, `.strtab` +27 | `.text` +64, `.bss` +32 | 2 host metadata |
  | d4-armlink | 108→192 (+84), unchanged | 12→44 (+32), unchanged | `ER_IROM1` +84, `.symtab` +48, `.strtab` +24, `.debug_frame` +16, `.comment` +4 | `ER_IROM1` +84 | 4 host metadata; 2 unpaired additions |

  The section counts and ambiguity are unchanged (m2-objects ~6; d4-armlink +2 −2 ~7, 4 unpaired). The armlink MAP is still refused with `ERR-MAP-3001`.
- **One consumer the contract did not name:** the desktop's "largest additions" for sections was not Core's list. The page sorted the change table itself (`changeKind: 'added'`, `sort: 'memorySize'`), so it would have recommended host metadata and unpaired names.
  - `CompareSummaryDto` now carries Core's `addedSections` and `sectionRankingExclusions`, and the page shows those.
  - Symbol additions are unchanged.
  - The summary test pins the new bounded shape.
- **Goldens:** only `golden/reports/p4-release` changed.
  - The release report's section ranking used to list `.debug_loclists` +221, `.debug_info` +112, `.debug_line` +107 and `.symtab` +80 around `.text` +88. It now lists `.text` +88 and `.bss` +4, with a heading and a one-line note.
  - The only other changes are that file's digest in `SHA256SUMS` and two digests and a size in the manifest.
  - The release id, the file set and every JSON golden are unchanged.
- **Gates:** `check.py` 20/20 (1614 Rust tests, 535 UI tests).
- **Native smoke:** 12 PASS. The first armlink run recorded one FAIL caused by the driver's own Markdown-escaping check; it is kept.
- `evidence/D4/CROSSCHECK.md` was not regenerated. It is `24`'s evidence of the defect, from before the fix.

## 4. Review

Pending on the candidate commit.

