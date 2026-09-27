-- Initial SQLite schema for FirmwareSight structured history.
--
-- Shapes follow 04_TECH/15_STORAGE_DATABASE_BASELINE.md and the entities in
-- 04_TECH/02_DOMAIN_MODEL.md. Gate and release tables arrive with their own phases; this file
-- deliberately stops at what P0 writes, but every name and relation below is chosen so later
-- phases can be added without reshaping what is here.
--
-- Raw artifact bytes are never stored. Only path metadata, hashes and derived facts live here.

CREATE TABLE projects (
    id         TEXT PRIMARY KEY NOT NULL,
    name       TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);

CREATE TABLE builds (
    id                     TEXT PRIMARY KEY NOT NULL,
    project_id             TEXT NOT NULL REFERENCES projects(id),
    snapshot_id            TEXT NOT NULL,
    normalization_version  TEXT NOT NULL,
    created_by_fwsight     TEXT NOT NULL,
    -- IMPORTING is how a partial write is recorded; only a committed transaction can leave a
    -- build sitting at COMPLETE.
    state                  TEXT NOT NULL CHECK (state IN ('IMPORTING','COMPLETE','FAILED')),
    created_at             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);

CREATE INDEX idx_builds_project ON builds(project_id);
CREATE INDEX idx_builds_snapshot ON builds(snapshot_id);

CREATE TABLE artifacts (
    id             TEXT PRIMARY KEY NOT NULL,
    build_id       TEXT NOT NULL REFERENCES builds(id) ON DELETE CASCADE,
    path           TEXT NOT NULL,
    kind           TEXT NOT NULL,
    sha256         TEXT NOT NULL CHECK (length(sha256) = 64),
    byte_size      INTEGER NOT NULL CHECK (byte_size >= 0),
    parser_id      TEXT NOT NULL,
    architecture   TEXT NOT NULL,
    bitness        TEXT NOT NULL,
    endianness     TEXT NOT NULL,
    -- A fact we could not determine is recorded as a reason, never silently as zero.
    entry_point    INTEGER,
    entry_unknown  TEXT,
    build_id_note  TEXT,
    build_id_unknown TEXT
);

CREATE INDEX idx_artifacts_build ON artifacts(build_id);
CREATE INDEX idx_artifacts_sha256 ON artifacts(sha256);

CREATE TABLE sections (
    build_id      TEXT NOT NULL REFERENCES builds(id) ON DELETE CASCADE,
    section_index INTEGER NOT NULL,
    name          TEXT,
    name_unknown  TEXT,
    role          TEXT NOT NULL,
    is_alloc      INTEGER NOT NULL,
    is_write      INTEGER NOT NULL,
    is_execute    INTEGER NOT NULL,
    virt_addr     INTEGER,
    virt_unknown  TEXT,
    load_addr     INTEGER,
    load_unknown  TEXT,
    file_offset   INTEGER,
    file_size     INTEGER NOT NULL CHECK (file_size >= 0),
    mem_size      INTEGER,
    mem_unknown   TEXT,
    region        TEXT,
    region_unknown TEXT,
    PRIMARY KEY (build_id, section_index)
);

CREATE TABLE symbols (
    build_id      TEXT NOT NULL REFERENCES builds(id) ON DELETE CASCADE,
    ordinal       INTEGER NOT NULL,
    name          TEXT,
    name_unknown  TEXT,
    address       INTEGER,
    size          INTEGER,
    size_unknown  TEXT,
    kind          TEXT NOT NULL,
    binding       TEXT NOT NULL,
    section_ref   TEXT NOT NULL,
    PRIMARY KEY (build_id, ordinal)
);

CREATE TABLE evidence (
    id              TEXT PRIMARY KEY NOT NULL,
    build_id        TEXT NOT NULL REFERENCES builds(id) ON DELETE CASCADE,
    field           TEXT NOT NULL,
    classification  TEXT NOT NULL CHECK (classification IN ('observed','derived','declared','unknown')),
    source_type     TEXT NOT NULL,
    source_locator  TEXT NOT NULL,
    raw_value       TEXT NOT NULL,
    rule            TEXT NOT NULL,
    confidence      TEXT
);

CREATE INDEX idx_evidence_build ON evidence(build_id);
CREATE INDEX idx_evidence_field ON evidence(build_id, field);

CREATE TABLE memory_footprints (
    build_id              TEXT PRIMARY KEY NOT NULL REFERENCES builds(id) ON DELETE CASCADE,
    layout_source         TEXT NOT NULL,
    weakest_basis         TEXT,
    admissible_hard_block INTEGER NOT NULL CHECK (admissible_hard_block IN (0, 1)),
    nonvolatile_state     TEXT NOT NULL CHECK (nonvolatile_state IN ('exact','partial','unknown')),
    nonvolatile_bytes     INTEGER,
    nonvolatile_class     TEXT NOT NULL,
    runtime_state         TEXT NOT NULL CHECK (runtime_state IN ('exact','partial','unknown')),
    runtime_bytes         INTEGER,
    runtime_class         TEXT NOT NULL,
    excluded_metadata     INTEGER NOT NULL CHECK (excluded_metadata >= 0)
);
