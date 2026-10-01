---
title: "SQLite Storage Baseline"
doc_id: "FS-TECH-016"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-09-30"
---

# SQLite Storage Baseline

## 1. Decision

SQLite is the local structured source of truth.
Binding: `rusqlite + bundled`.

JSON is not the project database.

## 2. Why

FirmwareSight needs:
- build history；
- artifact indexes；
- 100k+ symbols；
- evidence relationships；
- query/filter/sort；
- gate history；
- schema migration。

These are relational/indexed workloads.

## 3. Initial logical tables

```text
projects
builds
artifacts
sections
symbols
evidence
gate_runs
gate_findings
release_records
schema_migrations
```

Component/SBOM tables are added only in their phase.

## 4. Key relationships

Project 1─N Build  
Build 1─N Artifact  
Build 1─N Section  
Build 1─N Symbol  
Build 1─N Evidence  
Build 1─N GateRun

## 5. SQLite configuration

At connection initialization:
- `foreign_keys = ON`
- WAL mode where supported
- busy timeout
- transactional bulk import
- explicit migration version

Use NORMAL synchronous only after durability tradeoff is documented and tested; release bundle export itself remains filesystem-verified.

## 6. Import transaction

```text
begin
  insert build (state=IMPORTING)
  insert artifacts
  insert sections
  batch insert symbols
  insert evidence
  mark build COMPLETE
commit
```

If transaction fails:
- build not visible as COMPLETE；
- no half-import state presented as valid。

## 7. Raw files

Default:
- database stores path metadata/hash/facts；
- raw ELF/BIN remains at source path；
- release export may copy selected artifacts explicitly。

## 8. Migrations

Every schema change:
- numbered migration；
- test from oldest supported schema；
- backup/recovery plan before destructive operation；
- no silent reset database。

## 9. Portability

Portable outputs:
- versioned JSON；
- HTML report；
- Release Bundle。

SQLite file is an implementation detail, not long-term interchange format.

## P3 gate history schema (ADR-0023, ADR-0027, 2026-09-30)

`SCHEMA_VERSION` moves 2 → 3 with one additive migration, `migrations/0003_gate_history.sql`.
No earlier table is touched: a v1 or v2 database keeps every row it holds and gains four tables.
`release_records` from §3 stayed unwritten at P3 — a Release Bundle is P4 scope, and P4 writes it in
`0004_release_records.sql` below.

Tables, in the shape §3 anticipated:

| table | key | what it fixes |
| --- | --- | --- |
| `gate_runs` | `id` = `gate-<sha256>` | one immutable verdict per evaluation, tied to `builds(id)`, with an optional `baseline_build_id` that may never equal its own `build_id` |
| `gate_findings` | `(run_id, id)`, `UNIQUE (run_id, rule_id)`, `UNIQUE (run_id, ordinal)` | exactly one answer per rule, recorded in canonical rule order |
| `gate_finding_evidence` | `(run_id, finding_id, ordinal)` | the locators one rule cited, in order |
| `accepted_reviews` | `(run_id, finding_id)` | the owner's decision about one REVIEW, with actor, reason and SQLite's UTC insertion time |

Invariants the schema enforces rather than trusts:

- `state` / `effective_severity` pairing is a CHECK, so an `UNKNOWN` can never be stored as a PASS and
  an `N/A` carries the neutral severity;
- a run id is `gate-` plus 64 hex characters, never a wall clock or a uuid;
- an evidence locator may not contain `\`, so no Windows host path can be stored at all;
- `gate_runs` and `gate_findings` refuse UPDATE, and `accepted_reviews` refuses UPDATE and DELETE.

Typed APIs (§54 of the P3 prompt), none of them exposing rusqlite: `persist_gate_run` writes the run,
its findings and every ref in one transaction — a rejected row leaves no half-run — and reports
`AlreadyStored` for a repeat of the same content, while the same id over different semantics is
`StorageError::Invariant`, never an update. `gate_run_by_id` reads findings and refs back in stored
order. `accept_review` adds an audit row and touches no finding; `accepted_reviews_for_run` returns the
trail oldest-first.

New codes: `ERR-STORAGE-4007` unknown finding, `ERR-STORAGE-4008` not a review, `ERR-STORAGE-4009`
already accepted, `ERR-STORAGE-4010` missing actor or reason. `ERR-STORAGE-4006` still covers a write
failure and `ERR-INTERNAL-9002` the invariant.

Covered by `crates/firmwaresight-storage/tests/gate_history.rs` (22 tests): fresh → v3, v1 → v3 and
v2 → v3 with rows preserved, an obstructed 0003 that rolls back and is then recoverable, atomic insert,
dedupe, collision mismatch, immutability attempted by raw SQL, and every acceptance refusal.

## P4 release record schema (2026-09-30)

`SCHEMA_VERSION` moves 3 → 4 with one additive migration, `migrations/0004_release_records.sql`. No
earlier table is touched: a v1, v2 or v3 database keeps every row it holds and gains one table.

`release_records` is the **audit index of a published Release Bundle, not the bundle**. §9's portability
rule is what the shape enforces: deleting or moving the project database cannot invalidate a shipped
directory, and moving a directory cannot require this row. So the table stores no bundle destination, no
project root, no artifact source path and no manifest or HTML blob — one row is seven facts:

| column | what it fixes |
| --- | --- |
| `id` | `release-<sha256>`, the digest of the canonical release input; never a wall clock, a uuid or a counter |
| `build_id` | `builds(id)` — the build that ships |
| `baseline_build_id` | `builds(id)`, nullable, and never equal to `build_id` |
| `gate_run_id` | `gate_runs(id)` — the run whose `PASS` disposition qualified the release |
| `release_version` | the project release version the Gate judged |
| `manifest_sha256` | SHA-256 of `release-manifest.json` as written: the fact that ties this row to a directory without naming one |
| `created_at` | SQLite's UTC insertion time — an audit time, deliberately **not** part of the release id and not part of any portable bundle byte |

Invariants the schema enforces rather than trusts:

- `id` is `release-` plus 64 lowercase hex; the prefix alone is not a format, and the GLOB is
  case-sensitive so the uppercase spelling is refused too;
- `manifest_sha256` is 64 lowercase hex;
- `release_version` is non-empty, at most 64 characters, and carries no `/`, no `\`, no tab, newline or
  carriage return — the same rules `ReleaseVersion::parse` applies, because this value also reaches the
  proposed bundle directory name;
- a release is never its own baseline;
- UPDATE and DELETE are both refused by triggers: a published release happened, and re-pointing the row
  at other bytes would be the same act as rewriting a stored Gate verdict.

Storage keeps **no release semantics**: it does not look at a Gate disposition, because §8 of the P4
prompt (only a `PASS` may be packaged) is enforced by `ReleaseModel::validated` in Core. A second copy of
that rule here would be a second rule book.

Typed APIs (§54 of the P4 prompt), none exposing rusqlite: `persist_release_record` writes one row and
reports `AlreadyStored` for a repeat of the same content, while the same release id over different facts
is `StorageError::Invariant` and writes nothing; `release_record_by_id` reads it back. A malformed id,
digest or version is refused before any statement runs. `ERR-STORAGE-4006` still covers a write failure
(including a foreign key to a build or run that is not stored) and `ERR-INTERNAL-9002` the invariant.

Covered by `crates/firmwaresight-storage/tests/release_records.rs` (17 tests): fresh → v4, v1 → v4,
v2 → v4 and v3 → v4 with Gate runs and accepted reviews preserved, a future version refused without a
silent reset, an obstructed 0004 that rolls back to a recoverable v3 file, insert/read round trip, dedupe,
collision mismatch, immutability attempted by raw SQL, both foreign keys, baseline optional and distinct,
no path or blob column read out of `pragma_table_info`, and every CHECK re-checked through a
hand-written `INSERT`.

Three tests pinned the literal `3` and were updated by name rather than deleted:
`gate_history.rs::a_fresh_database_carries_the_gate_tables_it_was_migrated_for`,
`compare_candidates.rs::compare_added_no_schema_change`, and
`map_companion_persistence.rs::closing_the_identity_gap_adds_no_schema_version_and_no_migration`; the
desktop's `real_artifact_intake.rs::the_selection_path_adds_no_schema_migration` now asserts the migration
**names** instead of a count.
