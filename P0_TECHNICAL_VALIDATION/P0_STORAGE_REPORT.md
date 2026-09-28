---
title: "P0 Storage Report"
doc_id: "FS-P0-010"
product: "FirmwareSight"
version: "0.5.1"
status: "EXECUTION_RECORD"
owner: "Engineering"
last_updated: "2026-09-28"
---

# P0 Storage Report

Requirement: structured history lives in SQLite via `rusqlite + bundled`; schema changes go
through numbered migrations; a partial import is never observable as a finished one; and an
unrecognized schema is never repaired by deleting data.

Status: **LOCAL PASS**, after one schema defect was found and repaired

## Result

```
$ cargo test -p firmwaresight-storage
running 11 tests ... test result: ok. 11 passed; 0 failed
```

| Test | What it forbids |
| --- | --- |
| `a_fresh_database_migrates_to_the_supported_version` | a schema applied without a `schema_migrations` record |
| `reopening_an_existing_database_is_idempotent` | history being rewritten or duplicated on second open |
| `the_connection_carries_the_pragmas_the_schema_assumes` | `establish()` silently losing FK enforcement or the busy timeout |
| `foreign_keys_are_enforced_on_the_connection` | orphan rows that a later phase would read as real history |
| `a_snapshot_survives_a_full_round_trip_with_identical_facts` | a stored build drifting from the snapshot it came from |
| `per_section_and_evidence_detail_is_retrievable` | detail being flattened away into totals only |
| `a_failed_import_leaves_no_build_visible_as_complete` | a half-written import being visible as `COMPLETE` |
| `the_same_snapshot_is_never_stored_twice` | one artifact producing N identical builds because the UI was clicked N times |
| `an_unknown_newer_schema_is_refused_rather_than_reset` | an older binary deleting a newer database |
| `two_builds_may_record_the_same_evidence_identifier` | a second artifact being unstashable because evidence ids repeat across builds |
| `a_version_one_database_is_upgraded_without_losing_its_evidence` | an upgrade rebuilding a table and dropping the rows that were in it |

The last two were written after the desktop smoke found the defect they describe, and they fail
against the pre-fix schema - the first with the same `UNIQUE constraint failed: evidence.id` the
window displayed.

## Schema

Two migrations, applied in version order, each inside its own transaction and recorded in
`schema_migrations (version, name, applied_at)`:

- `migrations/0001_initial.sql` - the schema itself.
- `migrations/0002_evidence_keyed_by_build.sql` - rebuilds `evidence` on `PRIMARY KEY (build_id, id)`.

Tables: `projects`, `builds`, `artifacts`, `sections`, `symbols`, `evidence`,
`memory_footprints`. Constraints that carry meaning rather than decoration:

- `builds.state CHECK (state IN ('IMPORTING','COMPLETE','FAILED'))`
- `artifacts CHECK (length(sha256) = 64)`, with `entry_point` nullable alongside
  `entry_unknown` so an unknown stays distinguishable from a zero
- `evidence.classification CHECK`, and since `0002` a key of `(build_id, id)`, so one item cannot be
  recorded twice under two different claims *within a build* while different builds may each record
  the same-named fact. Version 1 keyed `id` alone, which contradicted `04_TECH/15` 4
  (`Build 1─N Evidence`) and made a second build impossible
- `sections` and `symbols` keyed by `(build_id, ordinal)`, `memory_footprints` keyed by `build_id`
- `ON DELETE CASCADE` from every child to `builds`

Raw artifact bytes are never stored. The database holds paths, hashes and derived facts.

## Transaction discipline

`import_snapshot` writes the build row as `IMPORTING`, writes artifacts, sections, symbols,
evidence and memory rows, then flips the build to `COMPLETE` **as the last write inside the same
transaction**, and commits.

`a_failed_import_leaves_no_build_visible_as_complete` forces the failure with a duplicate
evidence id and then counts: `builds = 0`, `sections = 0`, `projects = 0`, and a subsequent clean
import still yields `state = 'COMPLETE'`. The database is not left wedged by the failure.

## Refusing to guess about a newer schema

`an_unknown_newer_schema_is_refused_rather_than_reset` writes `version = 99` into
`schema_migrations`, reopens, and asserts both that the open fails with
`UnsupportedSchemaVersion` and that the row is still there afterwards. There is no delete-and-
recreate path in the crate; the remediation text tells the user to run the FirmwareSight version
that wrote the database.

## Connection facts observed on this host

```
$ cargo test -p firmwaresight-storage --test storage -- the_connection --nocapture
OBSERVED journal_mode=wal
```

`PRAGMA foreign_keys = ON`, a 5 s `busy_timeout`, and WAL requested where supported - observed
here as `journal_mode=wal`. The test asserts the pragmas rather than the specific mode, because
WAL availability is a property of the filesystem, not of the product.

## Desktop integration

The desktop shell opens its database under the platform application-data directory
(`firmwaresight-p0.sqlite`) and imports each analysis through the same function the tests drive.
`analyzing_the_same_fixture_twice_does_not_duplicate_history` clicks three times and asserts one
build row, using `build_id_for_snapshot` to recognize a content-addressed snapshot that is
already stored.

That dedupe guard is also why the shell's own tests never reached the defect above: it prevents a
*repeat of one snapshot*, which is the collision the key happened to catch, while a *second distinct
snapshot* reusing an evidence identifier is the case the key made impossible. Only a run against a
persistent, already-populated database - which is what a desktop app actually has - reaches it. The
real file at `%APPDATA%\com.firmwaresight.desktop\firmwaresight-p0.sqlite` was left in place and
upgraded by the rebuilt binary:

```text
schema_migrations: [(1, '0001_initial'), (2, '0002_evidence_keyed_by_build')]
builds:            2, both COMPLETE       evidence rows: 21 (11 + 10)
ev-sha256 rows:    2, one per build       primary key:   (build_id, id)
```

## Boundaries held

- `rusqlite` appears in exactly one crate; `firmwaresight-core` still has an empty
  `[dependencies]` table.
- No SQL reaches the WebView: there is no `run_sql` command, and the storage crate is not
  exposed through IPC.
- No migration is destructive. `0002` rebuilds one table and copies every row unchanged - no
  `DROP` of data, no re-import, no reset path - and the upgrade test asserts a pre-existing row's
  value survives it. P0 does now have a database worth upgrading: the developer's own.

## Carried forward

Gate and release tables are deliberately absent - `0001_initial.sql` stops at what P0 writes,
with names chosen so later phases add tables instead of reshaping these.
