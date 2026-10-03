---
title: "P5 Migration Decision"
doc_id: "FS-P5-MIGRATION"
product: "FirmwareSight"
version: "1.0"
status: "DECIDED_PENDING_IMPLEMENTATION"
stage: "P5_PRODUCTIZATION"
owner: "Engineering"
last_updated: "2026-10-03"
---

# P5 — migration decision (prompt §18)

Prompt §18 says: *"Do NOT create migration 0005 merely to make History easier"*, and if a migration is
still needed, write this document first. This document comes first: no `0005_*.sql` file exists in the
repository yet, and none will be created before this decision is read alongside the code it cites.

## 1. The question, stated honestly

Two separate needs are in play and only one of them can justify a schema change.

**History needs no new schema.** Everything a History page displays is already persisted and already
reachable by a `SELECT`: `builds` (`project_id`, `snapshot_id`, `state`, `created_at`,
`created_by_fwsight`), `artifacts` (`kind`, `sha256`, `byte_size`, architecture/bitness/endianness),
`gate_runs` (`overall_effective_severity`, `policy_sha256`, `created_at`), `accepted_reviews` and
`release_records` (`release_version`, `manifest_sha256`). What is missing is *read APIs* — storage exposes
only `gate_run_by_id` (`crates/firmwaresight-storage/src/gate.rs:292`) and `release_record_by_id`
(`crates/firmwaresight-storage/src/release.rs:132`), while `query.rs:214/270/326` page within one build.
Adding three listing queries is a code change, not a migration. Writing a migration for History would fail
§18, so this decision does not do that.

**The Unknown-reason gap needs no new feature at all — it is a data-loss defect.** Two columns in the
schema throw away information the engine already had in memory:

| column | definition | reason column | write site |
| --- | --- | --- | --- |
| `sections.file_offset` | `0001_initial.sql:65` | **absent** | `db.rs:408` via `optional_fact_u64` |
| `symbols.address` | `0001_initial.sql:79` | **absent** | `db.rs:451` via `optional_fact_u64` |

`optional_fact_u64` (`crates/firmwaresight-storage/src/db.rs:567-569`) is three lines long and discards
the reason on purpose-free grounds:

```rust
fn optional_fact_u64(fact: &Fact<u64>) -> Option<i64> {
    fact.value().copied().map(|value| value as i64)
}
```

Its two siblings keep it: `optional_string` (`:571-575`) returns `(value, reason)` and the numeric
`optional_fact` returns the pair used for `virt_addr`, `load_addr`, `mem_size` and `entry_point`. So of
the seven nullable **numeric** columns in the schema, five carry a companion `*_unknown` text column and
two do not; and of the nullable **text** columns, every one does (`name_unknown :56/:78`,
`region_unknown :70`, `build_id_unknown :46`). The asymmetry is not a design — it is the original table
shape (`0001_initial.sql`) never being completed, and the write path followed the schema rather than the
other way round.

The consequence is downstream and visible. The read model cannot hand back what was never stored, so it
downgrades the type: `SectionRow::file_offset` is `Option<u64>` with the comment *"The schema has no
`file_offset_unknown` column, so an absent offset can only be reported as `None`"*
(`query.rs:99-101`), and `SymbolRow::address` likewise (`query.rs:113-114`) — while their neighbours are
`Fact<u64>` (`query.rs:97-98`, `:115`). In the window, every other Unknown explains itself and these two
do not: `apps/desktop/ui/src/Details.tsx:479` renders `{row.fileOffset ?? <Unknown />}` and `:550`
renders `{row.address ?? <Unknown />}`, bare, the same cell, whereas `:474`, `:477` and `:483` pass
`reason={row.virtualAddressUnknownReason}` / `loadAddressUnknownReason` / `memorySizeUnknownReason`.

That is L6 and L7 in `G2_VALIDATION/G2_KNOWN_LIMITATIONS.md:28-29`, and it is a `PRESERVE UNKNOWN`
violation (§17 discipline, `AGENTS.md` 8: a fact that is Unknown must say why): the reason for a numeric
Unknown does not exist in `builds`, not in `artifacts`, not in `evidence`, and not in any portable
document. No read-side code can recover it, because the loss happens on the way in.

### 1a. What the evidence does and does not show

The two halves of that finding are not the same size, and saying so now is cheaper than being caught
saying it loosely at close.

**L6 is observed end to end.** `crates/firmwaresight-artifact/src/elf.rs:240-244` really does emit
`Fact::unknown("the section has no file range; it occupies no bytes on disk")` for a `SHT_NOBITS`
section — the `.bss` case G2 recorded — so the reason exists in memory, reaches `db.rs:408`, and is
dropped. The regression in §3.1 can be written against a committed fixture and it will be red for the
right reason.

**L7 has no producer today.** `elf.rs:264` is unconditional: `address: Fact::known(symbol.address())`.
Nothing in the artifact crate, the MAP adapter or Core ever hands storage an Unknown symbol address, so
no user-visible symptom of the missing column has been reproduced, and G2's L7 row — which inherits its
"same as L6" impact wording — describes a schema asymmetry rather than an observed window state. The
column is still added, for two reasons that do not depend on inventing a symptom: the schema's own rule
(`0001_initial.sql:42`'s comment, *"A fact we could not determine is recorded as a reason, never silently
as zero"*) applies to every nullable column, and a producer that later reports an Unknown address — an
ELF without a symbol-value interpretation, which is exactly the kind of thing the §26 fixture expansion
will put in front of the parser — must not be forced to write a bare NULL. What changes is the claim: the
L7 regression is written against a **synthetic** snapshot that constructs
`Symbol { address: Fact::unknown(…) }` directly, and the close-out wording must say the column was
missing and unreachable, not that the window was showing an unexplained Unknown.

## 2. Decision

**Add migration `0005_unknown_reasons`, additive only, and move `SCHEMA_VERSION` 4 → 5.**

```sql
-- migrations/0005_unknown_reasons.sql
ALTER TABLE sections ADD COLUMN file_offset_unknown TEXT;
ALTER TABLE symbols  ADD COLUMN address_unknown     TEXT;
CREATE INDEX idx_builds_created ON builds(created_at);
```

SQLite `ADD COLUMN` with a nullable, constraint-free type rewrites no rows and takes no table rebuild, so
this matches the pattern of the three migrations that came before it (`0002` rebuilds only because it
re-keys; `0003` and `0004` create tables). `migrate()` applies each migration in its own transaction
(`db.rs:132-141`), which is what makes `gate_history.rs:314 a_failed_gate_migration_leaves_the_previous_schema_intact_and_recoverable`
true today and will hold for `0005` unchanged.

The index rides along and is **not** the justification. `builds.created_at` has no index
(`0001_initial.sql:28-29` covers `project_id` and `snapshot_id` only) while every other history table that
sorts by time has one (`idx_gate_runs_created`, `idx_release_records_build (build_id, created_at, id)`). A
History page can be written without the index; the index is included because it closes a schema
inconsistency in the same additive slot, and it is called out separately in §5 so a reviewer can strike it
without disturbing the reason columns.

## 3. What the code change is, in order

1. **A failing regression first.** One test asserting that a build whose `.bss` section has an unknown
   file offset persists *the reason*, not just the absence. It must go red against schema 4 — that red run
   is the proof the defect is real, and it is the same discipline that closed E2E-F001/F002/F003.
2. `0005_unknown_reasons.sql` + `MIGRATIONS` entry (`db.rs:23-27`) + `SCHEMA_VERSION: 5` (`db.rs:14`).
3. **Fix the writer, do not route around it.** Replace `optional_fact_u64` with the paired form the
   siblings already use, and pass both halves at `db.rs:408` and `db.rs:451`. Leaving the lossy helper in
   place while adding columns nobody writes to would be the worst available outcome: schema honesty with
   the same data loss.
4. **Lift the read model.** `SectionRow::file_offset` and `SymbolRow::address` become `Fact<u64>` like
   their neighbours, read through the existing `bytes_or_unknown()` (`query.rs:493`) instead of the raw
   `row.get(...).map(...)` at `query.rs:450` and `:461`.
5. **Regenerate the IPC bindings.** `bridge.ts`, the ts-rs output and the drift gate
   (`scripts/check.py:199-205`) make this step impossible to forget: `cargo test -p firmwaresight-desktop`
   followed by `git diff --exit-code -- apps/desktop/ui/src/ipc/generated` is the check.
6. **Show it.** `Details.tsx:479` and `:550` gain the `reason={…}` prop the adjacent cells already pass,
   and the L6/L7 note in the Evidence Inspector disappears because there is nothing left to note.

No Gate rule, no diff semantic, no memory-accounting formula and no portable document format changes:
`fwsight`/desktop parity is about rendered documents, and the two fields become *more* informative in the
desktop, not differently-computed. Core keeps its headless, dependency-free position (`AGENTS.md` 3).

## 4. The rule that keeps this honest: no invented history

Rows written by schema ≤ 4 have `NULL` in the new columns, and `NULL` there means **"the version that
wrote this row did not record a reason"** — a third state, distinct from "the reason is empty" and from
"the value is known". Two things follow, and both are implementation requirements, not suggestions:

- the read model must not map a legacy `NULL` reason to a fabricated string, and
- the UI must not show a bare `Unknown` for those rows either. It shows the engine's fixed wording for the
  unrecorded case, so an old build's `.bss` row explains that its reason was lost by an older program
  rather than pretending the absence is informative.

Backfill is therefore **out of scope and refused**: re-parsing committed fixtures could restore reasons for
fixture builds, but the product's promise is about the user's own store, and a re-parse that disagrees
with what was originally observed would replace one lost fact with an invented one.

## 5. Migration and recovery matrix (§21), before any code runs

`04_TECH/15_STORAGE_DATABASE_BASELINE.md` §8 fixes four obligations: a numbered migration, a test from the
oldest supported schema, a backup/recovery plan before a destructive operation, and no silent reset. Mapped:

| path | expectation | test to hold it |
| --- | --- | --- |
| v1 → 5 | every row survives, four intermediate steps applied in order | extends `storage.rs:476 a_version_one_database_is_upgraded_without_losing_its_evidence` |
| v2 → 5, v3 → 5, v4 → 5 | same, with the stage rows of each schema intact | one parameterized upgrade test per start version |
| v4 rows and their `NULL` reasons | preserved exactly, and read back as *reason not recorded* | new test, §4 above |
| a failed `0005` | previous schema and every row untouched, recoverable | mirrors `gate_history.rs:314` |
| v5 opened by a schema-4 binary | refused, never reset | `db.rs:120-127` and `storage.rs:303 an_unknown_newer_schema_is_refused_rather_than_reset` |
| a build analyzed after the upgrade | `file_offset_unknown` / `address_unknown` populated from the parser | the §3.1 regression, now green |

**Destructive?** No. Additive columns and one index change no existing row and drop nothing, so there is
no destructive step to plan around. What *is* irreversible in the ordinary sense is the version move: a
schema-5 file cannot be opened by a schema-4 binary (it refuses, which is the correct behaviour, not a
revert path). The recovery plan for that is therefore a real one, and §22 has to supply it: a
hash-verified copy of the store taken **before** a user upgrades FirmwareSight, which today does not exist
anywhere in the codebase (`grep -rn "backup" crates/firmwaresight-storage/src` hits only the wording of an
error message at `error.rs:61` that already tells the user to "restore a backup"). So the sequencing rule
for P5 is explicit: **`0005` may not ship in a commit before the backup capability ships with it.**
Diagnoses and the copy use the same storage-owned code path (§46), not an ad-hoc file read.

## 6. Alternatives considered and rejected

- **Store the reason in the generic `evidence` table.** Rejected: `evidence` is keyed
  `(build_id, field)` with a classification and a source locator (`0001_initial.sql:88-98`), and the
  section/symbol read paths do not consult it at all. It would invent a per-column convention inside a
  table built for a different question, and every reader would need a join to render a cell.
- **Recompute the reason on read** from other columns. Rejected: impossible. The reason is a parser
  judgement about inputs it did not receive (a MAP that never named the region, a symbol table with no
  address), not a function of the bytes that are stored. Deriving one would be fabricating an Observed
  fact, which `AGENTS.md` 8 forbids.
- **Leave schema 4 and show a fixed generic explanation.** Rejected: that is today's behaviour and it is
  the finding. It also hard-codes the defect, since the reason the user most needs is the specific one.
- **A migration purely for History.** Rejected by §18 and unnecessary: §1 shows History needs queries.

## 7. What this closes

`G2_KNOWN_LIMITATIONS.md:28-29` — L6 (`sections.file_offset` reason) and L7 (`symbols.address` reason) —
close on the terms those rows named: *"P5 (needs a migration)"*. The disposition recorded at close must
state the legacy-`NULL` limit from §4, because for stores written before this round the reason genuinely
is not recoverable, and an honest "not recorded by the version that wrote it" is the whole of what can be
claimed.
