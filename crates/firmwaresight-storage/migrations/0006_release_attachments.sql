-- C1-U1 release attachments: the raw-byte facts one Gate run bound, stored with the run that judged them.
--
-- Additive only. No earlier table is touched, renamed or re-keyed and no existing row changes, so a run
-- stored before this migration still reads back with the same id and an empty attachment set
-- (`04_TECH/28` §7.7, `ADR-0030` D-9).
--
-- The `artifacts` table of `0001_initial.sql` is deliberately not reused: it requires `parser_id`,
-- `architecture`, `bitness` and `endianness` to be `NOT NULL`, so a BIN row there would have to carry
-- invented analysis values for a file nothing parsed.
--
-- Nothing here stores a file's bytes (the `RawInputBytes` rule of `0001_initial.sql:8`) or a host path
-- (`AGENTS.md` §7): one row is a digest, a length, a kind and the basis on which that kind was stated.
--
-- Four invariants are enforced by constraints rather than trusted to the caller, because these rows are
-- the evidence a stored verdict was judged from:
--
-- * the three kinds a release may attach, and no analyzed kind — `elf` and `map` are produced by the
--   analysis path (`04_TECH/28` §2.2);
-- * a 64-character lowercase hex digest, which is exactly the spelling the canonical `attachments[...]`
--   block prints, so a row cannot record an identity that hashes differently from the text it came from;
-- * a positive length: `AttachmentEmpty` (E-3) stated again at the storage boundary, because a 0-byte
--   row would describe an image that ships nothing;
-- * `ordinal` is the canonical position §5 rule 1 defines, so a re-read rebuilds the same text without
--   sorting again, and `UNIQUE (run_id, kind, sha256)` is the storage form of binding one exact
--   `(kind, digest)` pair once.
--
-- Immutability follows the run: an attachment row of a stored run is edited by nobody, for the same
-- reason `gate_runs` and `gate_findings` refuse an UPDATE. Deleting a run still cascades.

CREATE TABLE gate_run_attachments (
    run_id     TEXT NOT NULL REFERENCES gate_runs(id) ON DELETE CASCADE,
    ordinal    INTEGER NOT NULL CHECK (ordinal >= 0),
    kind       TEXT NOT NULL CHECK (kind IN ('bin','hex','unknown')),
    sha256     TEXT NOT NULL CHECK (length(sha256) = 64
                                    AND sha256 = lower(sha256)
                                    AND sha256 NOT GLOB '*[^0-9a-f]*'),
    byte_size  INTEGER NOT NULL CHECK (byte_size > 0),
    kind_basis TEXT NOT NULL CHECK (kind_basis IN ('declared','derived_from_leading_bytes')),
    PRIMARY KEY (run_id, ordinal),
    UNIQUE (run_id, kind, sha256)
);

CREATE TRIGGER gate_run_attachments_are_immutable
BEFORE UPDATE ON gate_run_attachments
BEGIN
    SELECT RAISE(ABORT, 'a stored attachment row is immutable; run the Gate again over the new bytes');
END;
