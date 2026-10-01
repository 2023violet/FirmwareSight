-- P4 Release Bundle records: one immutable row per release that was published, so the project can say
-- which release ids it has already issued, over which build, against which Gate run, and with which
-- manifest digest.
--
-- Additive only. No earlier table is touched, renamed or re-keyed, so a v1, v2 or v3 database keeps every
-- row it holds and gains one table.
--
-- This row is an audit index, not the bundle. The bundle is the portable directory a stranger reads;
-- deleting this database cannot invalidate it, and moving the directory cannot require this database
-- (prompt §67). So nothing here stores where the bytes went:
--
-- * no bundle destination path,
-- * no project root,
-- * no artifact source path,
-- * no manifest blob and no HTML blob.
--
-- `manifest_sha256` is the digest of `release-manifest.json` as written. That is the one fact that ties
-- this row to the shipped directory without storing a path: a reader who has the bundle can re-hash the
-- file and find the record, and a reader who does not have it learns only that a release id exists.
--
-- The invariants are constraints rather than code, for the same reason `0003` states its own:
--
-- * A release id is `release-` plus a SHA-256 (72 characters), never a wall clock, a uuid or a counter, so
--   the row cannot be keyed on something that changes when nothing semantic did.
-- * `manifest_sha256` is 64 lowercase hex, because a digest in either of the other two spellings verifies
--   nothing.
-- * A release is never compared against itself: the Gate refuses that comparison and so does the row.
-- * `release_version` is the project's version, non-empty and free of path separators and control
--   characters — the same rule `firmwaresight-core`'s `ReleaseVersion::parse` applies, because this value
--   also reaches the proposed bundle directory name.
-- * `created_at` is an audit time and the only time in this table. It is deliberately NOT part of the
--   canonical release input, so re-publishing the same release writes the same id (prompt §42).
-- * A stored release record is immutable: UPDATE and DELETE are both refused below. A release that was
--   published happened; correcting history by editing it would be the same act as rewriting a Gate
--   verdict, which `0003` already refuses.

CREATE TABLE release_records (
    id                TEXT PRIMARY KEY NOT NULL,
    build_id          TEXT NOT NULL REFERENCES builds(id) ON DELETE CASCADE,
    baseline_build_id TEXT REFERENCES builds(id) ON DELETE CASCADE,
    gate_run_id       TEXT NOT NULL REFERENCES gate_runs(id) ON DELETE CASCADE,
    release_version   TEXT NOT NULL,
    manifest_sha256   TEXT NOT NULL,
    created_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),

    CHECK (length(id) = 72 AND substr(id, 1, 8) = 'release-'),
    -- The prefix alone is not a format: `release-` plus 64 *hex* characters is. GLOB is case-sensitive
    -- for ASCII, so this also refuses the uppercase spelling that verifies nothing.
    CHECK (substr(id, 9) NOT GLOB '*[^0-9a-f]*'),
    CHECK (length(manifest_sha256) = 64
           AND manifest_sha256 = lower(manifest_sha256)
           AND manifest_sha256 NOT GLOB '*[^0-9a-f]*'),
    CHECK (baseline_build_id IS NULL OR baseline_build_id <> build_id),
    CHECK (length(trim(release_version)) > 0 AND length(release_version) <= 64),
    CHECK (release_version = trim(release_version)),
    CHECK (instr(release_version, char(47)) = 0
           AND instr(release_version, char(92)) = 0),
    -- The control characters a filename-mangling platform either drops or refuses outright. Core's
    -- `ReleaseVersion::parse` rejects the whole class; this names the three that are actually reachable
    -- from a Git tag or a config value, so a hand-written INSERT cannot smuggle a line break into a
    -- directory name.
    CHECK (instr(release_version, char(9)) = 0
           AND instr(release_version, char(10)) = 0
           AND instr(release_version, char(13)) = 0),
    CHECK (created_at GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]T[0-9][0-9]:[0-9][0-9]:[0-9][0-9]Z')
);

-- A build's release history, in the order it happened. Two indexes because the two questions a release
-- owner asks are "what did this build ship" and "has this release id already been issued"; the second is
-- answered by the primary key.
CREATE INDEX idx_release_records_build ON release_records(build_id, created_at, id);
CREATE INDEX idx_release_records_run ON release_records(gate_run_id);

CREATE TRIGGER release_records_are_immutable
BEFORE UPDATE ON release_records
BEGIN
    SELECT RAISE(ABORT, 'a published release record is immutable; publish a new release instead');
END;

CREATE TRIGGER release_records_cannot_be_deleted
BEFORE DELETE ON release_records
BEGIN
    SELECT RAISE(ABORT, 'a published release record is an audit row and cannot be deleted');
END;
