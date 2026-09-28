-- Evidence is a child of a build: 04_TECH/15 4 states Build 1-N Evidence, and the analyzer names
-- each fact within one analysis, so every build legitimately records `ev-sha256`, `ev-byte-size`,
-- `ev-entry` and the rest. Version 1 made `id` the whole-table primary key, which let a database
-- hold one build's evidence and refuse the next one's. The desktop shipped that way: analyzing a
-- second fixture failed with `UNIQUE constraint failed: evidence.id`, and no test had ever put two
-- builds in one database, so CI could not see it.
--
-- SQLite cannot alter a primary key, so the table is rebuilt. Column order follows the new key.
-- Nothing else in the schema references evidence, and no row content changes.

CREATE TABLE evidence_rekeyed (
    build_id        TEXT NOT NULL REFERENCES builds(id) ON DELETE CASCADE,
    id              TEXT NOT NULL,
    field           TEXT NOT NULL,
    classification  TEXT NOT NULL CHECK (classification IN ('observed','derived','declared','unknown')),
    source_type     TEXT NOT NULL,
    source_locator  TEXT NOT NULL,
    raw_value       TEXT NOT NULL,
    rule            TEXT NOT NULL,
    confidence      TEXT,
    PRIMARY KEY (build_id, id)
);

INSERT INTO evidence_rekeyed
    (build_id, id, field, classification, source_type, source_locator, raw_value, rule, confidence)
SELECT build_id, id, field, classification, source_type, source_locator, raw_value, rule, confidence
  FROM evidence;

DROP TABLE evidence;
ALTER TABLE evidence_rekeyed RENAME TO evidence;

-- Dropped with the table, and the writer and the build summary both rely on them.
CREATE INDEX idx_evidence_build ON evidence(build_id);
CREATE INDEX idx_evidence_field ON evidence(build_id, field);
