---
title: "SQLite Storage Baseline"
doc_id: "FS-TECH-016"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-10-09"
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

## P5 numeric Unknown reasons (2026-10-03)

`SCHEMA_VERSION` moves 4 → 5 with one additive migration, `migrations/0005_unknown_reasons.sql`. It adds
two columns and one index and touches no existing row:

```
ALTER TABLE sections ADD COLUMN file_offset_unknown TEXT;
ALTER TABLE symbols  ADD COLUMN address_unknown     TEXT;
CREATE INDEX idx_builds_created ON builds(created_at);
```

Why: `0001_initial.sql` states "a fact we could not determine is recorded as a reason, never silently as
zero" and pairs five of its seven nullable numeric columns with a `*_unknown` twin, but
`sections.file_offset` and `symbols.address` have none, so `optional_fact_u64` discarded the reason the
parser handed it. That is L6/L7 in `G2_VALIDATION/G2_KNOWN_LIMITATIONS.md`. The writer now uses the same
`optional_fact` pair the neighbours use, `SectionRow::file_offset` and `SymbolRow::address` are
`Fact<u64>` like every other numeric column, and the desktop's `SectionRowDto` / `SymbolRowDto` carry
`fileOffsetUnknownReason` / `addressUnknownReason` so the Analyze tables explain an Unknown the way the
Evidence Inspector already does. `P5_VALIDATION/P5_MIGRATION_DECISION.md` holds the decision, the rejected
alternatives and the checkpoint that authorized it.

What this migration is **not**: it is not History's. Every row a History page shows is already persisted,
and §18 forbids a migration written to make a screen easier. The index on `builds.created_at` rides along
because `builds` is the one history table whose ordering column has no index while every comparable column
does; a History read model works without it.

A `NULL` in either new column is a third state and stays one: it means the version that wrote the row did
not record a reason, and `query.rs::bytes_or_unknown` renders that as the words "no reason was recorded"
rather than reconstructing a cause. No backfill is attempted — re-parsing an artifact can disagree with
what was originally observed, and replacing a lost fact with an invented one is the failure this rule
exists to prevent.

Diff semantics and the portable documents did not change. `DiffSection::file_offset` and
`DiffSymbol::address` keep their existing `Option<u64>` shape (`compare.rs`, `report/diff.rs`,
`desktop/src/compare.rs`), so `diff.json`, `diff.html` and the CLI↔desktop parity claims are byte-for-byte
what P2 sealed; only the stored-detail path gained the reason.

Covered by `crates/firmwaresight-storage/tests/unknown_reasons.rs` (5 tests): a committed fixture's
unknown `.bss` offset keeps its reason through the write and comes back through the paging query as
`Fact::Unknown` with that reason; a symbol whose address is reported Unknown keeps it (the ELF parser does
not emit that today, so the case is constructed, not claimed); the schema is at version 5 with both
columns present; and a hand-built v4 file upgrades with its `.bss` row intact and its reason still
unrecorded. Four named-stage guards that pinned the old migration list were updated rather than deleted —
`compare_candidates.rs::compare_added_no_schema_change`,
`map_companion_persistence.rs::closing_the_identity_gap_adds_no_schema_version_and_no_migration`,
`release_records.rs::the_schema_is_at_version_five_and_the_release_table_starts_empty` (renamed from
"version four") and the desktop's `real_artifact_intake.rs::the_selection_path_adds_no_schema_migration` —
and `release_records.rs::a_database_from_the_future_is_refused_and_left_alone` now matches the refusal
against `SCHEMA_VERSION` instead of a literal, so the guard tests the behaviour rather than the number.

One sequencing rule travels with this schema: a schema-5 store cannot be opened by a schema-4 binary — it
is refused, not reset — so the recovery path for an upgrade is a backup taken before it. That capability
does not exist yet (P5 prompt §22, which lands in the commit §61 calls "diagnostics and recovery
support"), which is why `P5_MIGRATION_DECISION.md` states that no build carrying this migration may be
installed over a user's real store before it does.

## C1-U1 release attachment facts (2026-10-09)

`SCHEMA_VERSION` moves 5 → 6 with one additive migration, `migrations/0006_release_attachments.sql`. It
creates one table and touches nothing else: no earlier table is altered, renamed or re-keyed, and no
existing row changes, so a run stored before this migration still reads back with the same id and an empty
attachment set.

`gate_run_attachments` is **the set of raw-byte facts one stored Gate run bound**, not a file catalogue and
not a copy of the bytes:

| column | what it fixes |
| --- | --- |
| `run_id` | `gate_runs(id)` `ON DELETE CASCADE` — the run that judged these bytes |
| `ordinal` | the canonical position `04_TECH/28` §5 rule 1 defines, so a re-read rebuilds the same text without sorting again |
| `kind` | `bin` / `hex` / `unknown` — an analyzed kind cannot be stored |
| `sha256` | 64-character lowercase hex, the exact spelling the canonical `attachments[…]` block prints |
| `byte_size` | a positive length: a zero-byte row would describe an image that ships nothing |
| `kind_basis` | `declared` or `derived_from_leading_bytes`, the evidence class of the kind claim |

`PRIMARY KEY (run_id, ordinal)` and `UNIQUE (run_id, kind, sha256)` are the storage form of "one exact
`(kind, digest)` pair, bound once, in one order". The table stores no bytes (the `RawInputBytes` rule of
`0001_initial.sql:8`), no name, no path and no mtime (`AGENTS.md` §7): one row is a digest, a length, a
kind and the basis for that kind.

Why `artifacts` was **not** reused: `0001_initial.sql` makes `parser_id`, `architecture`, `bitness` and
`endianness` `NOT NULL`. A BIN or HEX attachment is by definition a file nothing parsed, so storing it
there would require inventing analysis values for it — the exact failure `ADR-0030` exists to prevent. The
two tables therefore mean two different things: `artifacts` is what an import observed about a parsed file
of a build, `gate_run_attachments` is what one verdict bound.

`gate_run_attachments_are_immutable` refuses UPDATE for the same reason `gate_runs` and `gate_findings` do:
editing which bytes a stored verdict was judged from is rewriting the verdict. Deleting the run still
cascades.

Write path (`crates/firmwaresight-storage/src/gate.rs`, no rusqlite escapes):
`GateRunDraft.attachments` is a borrow of the same `GateAttachmentFact` rows the Gate canonicalized, and
`persist_gate_run` refuses, **before any statement runs**, a draft whose rows are not the canonical set or
whose attachment carries no observed digest. Rows insert in the same transaction as the run and its
findings, so a violation leaves no half-stored run. Replaying the same `run_id` over a different attachment
set is `StorageError::Invariant`, not an overwrite.

Read path: `gate_run_by_id` returns the rows in stored `ordinal` order and `StoredGateAttachment::as_gate_fact`
rebuilds them, which is what makes "a re-read restores the same canonical text and the same run id" a
checked property rather than a claim (`gate_history.rs::a_re_read_of_the_stored_rows_rebuilds_the_same_canonical_input_and_run_id`).
A `derived_from_leading_bytes` row is reported as an invariant instead of being read back as
`KindBasis::DerivedFromLeadingBytes` with a fabricated sample, because this schema persists the basis word
and not the bytes it was derived from.

Covered by `crates/firmwaresight-storage/tests/gate_history.rs` (31 tests, 10 of them this unit's): the
canonical ordinal read-back, the `/2` re-read, same-id-different-attachment-set refusal, non-canonical and
unobserved-digest drafts refused with zero rows written, a zero-attachment run storing no rows, eight
boundary CHECK refusals re-checked through hand-written `INSERT`s, the three allowed kind words, UPDATE
refusal by raw SQL, transaction rollback leaving no half-stored run, and the derived-basis read refusal.
`release_records.rs` adds `a_v4_store_holding_a_gate_run_gains_the_attachment_table_and_keeps_the_run_readable`,
so the 5 → 6 step is proven against a v4 file that holds real Gate rows.

Six guards that pinned the old migration list were updated by naming the sixth stage rather than weakened:
`compare_candidates.rs`, `map_companion_persistence.rs`, `release_records.rs` (whose
`the_schema_is_at_version_five_…` is now `the_schema_is_at_this_builds_version_…` and whose
`step_down_to_v3` drops the new table), `unknown_reasons.rs`, the desktop's `real_artifact_intake.rs`, and
the two hand-built v4 stores in `startup.rs` / `diagnostics.rs`. One user-visible consequence follows: a v4
store's pre-migration backup is now named `…v4-to-v6.sqlite` instead of `…v4-to-v5.sqlite`, because the
backup name records the step this build actually takes.

Nothing here makes an attachment usable by a person: no UI control, CLI flag or IPC command offers a file
yet, and `artifacts.required` / `artifacts.hashes` still judge only the analyzed artifacts. Those are
C1-U2 and later.
