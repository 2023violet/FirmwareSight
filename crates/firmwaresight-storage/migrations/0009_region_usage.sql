-- D8 / ADR-0041 D1: per-region usage, stored with the build it describes.
--
-- Additive only. No earlier table is touched, renamed or re-keyed and no existing row changes. There is no
-- backfill (DR1): a build stored before this migration has no set row and reads as "stored before this
-- version", which is Unknown and is never shown as zero.
--
-- Two tables, because three different answers must stay apart:
--
-- * no `region_usage_sets` row      -> stored before this version (Unknown);
-- * a set row with region_count = 0 -> the build had no region table (no MAP, no layout);
-- * region rows                     -> the usage the analysis recorded.
--
-- A region name is the linker's region name, never a host path (AGENTS.md §7). `capacity_bytes` and
-- `used_bytes` are NULL when Unknown, are kept apart from 0, and `reasons` says why. `ordinal` keeps the
-- analysis's own order.
-- Rows follow their build: an edit is refused like every other stored fact, and deleting a build cascades.

CREATE TABLE region_usage_sets (
    build_id      TEXT    PRIMARY KEY NOT NULL REFERENCES builds(id) ON DELETE CASCADE,
    layout_source TEXT    NOT NULL CHECK (layout_source IN ('project-config','map','none')),
    region_count  INTEGER NOT NULL CHECK (region_count >= 0)
);

CREATE TABLE region_usage (
    build_id        TEXT    NOT NULL REFERENCES region_usage_sets(build_id) ON DELETE CASCADE,
    ordinal         INTEGER NOT NULL CHECK (ordinal >= 0),
    name            TEXT    NOT NULL CHECK (length(name) > 0),
    kind            TEXT    NOT NULL CHECK (kind IN ('nonvolatile','volatile_ram','unknown')),
    origin          INTEGER NOT NULL,
    capacity_bytes  INTEGER CHECK (capacity_bytes >= 0),
    used_bytes      INTEGER CHECK (used_bytes >= 0),
    state           TEXT    NOT NULL CHECK (state IN ('exact','partial','unknown')),
    risk            TEXT    NOT NULL CHECK (risk IN ('within_capacity','exceeded','review','unknown')),
    evidence_source TEXT    NOT NULL,
    -- Why a value is Unknown or partial, as the analysis said it (one reason per line); NULL when none.
    reasons         TEXT,
    PRIMARY KEY (build_id, ordinal)
);

CREATE TRIGGER region_usage_sets_are_immutable
BEFORE UPDATE ON region_usage_sets
BEGIN
    SELECT RAISE(ABORT, 'region usage is immutable');
END;

CREATE TRIGGER region_usage_is_immutable
BEFORE UPDATE ON region_usage
BEGIN
    SELECT RAISE(ABORT, 'region usage is immutable');
END;
