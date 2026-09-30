//! The Gate history boundary: one immutable run, its findings and evidence refs, and the review
//! acceptances that dispose of a finding without changing it (prompt §32, §33, §34, §58).
//!
//! Every run stored here is a real `firmwaresight-core` evaluation of a hand-built `GateContext`,
//! because the record a release owner is accountable for has to be the record Core produces rather
//! than one that merely looks alike.
//!
//! Two of §32's items already have a home in `tests/storage.rs`: the newer-schema refusal and the
//! no-silent-reset that follows from it. Both are schema-version generic and stay green at version 3.

use std::path::{Path, PathBuf};

use firmwaresight_core::domain::diff::{ByteChange, Comparability};
use firmwaresight_core::domain::gate::{
    EffectiveSeverity, FindingState, GateArtifactFact, GateBudgetFact, GateContext, GateEvaluation,
    GateGitFacts, GateGrowthFacts, GateMemoryFacts, GatePolicy, GateRuleId, GateUnknownEvidence,
    UnknownDisposition, UnknownPolicy,
};
use firmwaresight_core::domain::identity::{ArtifactKind, Fact};
use firmwaresight_storage::{
    AcceptReviewError, Database, GateRunDraft, GateRunWrite, SCHEMA_VERSION, StorageError,
};
use rusqlite::params;

/// A 64-character lowercase hex fingerprint for one test label. A real one is a SHA-256 computed by
/// the project adapter; these are fixed-shape stand-ins, distinct per label, so the schema's digest
/// checks can be exercised by name rather than skipped.
fn hex64(label: &str) -> String {
    let mut digest = String::with_capacity(64);
    for byte in label.bytes() {
        digest.push_str(&format!("{byte:02x}"));
    }
    while digest.len() < 64 {
        digest.push('0');
    }
    digest.truncate(64);
    digest
}

/// A run id shaped exactly like the fingerprint the project adapter produces: `gate-` and 64 hex.
fn run_id(tag: &str) -> String {
    format!("gate-{}", hex64(tag))
}

fn sha(tag: &str) -> String {
    hex64(tag)
}

const TARGET_SNAPSHOT: &str = "0a1b2c3d4e5f60718293a4b5c6d7e8f90123456789abcdef0123456789abcdef";
const BASELINE_SNAPSHOT: &str = "9f8e7d6c5b4a39281706f5e4d3c2b1a09876543210fedcba9876543210fedcba";
const HEAD_COMMIT: &str = "32b23aa0f1e2d3c4b5a69788796a5b4c3d2e1f00";

/// `import_snapshot` names a build `build-<snapshot id>`, so the tests use the same rule and keep one
/// identity for a build instead of two that could drift.
static TARGET_BUILD: std::sync::LazyLock<String> =
    std::sync::LazyLock::new(|| format!("build-{TARGET_SNAPSHOT}"));
static BASELINE_BUILD: std::sync::LazyLock<String> =
    std::sync::LazyLock::new(|| format!("build-{BASELINE_SNAPSHOT}"));
static POLICY_SHA: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| sha("p1"));

/// The run that carries one finding in each of the five states, so a single stored record can test
/// every acceptance refusal: `git.clean` blocks on a dirty workspace, `artifacts.*` pass,
/// `memory.flash_budget` loses its evidence, `diff.growth` is over its review threshold, and the
/// rules that depend on unconfigured policy do not apply.
fn mixed_context() -> GateContext {
    GateContext {
        snapshot_id: TARGET_SNAPSHOT.to_owned(),
        artifacts: vec![GateArtifactFact {
            kind: ArtifactKind::Elf,
            sha256: Fact::known(sha("e1")),
            byte_size: 4096,
        }],
        memory: Some(GateMemoryFacts {
            nonvolatile: Some(GateBudgetFact::unknown(
                "the build carries no MAP companion",
            )),
            runtime_ram: None,
        }),
        git: GateGitFacts {
            available: true,
            head_commit: Fact::known(HEAD_COMMIT.to_owned()),
            exact_tag: Fact::unknown("HEAD carries no tag"),
            dirty: Fact::known(true),
        },
        version: None,
        release_notes: None,
        growth: GateGrowthFacts {
            baseline_snapshot_id: Some(BASELINE_SNAPSHOT.to_owned()),
            nonvolatile: Some(ByteChange::between(
                Some(1_000),
                Some(2_000),
                Comparability::Exact,
            )),
            runtime_ram: None,
        },
        unknown_evidence: GateUnknownEvidence::default(),
        policy: GatePolicy {
            flash_budget: Some(10_000),
            flash_growth_review_bytes: Some(500),
            require_release_notes: false,
            ..GatePolicy::default()
        },
    }
}

/// A run whose only obstacle is one reviewable finding: the workspace is clean, no budget is
/// configured, and the growth over the review threshold is the whole verdict.
fn review_context() -> GateContext {
    let mut context = mixed_context();
    context.git.dirty = Fact::known(false);
    context.policy.flash_budget = None;
    context
}

fn evaluate_run(context: &GateContext, id: &str) -> GateEvaluation {
    context.evaluate(id)
}

struct TempDb(PathBuf);

impl TempDb {
    fn new(label: &str) -> Self {
        let mut path = std::env::temp_dir();
        let unique = format!(
            "fwsight-p3-gate-{label}-{}-{:?}.sqlite",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        );
        path.push(unique);
        let _ = std::fs::remove_file(&path);
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDb {
    fn drop(&mut self) {
        for suffix in ["", "-wal", "-shm"] {
            let mut candidate = self.0.as_os_str().to_os_string();
            candidate.push(suffix);
            let _ = std::fs::remove_file(Path::new(&candidate));
        }
    }
}

/// A build row, because a Gate verdict is only reviewable later if the build it names is still there.
fn seed_build(db: &Database, build_id: &str, snapshot_id: &str) {
    let conn = db.connection();
    conn.execute(
        "INSERT OR IGNORE INTO projects (id, name) VALUES ('proj-gate', 'Gate project')",
        [],
    )
    .expect("project row");
    conn.execute(
        "INSERT INTO builds (id, project_id, snapshot_id, normalization_version, created_by_fwsight,
                             state)
         VALUES (?1, 'proj-gate', ?2, 'p0-normalize-1', '0.1.0', 'COMPLETE')",
        params![build_id, snapshot_id],
    )
    .expect("build row");
}

/// The two builds the mixed run names: the evaluated snapshot and its baseline.
fn seeded(label: &str) -> (TempDb, Database) {
    let file = TempDb::new(label);
    let db = Database::open(file.path()).expect("open");
    seed_build(&db, TARGET_BUILD.as_str(), TARGET_SNAPSHOT);
    seed_build(&db, BASELINE_BUILD.as_str(), BASELINE_SNAPSHOT);
    (file, db)
}

fn draft<'a>(id: &'a str, evaluation: &'a GateEvaluation) -> GateRunDraft<'a> {
    GateRunDraft {
        run_id: id,
        build_id: TARGET_BUILD.as_str(),
        baseline_build_id: Some(BASELINE_BUILD.as_str()),
        policy_sha256: POLICY_SHA.as_str(),
        evaluation,
    }
}

fn count(db: &Database, table: &str) -> i64 {
    db.connection()
        .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
        .expect("count")
}

/// `strftime`'s UTC shape, checked without a date library.
fn looks_utc(stamp: &str) -> bool {
    let bytes = stamp.as_bytes();
    bytes.len() == 20
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[10] == b'T'
        && bytes[13] == b':'
        && bytes[16] == b':'
        && bytes[19] == b'Z'
        && stamp
            .bytes()
            .all(|c| c.is_ascii_digit() || b"-T:Z".contains(&c))
}

/// The locator schemes Core is allowed to cite, and the two shapes a host path always has. A
/// project-relative path is legal — `file:docs/RELEASE_NOTES.md` names a file inside the project — so
/// what is refused is an absolute one, in either separator.
fn is_stable_locator(reference: &str) -> bool {
    [
        "artifact:",
        "diff:",
        "evidence:",
        "file:",
        "git:",
        "policy:",
    ]
    .iter()
    .any(|scheme| reference.starts_with(scheme))
        && !reference.contains('\\')
        && !reference.starts_with('/')
}

#[test]
fn a_fresh_database_carries_the_gate_tables_at_version_three() {
    let file = TempDb::new("fresh");
    let db = Database::open(file.path()).expect("open applies every migration");

    assert_eq!(SCHEMA_VERSION, 3, "P3 raises the schema to version 3");
    let version: i64 = db
        .connection()
        .query_row("SELECT MAX(version) FROM schema_migrations", [], |r| {
            r.get(0)
        })
        .expect("version");
    assert_eq!(version, SCHEMA_VERSION);

    for table in [
        "gate_runs",
        "gate_findings",
        "gate_finding_evidence",
        "accepted_reviews",
    ] {
        assert_eq!(count(&db, table), 0, "{table} exists and starts empty");
    }
}

#[test]
fn a_version_two_database_gains_the_gate_tables_and_keeps_its_history() {
    // Migration 0003 is additive, so the proof is that a v2 file keeps every row it holds. Built by
    // hand at version 2 with one build, then opened.
    let file = TempDb::new("upgrade-0003");
    {
        let conn = rusqlite::Connection::open(file.path()).expect("raw create");
        conn.execute_batch(include_str!("../migrations/0001_initial.sql"))
            .expect("version 1 schema");
        conn.execute_batch(include_str!(
            "../migrations/0002_evidence_keyed_by_build.sql"
        ))
        .expect("version 2 schema");
        conn.execute_batch(
            "CREATE TABLE schema_migrations (
                 version    INTEGER PRIMARY KEY NOT NULL,
                 name       TEXT NOT NULL,
                 applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
             );
             INSERT INTO schema_migrations (version, name) VALUES (1, '0001_initial');
             INSERT INTO schema_migrations (version, name) VALUES (2, '0002_evidence_keyed_by_build');
             INSERT INTO projects (id, name) VALUES ('proj-1', 'P0 project');
             INSERT INTO builds (id, project_id, snapshot_id, normalization_version,
                                 created_by_fwsight, state)
                  VALUES ('build-old', 'proj-1', 'snap-old', 'p0-normalize-1', '0.1.0', 'COMPLETE');",
        )
        .expect("version 2 contents");
    }

    let db = Database::open(file.path()).expect("open must upgrade v2 in place");

    let recorded: i64 = db
        .connection()
        .query_row("SELECT COUNT(*) FROM schema_migrations", [], |r| r.get(0))
        .expect("count");
    assert_eq!(recorded, 3, "all three migrations are recorded, once each");

    let kept: String = db
        .connection()
        .query_row(
            "SELECT snapshot_id FROM builds WHERE id = 'build-old'",
            [],
            |r| r.get(0),
        )
        .expect("the pre-upgrade build survives");
    assert_eq!(kept, "snap-old", "an upgrade must not rewrite history");

    assert_eq!(
        count(&db, "gate_runs"),
        0,
        "and the Gate tables arrive empty"
    );
}

#[test]
fn a_failed_gate_migration_leaves_the_previous_schema_intact_and_recoverable() {
    // The obstruction is a real one: something already holds the name 0003 wants. The migration must
    // fail as a typed version-3 error, roll back, and change nothing the user had.
    let file = TempDb::new("failed-0003");
    {
        let conn = rusqlite::Connection::open(file.path()).expect("raw create");
        conn.execute_batch(include_str!("../migrations/0001_initial.sql"))
            .expect("version 1 schema");
        conn.execute_batch(include_str!(
            "../migrations/0002_evidence_keyed_by_build.sql"
        ))
        .expect("version 2 schema");
        conn.execute_batch(
            "CREATE TABLE schema_migrations (
                 version    INTEGER PRIMARY KEY NOT NULL,
                 name       TEXT NOT NULL
             );
             INSERT INTO schema_migrations (version, name) VALUES (1, '0001_initial');
             INSERT INTO schema_migrations (version, name) VALUES (2, '0002_evidence_keyed_by_build');
             CREATE TABLE gate_runs (id TEXT PRIMARY KEY);
             INSERT INTO projects (id, name) VALUES ('proj-1', 'P0 project');
             INSERT INTO builds (id, project_id, snapshot_id, normalization_version,
                                 created_by_fwsight, state)
                  VALUES ('build-old', 'proj-1', 'snap-old', 'p0-normalize-1', '0.1.0', 'COMPLETE');",
        )
        .expect("a v2 database whose next migration is obstructed");
    }

    let err = Database::open(file.path())
        .err()
        .expect("an obstructed migration must fail, not be skipped");
    assert!(
        matches!(err, StorageError::Migration { version: 3, .. }),
        "the failure must name version 3, got {err:?}"
    );

    let conn = rusqlite::Connection::open(file.path()).expect("raw reopen");
    let version: i64 = conn
        .query_row("SELECT MAX(version) FROM schema_migrations", [], |r| {
            r.get(0)
        })
        .expect("version");
    assert_eq!(version, 2, "a rolled-back migration records nothing");
    let builds: i64 = conn
        .query_row("SELECT COUNT(*) FROM builds", [], |r| r.get(0))
        .expect("count");
    assert_eq!(
        builds, 1,
        "the failure must not have touched stored history"
    );
    drop(conn);

    // Clear the obstruction: the same file then upgrades normally.
    let conn = rusqlite::Connection::open(file.path()).expect("raw reopen");
    conn.execute("DROP TABLE gate_runs", [])
        .expect("drop the obstructing table");
    drop(conn);
    let db = Database::open(file.path()).expect("the database is recoverable");
    let version: i64 = db
        .connection()
        .query_row("SELECT MAX(version) FROM schema_migrations", [], |r| {
            r.get(0)
        })
        .expect("version");
    assert_eq!(version, SCHEMA_VERSION);
}

#[test]
fn a_run_its_findings_and_its_refs_commit_together_or_not_at_all() {
    let (_file, mut db) = seeded("atomic");
    let id = run_id("a1");
    let mut evaluation = evaluate_run(&mixed_context(), &id);
    // One host-path locator anywhere in the run must cost the whole run, not just its tail.
    let host_path = "C:\\Users\\release\\out\\firmware.elf".to_owned();
    evaluation.findings[2].evidence_refs.push(host_path);

    let err = db
        .persist_gate_run(&draft(&id, &evaluation))
        .expect_err("a host path is not a stable locator");
    assert!(matches!(err, StorageError::Write { .. }), "got {err:?}");

    assert_eq!(count(&db, "gate_runs"), 0, "no run row survives");
    assert_eq!(count(&db, "gate_findings"), 0, "no findings survive");
    assert_eq!(count(&db, "gate_finding_evidence"), 0, "no refs survive");
    assert_eq!(
        db.gate_run_by_id(&id).expect("query"),
        None,
        "a half-run is not readable"
    );

    let clean = evaluate_run(&mixed_context(), &id);
    db.persist_gate_run(&draft(&id, &clean))
        .expect("the same database still accepts a clean run");
}

#[test]
fn a_stored_run_reads_back_every_finding_and_ref_in_the_order_it_recorded_them() {
    let (_file, mut db) = seeded("readback");
    let id = run_id("b1");
    let evaluation = evaluate_run(&mixed_context(), &id);
    assert_eq!(
        db.persist_gate_run(&draft(&id, &evaluation))
            .expect("persist"),
        GateRunWrite::Inserted
    );

    let stored = db
        .gate_run_by_id(&id)
        .expect("query")
        .expect("the run is there");
    assert_eq!(stored.run_id, id);
    assert_eq!(stored.build_id, *TARGET_BUILD);
    assert_eq!(
        stored.baseline_build_id.as_deref(),
        Some(BASELINE_BUILD.as_str())
    );
    assert_eq!(stored.policy_sha256, *POLICY_SHA);
    assert_eq!(
        stored.overall_effective_severity,
        evaluation.overall_effective_severity
    );
    assert!(
        looks_utc(&stored.created_at),
        "created_at is SQLite's UTC insertion time: {:?}",
        stored.created_at
    );

    let rules: Vec<&str> = stored.findings.iter().map(|f| f.rule_id.as_str()).collect();
    assert_eq!(
        rules,
        GateRuleId::ALL.map(GateRuleId::as_str),
        "canonical rule order, not the query planner's order"
    );
    assert_eq!(
        stored.findings[0].rule(),
        Some(GateRuleId::GitClean),
        "a stored rule id parses back to the identity Core named"
    );

    for (stored, fresh) in stored.findings.iter().zip(&evaluation.findings) {
        assert_eq!(stored.id, fresh.id);
        assert_eq!(stored.state, fresh.state);
        assert_eq!(stored.effective_severity, fresh.effective_severity);
        assert_eq!(stored.summary, fresh.summary);
        assert_eq!(stored.remediation, fresh.remediation);
        assert_eq!(
            stored.evidence_refs, fresh.evidence_refs,
            "every locator the rule cited comes back, in order"
        );
        for reference in &stored.evidence_refs {
            assert!(
                is_stable_locator(reference),
                "a stored locator is a scheme-prefixed fact, never an absolute path: {reference}"
            );
        }
    }
}

#[test]
fn the_five_states_are_all_present_and_each_keeps_its_severity_pairing() {
    let (_file, mut db) = seeded("five-states");
    let id = run_id("c1");
    let evaluation = evaluate_run(&mixed_context(), &id);
    db.persist_gate_run(&draft(&id, &evaluation))
        .expect("persist");
    let stored = db
        .gate_run_by_id(&id)
        .expect("query")
        .expect("the run is there");

    let state_of = |rule: GateRuleId| {
        stored
            .findings
            .iter()
            .find(|f| f.rule_id == rule.as_str())
            .unwrap_or_else(|| panic!("no stored finding for {rule}"))
            .state
    };
    assert_eq!(state_of(GateRuleId::GitClean), FindingState::Block);
    assert_eq!(state_of(GateRuleId::RequiredArtifacts), FindingState::Pass);
    assert_eq!(state_of(GateRuleId::FlashBudget), FindingState::Unknown);
    assert_eq!(state_of(GateRuleId::BaselineGrowth), FindingState::Review);
    assert_eq!(
        state_of(GateRuleId::VersionMatchesPolicy),
        FindingState::NotApplicable
    );

    for finding in &stored.findings {
        match finding.state {
            // An evidence gap keeps its own name and escalates only through its severity.
            FindingState::Unknown => {
                assert_ne!(finding.effective_severity, EffectiveSeverity::Pass)
            }
            FindingState::Pass | FindingState::NotApplicable => {
                assert_eq!(finding.effective_severity, EffectiveSeverity::Pass)
            }
            FindingState::Review => {
                assert_eq!(finding.effective_severity, EffectiveSeverity::Review)
            }
            FindingState::Block => {
                assert_eq!(finding.effective_severity, EffectiveSeverity::Block)
            }
        }
    }
}

#[test]
fn storing_the_same_run_again_is_a_dedupe_not_a_duplicate() {
    let (_file, mut db) = seeded("dedupe");
    let id = run_id("d1");
    let evaluation = evaluate_run(&mixed_context(), &id);
    db.persist_gate_run(&draft(&id, &evaluation))
        .expect("first persist");

    assert_eq!(
        db.persist_gate_run(&draft(&id, &evaluation))
            .expect("second persist"),
        GateRunWrite::AlreadyStored,
        "the same content under the same fingerprint is one run"
    );
    assert_eq!(count(&db, "gate_runs"), 1);
    assert_eq!(
        count(&db, "gate_findings"),
        GateRuleId::ALL.len() as i64,
        "a re-run never doubles the findings"
    );
}

#[test]
fn the_same_run_id_hiding_a_different_verdict_is_an_invariant_not_an_update() {
    let (_file, mut db) = seeded("collision");
    let id = run_id("e1");
    let evaluation = evaluate_run(&mixed_context(), &id);
    db.persist_gate_run(&draft(&id, &evaluation))
        .expect("persist");

    // A fingerprint collision with different semantics: same id, different summary on one finding.
    let mut tampered = evaluation.clone();
    tampered.findings[0].summary = "Workspace was clean.".to_owned();
    let err = db
        .persist_gate_run(&draft(&id, &tampered))
        .expect_err("a different verdict under one id must not pass");
    assert!(matches!(err, StorageError::Invariant { .. }), "got {err:?}");

    let stored = db
        .gate_run_by_id(&id)
        .expect("query")
        .expect("the run is there");
    assert_eq!(
        stored.findings[0].summary, evaluation.findings[0].summary,
        "the refused write changed nothing"
    );
}

#[test]
fn a_policy_or_workspace_change_arrives_as_a_new_run_rather_than_a_rewrite() {
    let (_file, mut db) = seeded("two-runs");
    let first_id = run_id("f1");
    let second_id = run_id("f2");
    let dirty = evaluate_run(&mixed_context(), &first_id);
    let clean = evaluate_run(&review_context(), &second_id);

    db.persist_gate_run(&draft(&first_id, &dirty))
        .expect("first run");
    db.persist_gate_run(&draft(&second_id, &clean))
        .expect("a second verdict on the same build is a second run");

    assert_eq!(count(&db, "gate_runs"), 2);
    assert_eq!(
        count(&db, "gate_findings"),
        GateRuleId::ALL.len() as i64 * 2
    );
    assert_eq!(
        db.gate_run_by_id(&first_id)
            .expect("query")
            .expect("first run")
            .overall_effective_severity,
        dirty.overall_effective_severity,
        "the earlier record is untouched"
    );
}

#[test]
fn a_stored_verdict_cannot_be_rewritten_by_sql() {
    // The immutability a release owner depends on is a trigger, so it holds for a hand-written
    // statement and not only for the API.
    let (_file, mut db) = seeded("immutable");
    let id = run_id("g1");
    let evaluation = evaluate_run(&mixed_context(), &id);
    db.persist_gate_run(&draft(&id, &evaluation))
        .expect("persist");

    let flipped = db.connection().execute(
        "UPDATE gate_runs SET overall_effective_severity = 'PASS' WHERE id = ?1",
        params![&id],
    );
    assert!(
        flipped.is_err(),
        "a BLOCK cannot be edited into a PASS: {flipped:?}"
    );

    let finding_id = evaluation.findings[0].id.clone();
    let softened = db.connection().execute(
        "UPDATE gate_findings SET state = 'PASS', effective_severity = 'PASS'
          WHERE run_id = ?1 AND id = ?2",
        params![&id, &finding_id],
    );
    assert!(
        softened.is_err(),
        "a finding cannot be edited: {softened:?}"
    );

    let still = db
        .gate_run_by_id(&id)
        .expect("query")
        .expect("the run is there");
    assert_eq!(still.findings[0].state, FindingState::Block);
}

#[test]
fn the_schema_refuses_a_state_pairing_that_adr_0023_forbids() {
    let (_file, mut db) = seeded("pairing");
    let id = run_id("h1");
    let evaluation = evaluate_run(&mixed_context(), &id);
    db.persist_gate_run(&draft(&id, &evaluation))
        .expect("persist");

    let err = db.connection().execute(
        "INSERT INTO gate_findings (run_id, id, rule_id, state, effective_severity, summary, ordinal)
         VALUES (?1, ?2, 'release.notes', 'UNKNOWN', 'PASS', 'An evidence gap read as a pass.', 99)",
        params![&id, &format!("{id}#release.notes")],
    );
    assert!(
        err.is_err(),
        "UNKNOWN with a PASS severity must be refused by the CHECK: {err:?}"
    );

    let self_baseline = db.connection().execute(
        "INSERT INTO gate_runs (id, build_id, baseline_build_id, policy_sha256,
                                overall_effective_severity)
         VALUES (?1, ?2, ?2, ?3, 'PASS')",
        params![run_id("h2"), TARGET_BUILD.as_str(), sha("h2")],
    );
    assert!(
        self_baseline.is_err(),
        "a release cannot be its own baseline: {self_baseline:?}"
    );
}

#[test]
fn a_host_path_can_never_be_stored_as_an_evidence_locator() {
    let (_file, mut db) = seeded("host-path");
    let id = run_id("i1");
    let evaluation = evaluate_run(&mixed_context(), &id);
    db.persist_gate_run(&draft(&id, &evaluation))
        .expect("persist");

    let finding = &evaluation.findings[0];
    for candidate in [
        "C:\\Users\\release\\out\\firmware.elf",
        "\\\\nas\\share\\RELEASE_NOTES.md",
        "release\\out\\firmware.elf",
    ] {
        let stored = db.connection().execute(
            "INSERT INTO gate_finding_evidence (run_id, finding_id, ordinal, evidence_ref)
             VALUES (?1, ?2, 500, ?3)",
            params![&id, &finding.id, candidate],
        );
        assert!(
            stored.is_err(),
            "`{candidate}` names a machine, so the CHECK must refuse it"
        );
    }

    let stored = db
        .gate_run_by_id(&id)
        .expect("query")
        .expect("the run is there");
    for finding in &stored.findings {
        for reference in &finding.evidence_refs {
            assert!(
                is_stable_locator(reference),
                "a stored locator is a scheme-prefixed fact, never an absolute path: {reference}"
            );
        }
    }
}

#[test]
fn accepting_a_review_keeps_the_finding_at_review_and_survives_a_reopen() {
    let file = TempDb::new("accept");
    let id = run_id("j1");

    let acceptance = {
        let mut db = Database::open(file.path()).expect("open");
        seed_build(&db, TARGET_BUILD.as_str(), TARGET_SNAPSHOT);
        seed_build(&db, BASELINE_BUILD.as_str(), BASELINE_SNAPSHOT);
        let evaluation = evaluate_run(&mixed_context(), &id);
        db.persist_gate_run(&draft(&id, &evaluation))
            .expect("persist");

        let review = evaluation
            .finding(GateRuleId::BaselineGrowth)
            .expect("the growth finding");
        let acceptance = db
            .accept_review(
                &id,
                &review.id,
                "release owner",
                "the 1000 B growth is the new CAN buffer, intended for this release",
            )
            .expect("accept the review");
        assert_eq!(acceptance.original_state, FindingState::Review);
        assert_eq!(acceptance.actor, "release owner");
        assert_eq!(acceptance.finding_id, review.id);
        assert!(
            looks_utc(&acceptance.accepted_at),
            "accepted_at is SQLite's UTC insertion time: {:?}",
            acceptance.accepted_at
        );

        let after = db
            .gate_run_by_id(&id)
            .expect("query")
            .expect("the run is there");
        let growth = after
            .findings
            .iter()
            .find(|f| f.rule_id == GateRuleId::BaselineGrowth.as_str())
            .expect("the growth finding");
        assert_eq!(
            growth.state,
            FindingState::Review,
            "accepting a review never rewrites the finding"
        );
        assert_eq!(growth.effective_severity, EffectiveSeverity::Review);
        assert_eq!(growth.summary, review.summary);
        assert_eq!(
            after.acceptable_findings().len(),
            1,
            "only the REVIEW finding is offered for acceptance"
        );

        db.accept_review(&id, &review.id, "someone else", "a second opinion")
            .expect_err("one finding is accepted once, and never overwritten");
        acceptance
    };

    let db = Database::open(file.path()).expect("reopen");
    let trail = db.accepted_reviews_for_run(&id).expect("read the trail");
    assert_eq!(trail.len(), 1, "the audit row survives a reopen");
    assert_eq!(trail[0], acceptance);
    assert_eq!(
        db.gate_run_by_id(&id)
            .expect("query")
            .expect("the run is there")
            .findings
            .len(),
        GateRuleId::ALL.len()
    );
}

#[test]
fn an_acceptance_must_name_a_person_and_a_reason() {
    let (_file, mut db) = seeded("required-fields");
    let id = run_id("k1");
    let evaluation = evaluate_run(&mixed_context(), &id);
    db.persist_gate_run(&draft(&id, &evaluation))
        .expect("persist");
    let review_id = evaluation
        .finding(GateRuleId::BaselineGrowth)
        .expect("the growth finding")
        .id
        .clone();

    for (actor, reason) in [("   ", "a reason"), ("release owner", "   ")] {
        let err = db
            .accept_review(&id, &review_id, actor, reason)
            .expect_err("an acceptance without an actor or a reason is not an audit record");
        assert_eq!(err.stable_code(), "ERR-STORAGE-4010", "got {err:?}");
        assert!(
            matches!(
                err,
                AcceptReviewError::EmptyActor | AcceptReviewError::EmptyReason
            ),
            "got {err:?}"
        );
    }
    assert_eq!(
        count(&db, "accepted_reviews"),
        0,
        "a refused acceptance wrote nothing"
    );

    let accepted = db
        .accept_review(&id, &review_id, "  release owner  ", "  intended growth  ")
        .expect("a padded actor and reason still count, and are stored trimmed");
    assert_eq!(accepted.actor, "release owner");
    assert_eq!(accepted.reason, "intended growth");
}

#[test]
fn only_a_review_can_be_accepted() {
    let (_file, mut db) = seeded("only-review");
    let id = run_id("l1");
    let evaluation = evaluate_run(&mixed_context(), &id);
    db.persist_gate_run(&draft(&id, &evaluation))
        .expect("persist");

    for rule in GateRuleId::ALL {
        let finding = evaluation.finding(rule).expect("every rule answers");
        let result = db.accept_review(&id, &finding.id, "release owner", "because I said so");
        if finding.state == FindingState::Review {
            assert!(result.is_ok(), "a REVIEW finding must be acceptable");
            continue;
        }
        let Err(err) = result else {
            panic!(
                "{} is {}/{} and must not be acceptable",
                rule.as_str(),
                finding.state,
                finding.effective_severity
            );
        };
        assert_eq!(err.stable_code(), "ERR-STORAGE-4008");
        assert!(matches!(err, AcceptReviewError::NotReview { .. }));
    }
}

#[test]
fn a_second_acceptance_names_the_person_who_answered_first() {
    let (_file, mut db) = seeded("second");
    let id = run_id("m1");
    let evaluation = evaluate_run(&mixed_context(), &id);
    db.persist_gate_run(&draft(&id, &evaluation))
        .expect("persist");
    let review = evaluation
        .finding(GateRuleId::BaselineGrowth)
        .expect("the growth finding");

    db.accept_review(&id, &review.id, "ada", "intended growth")
        .expect("first acceptance");
    let err = db
        .accept_review(&id, &review.id, "linus", "a different reason")
        .expect_err("one finding is accepted once, and never overwritten");
    assert_eq!(err.stable_code(), "ERR-STORAGE-4009");
    assert!(
        matches!(&err, AcceptReviewError::AlreadyAccepted { actor, .. } if actor == "ada"),
        "the refusal has to say who already answered: {err:?}"
    );

    let trail = db.accepted_reviews_for_run(&id).expect("read the trail");
    assert_eq!(trail.len(), 1, "the first record was not replaced");
    assert_eq!(trail[0].reason, "intended growth");
}

#[test]
fn accepting_a_finding_that_belongs_to_no_stored_run_is_refused() {
    let (_file, mut db) = seeded("missing");
    let id = run_id("n1");
    let evaluation = evaluate_run(&mixed_context(), &id);
    db.persist_gate_run(&draft(&id, &evaluation))
        .expect("persist");

    let err = db
        .accept_review(&id, &format!("{id}#not.a.rule"), "ada", "a reason")
        .expect_err("an invented finding id cannot be accepted");
    assert_eq!(err.stable_code(), "ERR-STORAGE-4007");

    // The same refusal for a run that does not exist at all: nothing is created along the way.
    let ghost = run_id("zz");
    let err = db
        .accept_review(&ghost, &format!("{ghost}#git.clean"), "ada", "a reason")
        .expect_err("an invented run cannot be accepted into");
    assert_eq!(err.stable_code(), "ERR-STORAGE-4007");
    assert_eq!(count(&db, "gate_runs"), 1);
    assert_eq!(count(&db, "accepted_reviews"), 0);
}

#[test]
fn an_acceptance_is_an_audit_record_that_neither_edits_nor_disappears() {
    let (_file, mut db) = seeded("audit");
    let id = run_id("o1");
    let evaluation = evaluate_run(&mixed_context(), &id);
    db.persist_gate_run(&draft(&id, &evaluation))
        .expect("persist");
    let review = evaluation
        .finding(GateRuleId::BaselineGrowth)
        .expect("the growth finding");
    db.accept_review(&id, &review.id, "ada", "intended growth")
        .expect("accept");

    let edited = db.connection().execute(
        "UPDATE accepted_reviews SET reason = 'no growth at all' WHERE run_id = ?1",
        params![&id],
    );
    assert!(
        edited.is_err(),
        "an audit record cannot be edited: {edited:?}"
    );

    let deleted = db.connection().execute(
        "DELETE FROM accepted_reviews WHERE run_id = ?1",
        params![&id],
    );
    assert!(
        deleted.is_err(),
        "an audit record cannot be deleted: {deleted:?}"
    );

    assert_eq!(count(&db, "accepted_reviews"), 1);
}

#[test]
fn accepting_every_review_moves_only_the_aggregate() {
    let (_file, mut db) = seeded("aggregate");
    let id = run_id("p1");
    let evaluation = evaluate_run(&review_context(), &id);
    assert_eq!(
        evaluation.overall_effective_severity,
        EffectiveSeverity::Review,
        "the run's one obstacle is the growth review"
    );
    db.persist_gate_run(&draft(&id, &evaluation))
        .expect("persist");

    let stored = db
        .gate_run_by_id(&id)
        .expect("query")
        .expect("the run is there");
    let accepted: Vec<String> = stored
        .acceptable_findings()
        .iter()
        .map(|f| f.id.clone())
        .collect();
    assert_eq!(accepted.len(), 1);
    for finding_id in &accepted {
        db.accept_review(
            &id,
            finding_id,
            "ada",
            "the growth is this release's new buffer",
        )
        .expect("accept");
    }

    let trail = db
        .accepted_reviews_for_run(&id)
        .expect("read the trail")
        .iter()
        .map(|a| a.finding_id.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        evaluation.aggregate_with_acceptances(&trail),
        EffectiveSeverity::Pass,
        "the owner's decision is visible in the aggregate"
    );

    let reopened = db
        .gate_run_by_id(&id)
        .expect("query")
        .expect("the run is there")
        .findings;
    assert_eq!(
        reopened
            .iter()
            .filter(|f| f.state == FindingState::Review)
            .count(),
        accepted.len(),
        "every accepted review is still stored as a REVIEW"
    );
    assert_eq!(
        reopened
            .iter()
            .find(|f| f.rule_id == GateRuleId::BaselineGrowth.as_str())
            .expect("the growth finding")
            .state,
        FindingState::Review,
        "and the record itself still says what was evaluated"
    );
}

#[test]
fn an_unknown_disposition_of_block_is_stored_as_unknown_with_a_blocking_severity() {
    // §34's boundary: `on_unknown = block` escalates the aggregate without turning the factual state
    // into BLOCK, and an evidence gap is still not something a person may accept away.
    let (_file, mut db) = seeded("disposition");
    let id = run_id("q1");
    let base = mixed_context();
    let context = GateContext {
        git: GateGitFacts::unavailable("git could not be consulted"),
        policy: GatePolicy {
            on_unknown: UnknownPolicy {
                git_clean: UnknownDisposition::Block,
                ..UnknownPolicy::default()
            },
            ..base.policy
        },
        ..base
    };
    let evaluation = evaluate_run(&context, &id);
    db.persist_gate_run(&draft(&id, &evaluation))
        .expect("persist");

    let stored = db
        .gate_run_by_id(&id)
        .expect("query")
        .expect("the run is there");
    let git = stored
        .findings
        .iter()
        .find(|f| f.rule_id == GateRuleId::GitClean.as_str())
        .expect("git.clean");
    assert_eq!(git.state, FindingState::Unknown, "the gap stays an unknown");
    assert_eq!(git.effective_severity, EffectiveSeverity::Block);

    let err = db
        .accept_review(&id, &git.id, "ada", "close it anyway")
        .expect_err("an evidence gap is not a review");
    assert_eq!(err.stable_code(), "ERR-STORAGE-4008");
}

#[test]
fn a_run_is_only_storable_against_a_build_that_exists() {
    let file = TempDb::new("no-build");
    let mut db = Database::open(file.path()).expect("open");
    let id = run_id("r1");
    let evaluation = evaluate_run(&mixed_context(), &id);

    let err = db
        .persist_gate_run(&draft(&id, &evaluation))
        .expect_err("a verdict about no stored build cannot be reviewed later");
    assert!(matches!(err, StorageError::Write { .. }), "got {err:?}");
    assert_eq!(count(&db, "gate_runs"), 0, "and nothing half-arrived");

    seed_build(&db, TARGET_BUILD.as_str(), TARGET_SNAPSHOT);
    seed_build(&db, BASELINE_BUILD.as_str(), BASELINE_SNAPSHOT);
    db.persist_gate_run(&draft(&id, &evaluation))
        .expect("the same run stores once its build does");
}

#[test]
fn a_run_id_that_is_not_a_fingerprint_is_refused() {
    let (_file, mut db) = seeded("bad-id");
    let id = "gate-0".to_owned();
    let evaluation = evaluate_run(&mixed_context(), &id);

    let err = db
        .persist_gate_run(&draft(&id, &evaluation))
        .expect_err("a run id is a content fingerprint, not any old string");
    assert!(matches!(err, StorageError::Write { .. }), "got {err:?}");
    assert_eq!(count(&db, "gate_runs"), 0);

    // The draft must also agree with the evaluation it carries.
    let good = run_id("s1");
    let evaluation = evaluate_run(&mixed_context(), &good);
    let other = run_id("s2");
    let other_policy = sha("s2");
    let mismatched = GateRunDraft {
        run_id: &other,
        build_id: TARGET_BUILD.as_str(),
        baseline_build_id: None,
        policy_sha256: &other_policy,
        evaluation: &evaluation,
    };
    let err = db
        .persist_gate_run(&mismatched)
        .expect_err("one id cannot stand in for another");
    assert!(matches!(err, StorageError::Invariant { .. }), "got {err:?}");
    assert_eq!(count(&db, "gate_runs"), 0, "and the refusal wrote nothing");
}
