-- P5 productization: complete the Unknown-reason rule for the two numeric columns that predate it.
--
-- `0001_initial.sql` states the rule next to its first nullable column — "A fact we could not determine
-- is recorded as a reason, never silently as zero" — and pairs five of its seven nullable numeric columns
-- with a `*_unknown` text column (`entry_point`/`entry_unknown`, `virt_addr`/`virt_unknown`,
-- `load_addr`/`load_unknown`, `mem_size`/`mem_unknown`, `size`/`size_unknown`). `sections.file_offset` and
-- `symbols.address` were left without a twin, so `optional_fact_u64` had nothing to write the reason into
-- and dropped it: the window could say a byte offset was Unknown but not why, which is L6 and L7 in
-- `G2_VALIDATION/G2_KNOWN_LIMITATIONS.md`. The decision, its limits and the alternatives rejected are in
-- `P5_VALIDATION/P5_MIGRATION_DECISION.md`.
--
-- Additive only. No table is rebuilt, renamed or re-keyed and no existing row is rewritten: SQLite's
-- `ADD COLUMN` with a nullable, constraint-free type leaves every stored row in place and reads back
-- NULL, and the index covers rows that already exist. A v1, v2, v3 or v4 database keeps every row it
-- holds and gains two columns and one index.
--
-- A NULL in these two new columns is a third state, not an empty reason. It means *the version that
-- wrote this row did not record one*, and the read layer says so in words ("no reason was recorded",
-- `query.rs::bytes_or_unknown`) rather than reconstructing a cause from columns that never held it.
-- Backfilling reasons is deliberately not part of this migration: re-parsing an artifact can disagree
-- with what was originally observed, and replacing a lost fact with an invented one is the failure this
-- whole rule exists to prevent.
--
-- The index is not what justifies this migration — History is implementable without it, and `0001`
-- indexes the two columns it created for lookups. `builds.created_at` is the one ordering column in the
-- history shape that has no index while every comparable column does (`idx_gate_runs_created`,
-- `idx_release_records_build`), so the inconsistency is closed in the same additive slot.

ALTER TABLE sections ADD COLUMN file_offset_unknown TEXT;

ALTER TABLE symbols ADD COLUMN address_unknown TEXT;

CREATE INDEX idx_builds_created ON builds(created_at);
