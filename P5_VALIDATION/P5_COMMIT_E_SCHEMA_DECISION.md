---
title: "P5 Commit E schema decision request"
doc_id: "FS-P5-COMMIT-E-SCHEMA"
product: "FirmwareSight"
version: "1.0"
status: "STOPPED_FOR_ARCHITECT"
stage: "P5_PRODUCTIZATION"
owner: "Engineering"
last_updated: "2026-10-04"
---

# P5 Commit E — public-schema stop (prompt §42, work item §32 / limitation L15)

Prompt §42 says that if a fixture exposes a fix requiring a new portable field, a renamed serialized
enum, a schema version or a release-id semantic change, Commit E must STOP that subproblem, write this
document, and return it to the Architect. L15 does exactly that, so it is stopped here and nothing in
Commit E changed it. Unrelated authorized work continued, which §42 also permits.

This document decides nothing. It prices the options and names the surfaces each one moves.

## 1. The problem

`MemoryEvidenceBasis::ElfAddressAndFlags` is charged with a source label that names the wrong table.
`crates/firmwaresight-artifact/src/pipeline.rs:284-289` returns this tuple:

```rust
MemoryEvidenceBasis::ElfAddressAndFlags => (
    "elf-address-and-flags",              // rule
    SourceType::ElfProgramHeader,         // source type  <-- names program headers
    format!("{section_locator} + elf:sh_flags"),  // locator <-- names a section header field
    EvidenceClass::Observed,
),
```

The rule string, the locator string and the actual read all say *section header flags* (`sh_flags`);
only the `SourceType` says program headers. The same file is explicit that program headers are not read
at all for this purpose — `crates/firmwaresight-artifact/src/elf.rs:253` sets
`load_address: Fact::unknown("not present in the section header table")`, and the section scan in
`elf.rs` walks `file.sections()`, never `file.segments()`. So the label is not a simplification of a
program-header claim; it contradicts an `Unknown` the same adapter produces a few lines earlier.

P0's audit recorded this as L15 ("`ElfProgramHeader` source label") on the assumption that it was a
label to fix. §32 asks first whether it is (A) user-facing wording only or (B) serialized public
portable evidence semantics. **Measured answer: B**, in three namespaces at once, which is why §42
applies.

## 2. What the fixture evidence added

Before Commit E this label was visible only in a code reading. Two things the compatibility fixtures
produced make it an evidence problem rather than a style problem:

1. `fixtures/elf/p5-compat/clang-arm/firmware.elf` put a section on the `ElfAddressAndFlags` path in
   real bytes (`.ARM.exidx.text.main`, 8 bytes, `SHF_ALLOC` set, unrecognized name — see
   `P5_VALIDATION/P5_COMPATIBILITY_FIXTURE_REPORT.md`), so the mislabeled basis is now exercised by a
   committed binary rather than only by a unit fixture built in Rust.
2. Tracing why those 8 bytes fell out of both budgets led to the tuple above; reading it is what
   showed the rule, the locator and the source type disagreeing with each other. The commit that fixed
   the undercount deliberately did not touch the label, because §42 forbids it.

No fixture forced a *schema* change. The stop is about the label itself.

## 3. Where the string lives, measured

| namespace | value as stored | site | who reads it |
| --- | --- | --- | --- |
| portable analysis JSON | `"elf.program-header"` | `crates/firmwaresight-report/src/dto.rs:139`, enum member at `schemas/analysis.schema.json:616` | CLI `--json`, anything validating against `analysis:1` |
| local store | `ElfProgramHeader` (the Rust `Debug` name, not the wire name) | `crates/firmwaresight-storage/src/db.rs:507`, column `evidence.source_type` | History and Diagnostics read-back (`query.rs:503`) |
| shipped bundle document | `"elf.program-header"` inside `analysis.json` (`ANALYSIS_DOC_NAME`, `crates/firmwaresight-core/src/domain/release.rs:431`) | `crates/firmwaresight-project/src/bundle.rs:802` | bundle verification and read-back |
| desktop UI | passed through verbatim | `apps/desktop/ui/src/Details.tsx:667` renders `row.sourceType` in a `<dd>` | a stranger engineer reading Diagnostics |
| goldens | 4 rows | `golden/cli/p0-basic-analyze.json:100,110,120,130` (the only golden file carrying it: `grep -rc` over `golden/`) | `drift/goldens` step of `scripts/check.py` |

So one rename touches a public schema enum, already-written rows in the owner's store, the content of a
Release Bundle, and the goldens — and the store and the wire format do not even use the same spelling,
so there is no single token to replace.

## 4. The old contract, stated plainly

`analysis:1` declares `sourceType` as a closed enum that includes `"elf.program-header"`
(`schemas/analysis.schema.json:613-620`). A consumer that has seen an evidence item with
`basis = "elf-address-and-flags"` and `sourceType = "elf.program-header"` is entitled to read that as
"this charge consulted the ELF program header table". The product never consults it. A consumer acting
on that reading would be wrong, and the wrongness is in the contract, not in the prose around it.

## 5. Options, with what each one moves

Presented as choices for the Architect; §42 forbids Commit E from picking one.

**A. Display-only mapping (wording).** Keep `SourceType::ElfProgramHeader` and the wire string
untouched; map it to a caption at the presentation edge, the way Commit E's L20 work mapped basis names
to captions in `apps/desktop/ui/src/Compare.tsx`. Cost: small, no schema change, no migration, no
golden movement. What it does *not* fix: the CLI JSON, the stored `evidence.source_type` and the
bundle's `analysis.json` keep saying program header, so a non-UI consumer is still misled. It also adds
a fourth spelling of the same fact to the UI's caption table, which is exactly the class of drift L19
and L20 were opened for.

**B. Additive schema change.** Introduce e.g. `ElfSectionFlags` (wire `"elf.section-flags"`), emit it
for `ElfAddressAndFlags`, and deprecate the old member while leaving it in the enum. Cost: the enum
grows, so `analysis:1` content changes; goldens move (4 rows); new stores and old stores hold different
spellings for the same basis, and History has to render both; a bundle built by the new code is not
reproducible from the old code. Question for the Architect: whether an additive enum member is a minor
or major revision of `analysis:1`, and whether `sourceType` participates in any digest that the Gate run
id or release identity covers.

**C. Corrective rename.** Replace the member with an accurate one. Cost: breaking for `analysis:1`
consumers, requires a schema version bump, and existing `evidence.source_type` rows in the owner's live
store keep `ElfProgramHeader` forever unless a migration rewrites them — which is a *semantic* rewrite
of persisted evidence, and `AGENTS.md` §6/§9 put destructive or meaning-changing schema work behind
human confirmation. Goldens move; bundle read-back of any previously published bundle would fail to
reproduce.

**D. Leave it, documented.** Keep the label and record in `04_TECH` that `elf.program-header` names the
ELF-table family loosely rather than the literal table. Cost: zero code. Risk: the wrongness becomes the
documented contract, and L15 carries forward a third time.

## 6. Compatibility impact

Nothing here affects the *five-state Gate* or the numbers the memory model produces: the charge is
correct after the alloc fix, and only its source label is wrong. The compatibility weight is entirely in
how a downstream consumer interprets `sourceType`, and in whether stored rows and portable JSON must
agree. `urn:firmwaresight:schema:diff:1` and the Gate/accepted-review/release-manifest docs are not
touched by any of the four options; `analysis:1` is touched by B and C.

## 7. Release Bundle impact

Options B and C change the bytes of the shipped `analysis.json`, so:
- a bundle built after the change cannot be reproduced byte-for-byte by code before it (relevant to the
  bundle verification path at `crates/firmwaresight-project/src/bundle.rs:1607`);
- any Release Notes digest / Gate run id is unaffected — those hash Release Notes and policy, not the
  analysis document — but this should be confirmed by the Architect against ADR-0028 rather than assumed
  from this document, since release identity is exactly the semantics §42 refuses to change casually.

## 8. Golden impact

`golden/cli/p0-basic-analyze.json` carries the string in 4 rows and is the only golden that does.
`scripts/update_goldens.py` regenerates it; `drift/goldens` in `scripts/check.py` fails on unrecorded
movement. Option A moves no golden. B and C move that file (and its SHA-256 entry if the goldens
themselves are hashed in `golden/reports/*/SHA256SUMS`, which is a governance-file question Commit F
already holds through L26).

## 9. What Commit E did instead

- Fixed the real defect the fixture exposed (an allocated section entering neither budget) without
  touching the label: `crates/firmwaresight-artifact/src/elf.rs`, `map_section_kind(.., is_alloc)`.
- Kept L15 open with this document as its disposition record.
- Made the *user-facing basis wording* honest for Compare (L20) and the attribution wording for Analyze
  and Compare (L19), neither of which passes through `SourceType`.
- Recorded L15 as `CARRIED_FORWARD → ARCHITECT` in `P5_SUPPORTABILITY_REPORT.md`.

## 10. What the Architect needs to decide

1. Which of A/B/C/D above, or a fifth shape.
2. If B or C: whether `analysis:1`'s `sourceType` enum is versioned minor or major, and whether
   `sourceType` participates in any identity digest (per ADR-0028, identity is the exact bytes observed,
   so this must be checked, not argued).
3. If B or C: what History does with rows already stored under the old spelling, and whether that needs
   a migration (migration 0006 would be new schema work and needs its own written decision, per the
   rule that established 0005).
4. Whether the loose-vs-literal naming of ELF tables deserves a line in `04_TECH` regardless of the
   choice, since three crates spell the same idea differently.

Commit E does not answer these and does not implement any of them. **STOP** on this subproblem; the
rest of Commit E proceeded, and this row returns to the Architect with the fixture evidence attached.
