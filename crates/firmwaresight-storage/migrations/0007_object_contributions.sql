-- M2 / ADR-0033 D5: the GNU ld MAP's object rows, stored with the build they describe.
--
-- Additive only. No earlier table is touched, renamed or re-keyed and no existing row changes. A build
-- stored before this migration simply has no rows here, and the read layer reports its attribution as
-- unavailable rather than as zero.
--
-- No host path is stored (AGENTS.md §7): `object_key`, `archive` and `member` are the ADR-0033 D2
-- normalized names the MAP adapter produced, never the path the linker printed.
--
-- `ordinal` keeps the MAP's own row order, so a re-read rebuilds the same sequence without sorting.
-- Rows follow their build: an edit is refused like every other stored fact, and deleting a build
-- cascades.

CREATE TABLE object_contributions (
    build_id       TEXT    NOT NULL REFERENCES builds(id) ON DELETE CASCADE,
    ordinal        INTEGER NOT NULL CHECK (ordinal >= 0),
    object_key     TEXT    NOT NULL CHECK (length(object_key) > 0),
    archive        TEXT,
    member         TEXT    NOT NULL,
    input_section  TEXT    NOT NULL,
    output_section TEXT,
    address        INTEGER NOT NULL,
    size           INTEGER NOT NULL CHECK (size > 0),
    map_line       INTEGER NOT NULL CHECK (map_line > 0),
    PRIMARY KEY (build_id, ordinal)
);

CREATE INDEX object_contributions_by_key ON object_contributions (build_id, object_key);

CREATE TRIGGER object_contributions_are_immutable
BEFORE UPDATE ON object_contributions
BEGIN
    SELECT RAISE(ABORT, 'object contributions are immutable');
END;
