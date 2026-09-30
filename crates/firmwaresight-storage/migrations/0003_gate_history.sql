-- P3 Release Gate history: one immutable GateRun per evaluation, its findings, the evidence refs each
-- finding cites, and the review acceptances that dispose of a finding without changing it.
--
-- Additive only. No earlier table is touched, renamed or re-keyed, so a v1 or v2 database keeps every
-- row it holds and gains four tables. Nothing here stores a host path: `AGENTS.md` 7 keeps the project
-- root inside the adapter that read it.
--
-- Several invariants are enforced by constraints rather than by code, because a stored Gate verdict is
-- the artifact a release owner is accountable for:
--
-- * ADR-0023's state/severity pairing is a CHECK, so `UNKNOWN` cannot be written as a PASS and a UI or a
--   CLI cannot become the place where an evidence gap quietly turns into a green light.
-- * One finding per rule per run (UNIQUE), so a re-run of the same deterministic id cannot duplicate
--   findings.
-- * A run id is `gate-` plus a SHA-256, never a wall clock or a uuid.
-- * A stored verdict is immutable: the triggers below abort an UPDATE of a run or a finding, and
--   acceptance rows additionally abort DELETE. Acceptance is an audit record, so editing it would
--   rewrite the release owner's decision, and editing a run row would let a recorded BLOCK become a
--   PASS without anyone evaluating anything.

CREATE TABLE gate_runs (
    id                         TEXT PRIMARY KEY NOT NULL,
    build_id                   TEXT NOT NULL REFERENCES builds(id) ON DELETE CASCADE,
    baseline_build_id          TEXT REFERENCES builds(id) ON DELETE CASCADE,
    policy_sha256              TEXT NOT NULL,
    overall_effective_severity TEXT NOT NULL CHECK (overall_effective_severity IN ('PASS','REVIEW','BLOCK')),
    created_at                 TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    -- A release is never compared against itself: Core refuses that comparison, and the row that
    -- records its verdict says so too.
    CHECK (baseline_build_id IS NULL OR baseline_build_id <> build_id),
    CHECK (length(id) = 69 AND substr(id, 1, 5) = 'gate-'),
    CHECK (length(policy_sha256) = 64
           AND policy_sha256 = lower(policy_sha256)
           AND policy_sha256 NOT GLOB '*[^0-9a-f]*'),
    CHECK (created_at GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]T[0-9][0-9]:[0-9][0-9]:[0-9][0-9]Z')
);

CREATE INDEX idx_gate_runs_build ON gate_runs(build_id);
CREATE INDEX idx_gate_runs_created ON gate_runs(created_at);

CREATE TABLE gate_findings (
    run_id             TEXT NOT NULL REFERENCES gate_runs(id) ON DELETE CASCADE,
    id                 TEXT NOT NULL,
    rule_id            TEXT NOT NULL CHECK (rule_id IN (
        'git.clean',
        'release.commit_matches_expected',
        'release.version_matches_policy',
        'artifacts.required',
        'artifacts.hashes',
        'memory.flash_budget',
        'memory.ram_budget',
        'diff.growth',
        'release.notes',
        'evidence.unknown_review'
    )),
    state              TEXT NOT NULL CHECK (state IN ('PASS','REVIEW','BLOCK','UNKNOWN','N/A')),
    effective_severity TEXT NOT NULL CHECK (effective_severity IN ('PASS','REVIEW','BLOCK')),
    summary            TEXT NOT NULL,
    remediation        TEXT,
    -- The position the run recorded it at, which is canonical rule order. Stored so a re-read returns
    -- the same sequence instead of whatever order SQLite's query planner picked.
    ordinal            INTEGER NOT NULL CHECK (ordinal >= 0),
    PRIMARY KEY (run_id, id),
    UNIQUE (run_id, rule_id),
    UNIQUE (run_id, ordinal),
    CHECK (
        (state = 'PASS'    AND effective_severity = 'PASS')
        OR (state = 'REVIEW' AND effective_severity = 'REVIEW')
        OR (state = 'BLOCK'  AND effective_severity = 'BLOCK')
        -- N/A is neutral, and the severity vocabulary has exactly three members, so neutrality is the
        -- PASS member while the factual state keeps saying the rule did not apply.
        OR (state = 'N/A'     AND effective_severity = 'PASS')
        -- An evidence gap maps to review or block. Never to PASS.
        OR (state = 'UNKNOWN' AND effective_severity IN ('REVIEW','BLOCK'))
    )
);

CREATE INDEX idx_gate_findings_run ON gate_findings(run_id, ordinal);

CREATE TABLE gate_finding_evidence (
    run_id       TEXT NOT NULL,
    finding_id   TEXT NOT NULL,
    ordinal      INTEGER NOT NULL CHECK (ordinal >= 0),
    evidence_ref TEXT NOT NULL CHECK (length(evidence_ref) BETWEEN 1 AND 512
                                     -- A Windows separator is the one shape a host path always has,
                                     -- and no stable locator uses it.
                                     AND instr(evidence_ref, char(92)) = 0),
    PRIMARY KEY (run_id, finding_id, ordinal),
    FOREIGN KEY (run_id, finding_id) REFERENCES gate_findings(run_id, id) ON DELETE CASCADE
);

CREATE TABLE accepted_reviews (
    run_id         TEXT NOT NULL REFERENCES gate_runs(id) ON DELETE CASCADE,
    finding_id     TEXT NOT NULL,
    actor          TEXT NOT NULL CHECK (length(trim(actor)) > 0),
    accepted_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    reason         TEXT NOT NULL CHECK (length(trim(reason)) > 0),
    -- ADR-0023 requires the accepted record to state what it accepted, and only REVIEW is acceptable.
    original_state TEXT NOT NULL CHECK (original_state = 'REVIEW'),
    PRIMARY KEY (run_id, finding_id),
    FOREIGN KEY (run_id, finding_id) REFERENCES gate_findings(run_id, id) ON DELETE CASCADE,
    CHECK (accepted_at GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]T[0-9][0-9]:[0-9][0-9]:[0-9][0-9]Z')
);

CREATE INDEX idx_accepted_reviews_run ON accepted_reviews(run_id);

-- Immutability, stated in the schema rather than trusted to the caller.
CREATE TRIGGER gate_runs_are_immutable
BEFORE UPDATE ON gate_runs
BEGIN
    SELECT RAISE(ABORT, 'a stored Gate run is immutable; run the Gate again for a new verdict');
END;

CREATE TRIGGER gate_findings_are_immutable
BEFORE UPDATE ON gate_findings
BEGIN
    SELECT RAISE(ABORT, 'a stored Gate finding is immutable; accept the review or run the Gate again');
END;

CREATE TRIGGER accepted_reviews_cannot_be_edited
BEFORE UPDATE ON accepted_reviews
BEGIN
    SELECT RAISE(ABORT, 'a review acceptance is an audit record and cannot be edited');
END;

CREATE TRIGGER accepted_reviews_cannot_be_deleted
BEFORE DELETE ON accepted_reviews
BEGIN
    SELECT RAISE(ABORT, 'a review acceptance is an audit record and cannot be deleted');
END;
