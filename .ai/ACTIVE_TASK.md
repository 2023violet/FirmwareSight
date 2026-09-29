---
title: "Active Task"
doc_id: "FS-AI-005"
product: "FirmwareSight"
version: "0.6.0"
status: "ACTIVE_TASK"
owner: "Engineering"
last_updated: "2026-09-29"
---

# ACTIVE TASK

```text
P1_ANALYZE_DETAILS — the remainder of the Analyze verb, authorized as the first MVP stage under
ADR-0026 (Open-Source MVP-First Delivery). It is a coding task with a fixed acceptance list.
It does NOT authorize P2 Compare, P3 Gate or P4 Release Bundle.
```

```text
STATUS 2026-09-29: IN PROGRESS. Started from be09c65 (Run #12 36520562718, success, 7 of 7 jobs).
Parent prompt: FirmwareSight Open-Source MVP-First Governance Reset + P1 Analyze Completion v1.0.
Acceptance addendum: FirmwareSight P1 Analyze Acceptance Closure v1.0, which binds P1 PASS to the
frozen US-001 criteria, including the bytes/KiB presentation switch.

Governance moved the same day: ADR-0026 re-based G1 on P0 PASS and turned V0 into
NON_BLOCKING_USER_FEEDBACK_TRACK. V0's sample is still honestly 0 / 8 — no participant was invented,
and the withdrawn price-anchor prompt was never executed.
```

Evidence (created as this task completes): `P1_VALIDATION/P1_ANALYZE_EXECUTION_REPORT.md`,
`P1_VALIDATION/P1_ANALYZE_EXIT_CHECKLIST.md`, `P1_VALIDATION/P1_ANALYZE_DESIGN_CHECKLIST.md`,
`P1_VALIDATION/P1_ANALYZE_DETAILS_SMOKE_REPORT.md`. Prior slice: `P1_A0_VALIDATION/`.

## The user outcome

A person who can already load their own ELF (plus optional GNU ld MAP) and read a trustworthy summary
can now also **inspect** what the analysis found:

```text
intake → summary → top contributors → [Sections | Symbols | Evidence]
```

§8 of the parent prompt defines P1's ten capabilities; P1-A0 satisfies 1–4 and error recovery. This
slice completes **5 inspect Sections, 6 inspect Symbols, 7 inspect Evidence, 8 identify largest
contributors, 9 understand evidence quality**.

## Acceptance list that decides P1 PASS

The frozen criteria in `01_PRODUCT/04_USER_STORIES_ACCEPTANCE.md` **US-001 Analyze ELF** are the
acceptance list, not a suggestion. Every item below must be green or P1 stays `IN_PROGRESS`:

- sections visible, symbols visible, memory summary present;
- symbol list sortable **and** filterable, with the search executed in Rust/SQLite rather than by
  shipping every row to React;
- artifact hash shown; unsupported file shows a reason and does not crash when debug info is absent;
- **numeric unit switch between bytes and KiB**: default `Bytes`, `1 KiB = 1024 bytes`, applied
  consistently to artifact size, both footprints, excluded metadata, section file/memory sizes, top
  contributor sizes and symbol sizes; never applied to addresses, file offsets, hashes, counts,
  ordinals or snapshot ids; `Unknown` renders as `Unknown`, never as `0 B` or `0 KiB`;
- `capabilities.debugInfo` row stays visible and renders from the Rust DTO, and its absence must not
  crash Analyze (addendum §6: no second debug-info module);
- switching units is presentation-only: it re-parses nothing, creates no snapshot, writes no SQLite
  row, and changes no CLI or golden output.

## Hard boundaries

- **Bounded payloads.** Default `limit = 100`, maximum `limit <= 500`, both enforced server-side, with
  `rows`, total count and the next offset returned. `04_TECH/14` forbids sending the whole symbol table;
  `desktop_parity.rs` documents that invariant today.
- **Three use-case commands only**: `query_sections`, `query_symbols`, `query_evidence`. No `run_sql`,
  `read_table`, `query_any`, `get_database`. No raw SQL crosses the boundary, no `rusqlite` type crosses
  IPC, no Tauri type enters Core. ts-rs stays the generation boundary and bindings are committed.
- **No schema migration.** `SCHEMA_VERSION` stays 2; sections, symbols and evidence are already stored.
  If a measured need for an index appears, stop and report before writing `0003`.
- **No new dependency and no new design token.** `assets/design-tokens.json` is a frozen asset: a value
  that genuinely has no token stops this round rather than being written as a magic number.
- **Details are bound to the last-good snapshot id.** A failed later attempt must not repoint the
  tables at the failed candidate, and a detail-query error belongs to the details area only: it never
  wipes the summary.
- **No invented data.** Unknown stays Unknown with its reason, addresses stay hexadecimal technical
  presentation, a symbol's storage ordinal is a row position and not identity, and no object or module
  attribution appears without real evidence.
- **Out of scope**: Compare, Gate, Bundle, History, Component Evidence, SBOM, CI integration, Keil/IAR
  adapter, project wizard, treemap, chart library, virtualization library, settings page, pricing,
  user recruitment, cloud, accounts, telemetry.
- UI rules of `AGENTS.md` 11 and `DESIGN.md` apply: dense not crowded, hairline borders rather than
  shadows on tables and panels, mono numerics, status as icon + label and never colour alone,
  keyboard-accessible selector with correct semantics, light theme only.

## State this task runs against

```text
P0                  PASS — frozen at v0.6.0
G1                  PASS — basis P0 PASS, per ADR-0026 (2026-09-29); before that it was V0 PASS + P0 PASS and NOT CLAIMED
Baseline            v0.6.0 — unchanged by this slice; no v0.7.0
Design tokens       v0.2.1
P1-A0               COMPLETE (+ correctness closure COMPLETE)
P1                  IN_PROGRESS — this task
V0                  NON_BLOCKING_USER_FEEDBACK_TRACK — 0 / 8, gates nothing, nothing fabricated
P2 / P3 / P4        NOT STARTED — each still needs its own architect prompt
G2                  NOT REACHED
Pricing/commercial  DEFERRED_POST_MVP
Run #12             36520562718 on be09c65 — success, 7 of 7 jobs (this round's start fact)
Peak RSS            NOT MEASURED      Fuzz: NOT RUN
RustSec             two accepted transitive advisories, unchanged
```

## Next gate after this task

When every item above is green and the validation and real desktop smoke in the parent prompt pass, P1
may be marked `PASS / COMPLETE` and `P2 Compare` becomes the next authorizable stage. **This prompt does
not implement P2.** `AGENTS.md` 1 still forbids inferring the next task from the roadmap.
