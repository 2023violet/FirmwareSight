---
title: "SQLite Storage Baseline"
doc_id: "FS-TECH-016"
product: "FirmwareSight"
version: "0.5.0"
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
