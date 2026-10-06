---
title: "P5 Migration and Recovery Report"
doc_id: "FS-P5-MIGRATION-RECOVERY"
product: "FirmwareSight"
version: "0.6.0"
status: "VALIDATED"
stage: "P5_PRODUCTIZATION"
owner: "Engineering"
last_updated: "2026-10-06"
---

# P5 — migration and recovery, and exactly which layer proves what

Prompt §31 asks for a boundary, not a boast: separate what the **storage integration suite** proves from
what the **installed F1 binary** was actually made to do, and do not let the second inherit the first.
This file is that boundary. It was written by Commit F2, which ran the installed half.

The short version: **one installed migration path was executed — synthetic v4 → v5 — and it passed on 34
measured checks.** Everything older is a storage-level proof. The reason that matters is concrete: the
owner's real store on this machine turned out to be at **schema v2**, so the chained v2 → v5 upgrade that a
returning pre-P5 user would actually receive has never been run through an installed build by anyone.

## 1. What the schema is, and where the version lives

`SCHEMA_VERSION` is **5**, and it is recorded in the `schema_migrations` table, **not** in
`PRAGMA user_version`. Anyone who checks `user_version` will find it unmanaged and conclude the wrong thing.

| Migration | What it does |
| --- | --- |
| `0001_initial` | projects, builds, artifacts, sections, symbols, evidence, memory_footprints |
| `0002_evidence_keyed_by_build` | evidence keyed by build |
| `0003_gate_history` | `gate_runs`, `gate_findings`, `accepted_reviews`, with immutability triggers on the Gate tables |
| `0004_release_records` | `release_records` |
| `0005_unknown_reasons` | **additive only**: `sections.file_offset_unknown`, `symbols.address_unknown`, index `idx_builds_created` |

`0005` exists to close L6 and L7 — a numeric `Unknown` used to arrive with no reason attached. It is
additive, which F2 verified rather than trusted (see §3).

Before any older store is migrated, the product takes a pre-migration snapshot named
`{stem}.pre-migration-v{from}-to-v{to}.sqlite`, in the store's own directory, one per transition, using a
WAL-aware online backup.

## 2. What the integration suite proves

These run in `cargo test`, on three CI platforms, against real SQLite files. They are genuine proofs of
the migration machinery. They are **not** proofs about the installed application.

| Path or property | Test |
| --- | --- |
| fresh → v5 | `a_fresh_database_migrates_to_the_supported_version` (`tests/storage.rs:73`) |
| v1 → v5, v2 → v5, v3 → v5, v4 → v5, each snapshotted at the version it was found at | `every_older_file_backed_schema_is_snapshotted_at_the_version_it_was_found` (`tests/integrity_and_backup.rs:582`), looping `for found in 1..=4` |
| a v4 store is snapshotted before the upgrade reaches it | `a_v4_store_is_snapshotted_before_the_upgrade_reaches_it` (`:499`) |
| fresh store writes no backup | `a_fresh_store_writes_no_migration_backup` (`:677`) |
| **v5 reopen** writes no backup and re-runs nothing | `a_store_already_at_this_version_backs_nothing_up_on_reopen` (`:695`) |
| **failure rollback**: an unwritable snapshot stops the upgrade and leaves the store at v4 | `a_snapshot_that_cannot_be_written_stops_the_upgrade_and_leaves_the_store_at_v4` (`:727`) |
| a repeated transition keeps exactly one snapshot, replaced only by a written one | `a_repeated_transition_keeps_one_snapshot_and_replaces_it_only_with_a_written_one` (`:806`) |
| integrity: a migrated store is healthy; a corrupt store is reported unhealthy and **left alone**; a many-problem store stays bounded | `a_store_this_build_migrated_is_reported_healthy` (`:300`), `a_store_whose_pages_disagree_with_themselves_is_reported_unhealthy_and_left_alone` (`:376`), `a_store_that_names_many_problems_is_still_bounded` (`:422`) |
| a failed Gate migration leaves the previous schema intact and recoverable | `a_failed_gate_migration_leaves_the_previous_schema_intact_and_recoverable` (`tests/gate_history.rs:314`) |

So all seven §31 rows on the "integration tested" side are covered: `fresh→v5`, `v1→v5`, `v2→v5`, `v3→v5`,
`v4→v5`, `v5 reopen`, `failure rollback`, and `backup/integrity`.

## 3. What the installed F1 binary proved

One path, executed by the shipping `firmwaresight-desktop.exe` (`93be8c8e…`, 15,362,048 B) launched from
the Start Menu shortcut, against a disposable store at the live app-data path.

**The database under test was repository-valid, not hand-rolled.** `target/f2_build_v4_store.py` applied the
repository's own migrations `0001`–`0004` with `PRAGMA foreign_keys = ON`, then wrote the bookkeeping rows a
v4 build would have written. Pristine digest `59f90340526f3b8185b098b3736457ee5211c9eea95ec9cdb98d24801054db41`,
180,224 bytes, containing 1 project, 2 COMPLETE builds, 2 artifacts, 4 sections, 4 symbols, 6 evidence rows
(one of them the L15 legacy `ElfProgramHeader`), 2 memory footprints, 1 PASS gate run with 10 findings,
1 accepted review and 1 release record. It was genuinely History-readable before the migration, so "rows
preserved" could be measured rather than asserted.

**Launch surface.** The proof launch was `Start-Process` on the Start Menu `.lnk` at 07:56:19. An earlier
direct console launch (07:49–07:54) is filed under `diagnostic_direct_launch/` with a `NOTE.txt` explaining
why it is a diagnostic and not the proof; the live path was reset to the pristine v4 file before the real
attempt, digest re-verified.

**Every §31 item, measured** (`03_migration/MIGRATION_PROOF.txt`, `migration_comparison.json`,
`index_comparison.txt`):

| Item | Result |
| --- | --- |
| app starts | YES — window `FirmwareSight - Analyze`, pid 21688, Analyze page and Getting-started panel rendered |
| pre-migration snapshot created | YES — `firmwaresight-p0.pre-migration-v4-to-v5.sqlite`, 180,224 bytes, 07:56:20, beside the store |
| name identifies the transition | YES — `.pre-migration-v4-to-v5` is the whole infix |
| snapshot opens | YES, read-only |
| snapshot schema is v4 | YES — `MAX(version) = 4`, migrations 1–4 |
| live schema is v5 | YES — `MAX(version) = 5`, migrations 1–5 |
| expected rows preserved | YES — **34/34 checks pass**: all 11 data tables byte-equal on their common columns between snapshot and live |
| History still reads them | YES — the product's own `history_builds` / `history_gate_runs` / `history_releases` SQL returns exactly the pre-migration rows: 2 builds, 1 PASS gate run, 1 release, 6 evidence rows |
| `integrity_check` on both | `ok` on the snapshot and on the live store |
| migration 0005 present once | YES — one row `(5, '0005_unknown_reasons')`; `GROUP BY version HAVING COUNT(*) > 1` returns 0 in both files |
| second reopen creates no duplicate snapshot | YES — closed via the window, relaunched from the same `.lnk`, still exactly one snapshot file, unchanged at 180,224 bytes / 07:56:20, every count and History row identical |

**What 0005 actually did, measured rather than assumed.** The snapshot holds 12 `idx_*` indexes and the live
store holds 13, the only addition being `idx_builds_created`; the only new columns anywhere are
`sections.file_offset_unknown` and `symbols.address_unknown`; no other table changed shape and no row was
rewritten. That is the additive behaviour `0005_unknown_reasons.sql` documents, confirmed against the
database the installed binary produced.

**`INSTALLED_SYNTHETIC_MIGRATION = PASS`** — for v4 → v5, one path.

## 4. The boundary, stated where it could mislead

| Claim | Layer that supports it |
| --- | --- |
| fresh → v5, v1 → v5, v2 → v5, v3 → v5, v4 → v5, v5 reopen, failure rollback, backup and integrity | **storage integration tests**, run on three CI platforms |
| v4 → v5 through the **installed** binary, launched from an OS surface, with the result visible in History | **F2, measured above** |
| v1/v2/v3 → v5 through an installed binary | **nobody, yet** |
| Any real user's store migrating | **not claimed** — no real user store exists in this repository's evidence |

The third row is not a hypothetical gap. When F2 restored the owner's store it read it back, and it is at
**schema_version 2** with 2 builds and `integrity_check ok` — recorded twice, in
`17_owner_restore/RESTORE.txt` and `RESTORE_second_pass.txt`. A returning pre-P5 user on this very machine
would receive a **three-migration chained upgrade (0003 + 0004 + 0005) in one app launch**, and that exact
sequence has only ever been executed by the integration suite, never by the shipping artifact.

That is filed as **F2-2, `CARRIED_FORWARD — INSTALLED_PATH_UNCOVERED`** in `P5_KNOWN_LIMITATIONS.md` §6.
Closing it needs a synthetic v2 store at the live path and one installed launch — a small job, and one F2
did not do, because §31 asks for the boundary to be honest rather than for the coverage to be complete, and
because inventing coverage here would be the exact overclaim §31 forbids.

## 5. Two honesty notes about the evidence itself

- The snapshot's `-wal` (0 bytes) and `-shm` files are harness residue: SQLite created them when this round
  opened the snapshot read-only. The snapshot's **main file is untouched** — 180,224 bytes, mtime 07:56:20
  before and after those opens.
- The live store's `-wal` stayed at 20,632 bytes across a close, because the desktop process exits without
  a WAL checkpoint and the next open recovers from it. **Byte-for-byte hashing of the main file alone is
  therefore not a retention test anywhere in this round** — which is why every retention claim in P5 is
  built on row-level reads instead. This is also why prompt §21's warning about whole-file hashing is
  followed rather than noted.

## 6. Recovery surfaces that exist, and what they are for

| Surface | What it gives a user |
| --- | --- |
| `PRAGMA integrity_check`, owned by storage | a health verdict in the Diagnostics export (`health: healthy`) and a refusal to migrate a store it reports as unhealthy |
| pre-migration snapshot | a byte-exact copy of the store as it was found, named with the transition it precedes, kept beside the store |
| fail-closed upgrade | an unwritable snapshot stops the migration and leaves the store at its old version rather than half-migrated |
| Gate immutability triggers | a stored Gate run cannot be edited; a re-run adds a row rather than replacing one — the History page says this in words |
| Diagnostics export | the store's version, health, journal mode, bounded counts and snapshot names, with no path in it — see `P5_HISTORY_DIAGNOSTICS_REPORT.md` |

What P5 does **not** offer, and does not claim: any in-app restore button, any automatic rollback, and any
backup beyond the pre-migration snapshot. Recovery today is "the snapshot is on disk and the product will
tell you it is there", which is honest and is also the ceiling.
