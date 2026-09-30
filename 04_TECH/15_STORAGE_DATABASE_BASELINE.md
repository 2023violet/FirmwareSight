---
title: "SQLite Storage Baseline"
doc_id: "FS-TECH-016"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-09-26"
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
`release_records` from §3 stays unwritten — a Release Bundle is P4 scope.

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
