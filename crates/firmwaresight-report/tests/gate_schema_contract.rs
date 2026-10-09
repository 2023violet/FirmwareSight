//! The contract tests for the two documents P3 publishes: `gate-results` v1 and `accepted-reviews` v1
//! (prompt §35, §36, §55).
//!
//! Both schemas were frozen before P3 and stay at major 1, so the interesting question is not whether
//! a fresh document validates but whether P3 can say the things it needs to *without* widening a
//! published contract. Everything beyond v1's closed property lists travels in `extensions`, and the
//! one addition to a property list — `original_state` on an acceptance — is optional, so a v1 record
//! written before P3 still validates (§35). Both directions are tested here.
//!
//! Checks run through the same `firmwaresight_report::schema_check` subset the diff contract uses,
//! rather than a second copy of it (§55).

use std::path::{Path, PathBuf};

use firmwaresight_core::domain::diff::{ByteChange, Comparability};
use firmwaresight_core::domain::gate::{
    EffectiveSeverity, FindingState, GateArtifactFact, GateBudgetFact, GateContext, GateEvaluation,
    GateGitFacts, GateGrowthFacts, GateMemoryFacts, GatePolicy, GateRuleId, GateUnknownEvidence,
    UnknownDisposition, UnknownPolicy,
};
use firmwaresight_core::domain::identity::{ArtifactKind, Fact};
use firmwaresight_report::gate::{
    ACCEPTANCE_ORIGINAL_STATE, ACCEPTED_REVIEWS_SCHEMA_ID, ACCEPTED_REVIEWS_SCHEMA_VERSION,
    AcceptanceDto, AcceptedReviewsDto, GATE_SCHEMA_ID, GATE_SCHEMA_VERSION, GateResultsDto,
};
use firmwaresight_report::gate_render;
use firmwaresight_report::schema_check::{self, Failure};
use serde_json::{Value, json};

const RUN_ID: &str = "gate-1f2e3d4c5b6a798867564534231200ffeeddccbbaa99887766554433221100ee";
const POLICY_SHA: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const SNAPSHOT: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const BASELINE: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const COMMIT: &str = "32b23aa0f1e2d3c4b5a69788796a5b4c3d2e1f00";
const ACCEPTED_AT: &str = "2026-09-30T07:14:52Z";

/// The locator schemes Core cites as evidence. Each names a fact about this build, so none of them can
/// become a path on the machine that ran the Gate.
const STABLE_LOCATOR_SCHEMES: [&str; 6] = [
    "artifact:",
    "diff:",
    "evidence:",
    "file:",
    "git:",
    "policy:",
];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|parent| parent.parent())
        .expect("crates/firmwaresight-report lives at <root>/crates/<name>")
        .to_path_buf()
}

fn schema(name: &str) -> Value {
    let path = repo_root().join("schemas").join(name);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} must be readable: {error}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("{name} must parse: {error}"))
}

fn gate_schema() -> Value {
    schema("gate-results.schema.json")
}

fn reviews_schema() -> Value {
    schema("accepted-reviews.schema.json")
}

/// A context whose findings cover all five states, so the document is exercised in every shape a real
/// run can take rather than in the one easiest to write.
fn mixed_context() -> GateContext {
    GateContext {
        snapshot_id: SNAPSHOT.to_owned(),
        artifacts: vec![GateArtifactFact {
            kind: ArtifactKind::Elf,
            sha256: Fact::known(POLICY_SHA.to_owned()),
            byte_size: 4096,
        }],
        attachments: Vec::new(),
        memory: Some(GateMemoryFacts {
            nonvolatile: Some(GateBudgetFact::exact(
                40_000,
                "every allocatable section attributed",
                "evidence:ev-memory-nonvolatile",
            )),
            runtime_ram: Some(GateBudgetFact::unknown("no RAM attribution available")),
        }),
        git: GateGitFacts {
            available: true,
            head_commit: Fact::known(COMMIT.to_owned()),
            exact_tag: Fact::unknown("HEAD carries no tag"),
            dirty: Fact::known(true),
        },
        version: None,
        release_notes: None,
        growth: GateGrowthFacts {
            baseline_snapshot_id: Some(BASELINE.to_owned()),
            nonvolatile: Some(ByteChange::between(
                Some(39_000),
                Some(40_000),
                Comparability::Exact,
            )),
            runtime_ram: None,
        },
        unknown_evidence: GateUnknownEvidence::default(),
        policy: GatePolicy {
            // flash_budget over an exact 40_000 total blocks; ram_budget has no figure, so its gap
            // maps to block by default; growth is over its review threshold; notes are not required.
            flash_budget: Some(10_000),
            ram_budget: Some(1_024),
            flash_growth_review_bytes: Some(500),
            require_release_notes: false,
            ..GatePolicy::default()
        },
    }
}

fn evaluation() -> GateEvaluation {
    mixed_context().evaluate(RUN_ID)
}

fn gate_document() -> Value {
    let dto = GateResultsDto::from_evaluation(&evaluation(), POLICY_SHA, 1);
    serde_json::to_value(&dto).expect("a DTO always serializes")
}

fn acceptance_dto(finding_id: &str) -> AcceptanceDto {
    AcceptanceDto {
        finding_id: finding_id.to_owned(),
        actor: "release owner".to_owned(),
        accepted_at: ACCEPTED_AT.to_owned(),
        reason: "the growth is this release's new CAN buffer".to_owned(),
        original_state: ACCEPTANCE_ORIGINAL_STATE,
    }
}

fn reviews_document(acceptances: &[AcceptanceDto]) -> Value {
    let dto = AcceptedReviewsDto::new(RUN_ID, acceptances, EffectiveSeverity::Pass);
    serde_json::to_value(&dto).expect("a DTO always serializes")
}

fn growth_finding_id() -> String {
    evaluation()
        .finding(GateRuleId::BaselineGrowth)
        .expect("every rule answers")
        .id
        .clone()
}

// --------------------------------------------------------------------------- gate-results v1

#[test]
fn a_gate_run_projects_to_a_document_that_validates_against_the_published_v1() {
    let doc = gate_document();
    let errors = schema_check::validate(&gate_schema(), &doc);
    assert!(errors.is_empty(), "{errors:?}\n{doc}");

    assert_eq!(doc["schema_version"], 1);
    assert_eq!(doc["run_id"], RUN_ID);
    assert_eq!(doc["snapshot_id"], SNAPSHOT);
    assert_eq!(doc["findings"].as_array().expect("array").len(), 10);
    assert_eq!(doc["extensions"]["policy_sha256"], POLICY_SHA);
    assert_eq!(doc["extensions"]["project_config_schema_version"], 1);
    assert_eq!(doc["extensions"]["baseline_snapshot_id"], BASELINE);
    assert_eq!(doc["extensions"]["overall_effective_severity"], "BLOCK");
}

#[test]
fn the_two_schema_identities_are_the_stable_urns_at_major_one() {
    assert_eq!(GATE_SCHEMA_ID, gate_schema()["$id"].as_str().expect("id"));
    assert_eq!(
        ACCEPTED_REVIEWS_SCHEMA_ID,
        reviews_schema()["$id"].as_str().expect("id")
    );
    assert_eq!(GATE_SCHEMA_VERSION, 1);
    assert_eq!(ACCEPTED_REVIEWS_SCHEMA_VERSION, 1);
    assert_eq!(gate_schema()["properties"]["schema_version"]["const"], 1);
}

#[test]
fn every_state_appears_with_the_severity_pairing_adr_0023_keeps_separate() {
    let doc = gate_document();
    let state_of = |rule: &str| {
        doc["findings"]
            .as_array()
            .expect("findings")
            .iter()
            .find(|finding| finding["rule_id"] == rule)
            .unwrap_or_else(|| panic!("no finding for {rule}"))
            .clone()
    };
    assert_eq!(state_of("git.clean")["state"], "BLOCK");
    assert_eq!(state_of("artifacts.required")["state"], "PASS");
    assert_eq!(state_of("memory.flash_budget")["state"], "BLOCK");
    assert_eq!(state_of("memory.ram_budget")["state"], "UNKNOWN");
    assert_eq!(state_of("diff.growth")["state"], "REVIEW");
    assert_eq!(state_of("release.notes")["state"], "N/A");

    // The two fields that must never collapse into one: an evidence gap keeps its own name and
    // escalates only through its severity.
    assert_eq!(state_of("memory.ram_budget")["effective_severity"], "BLOCK");
    assert_eq!(state_of("release.notes")["effective_severity"], "PASS");
    for finding in doc["findings"].as_array().expect("findings") {
        let state = finding["state"].as_str().expect("state");
        let severity = finding["effective_severity"].as_str().expect("severity");
        if state == "UNKNOWN" {
            assert_ne!(
                severity, "PASS",
                "{state} may never carry a PASS severity: {finding}"
            );
        }
    }
}

#[test]
fn no_host_path_reaches_the_gate_document() {
    let doc = gate_document();
    let text = doc.to_string();
    // A Windows separator is the one shape every host path has, and no stable locator uses it.
    assert!(
        !text.contains('\\'),
        "a host path escaped into the document: {text}"
    );
    assert!(
        !text.contains(":/") && !text.contains(":\\"),
        "a drive or directory locator escaped into the document: {text}"
    );
    // Every finding id is the run id plus the rule it answers: that is the only durable pointer.
    for finding in doc["findings"].as_array().expect("findings") {
        assert!(
            finding["id"]
                .as_str()
                .expect("id")
                .starts_with(&format!("{RUN_ID}#")),
            "{finding}"
        );
        for reference in finding["evidence_refs"].as_array().expect("refs") {
            let reference = reference.as_str().expect("ref");
            assert!(
                STABLE_LOCATOR_SCHEMES
                    .iter()
                    .any(|scheme| reference.starts_with(scheme)),
                "a stored locator must name a fact, not a file: {reference}"
            );
        }
    }
}

#[test]
fn rendering_the_same_run_twice_produces_identical_bytes() {
    let first = gate_render::render_json(&GateResultsDto::from_evaluation(
        &evaluation(),
        POLICY_SHA,
        1,
    ));
    let second = gate_render::render_json(&GateResultsDto::from_evaluation(
        &evaluation(),
        POLICY_SHA,
        1,
    ));
    assert_eq!(
        first, second,
        "a portable document must not depend on when it was written"
    );
    assert_eq!(
        first.matches('\n').count(),
        1,
        "exactly one compact document plus one newline: {first}"
    );
    assert!(first.ends_with("}\n"));
}

#[test]
fn the_gate_document_rejects_a_field_v1_never_declared() {
    let mut doc = gate_document();
    doc["artifact_path"] = json!("D:\\work\\out\\firmware.elf");
    let errors: Vec<Failure> = schema_check::validate(&gate_schema(), &doc);
    assert!(
        errors
            .iter()
            .any(|failure| failure.message.contains("not declared")),
        "an undeclared top-level property must be refused: {errors:?}"
    );

    let mut wrong_state = gate_document();
    wrong_state["findings"][0]["state"] = json!("MAYBE");
    assert!(
        !schema_check::validate(&gate_schema(), &wrong_state).is_empty(),
        "a state word outside the frozen five must be refused"
    );

    let mut wrong_major = gate_document();
    wrong_major["schema_version"] = json!(2);
    assert!(
        !schema_check::validate(&gate_schema(), &wrong_major).is_empty(),
        "v1 must not accept a document at major 2"
    );

    let mut missing = gate_document();
    missing.as_object_mut().expect("object").remove("findings");
    assert!(
        !schema_check::validate(&gate_schema(), &missing).is_empty(),
        "findings is required"
    );
}

// --------------------------------------------------------------------------- accepted-reviews v1

#[test]
fn a_record_written_before_p3_still_validates_without_original_state() {
    // §35's compatibility promise: the property is optional and the required list is unchanged, so a
    // v1 document that names nothing beyond the four required fields is still a valid audit record.
    let minimal = json!({
        "schema_version": 1,
        "run_id": RUN_ID,
        "acceptances": [{
            "finding_id": format!("{RUN_ID}#diff.growth"),
            "actor": "release owner",
            "accepted_at": ACCEPTED_AT,
            "reason": "intended growth"
        }]
    });
    let errors = schema_check::validate(&reviews_schema(), &minimal);
    assert!(errors.is_empty(), "{errors:?}");
    assert_eq!(
        reviews_schema()["properties"]["acceptances"]["items"]["required"],
        json!(["finding_id", "actor", "accepted_at", "reason"]),
        "the required list must not have grown"
    );
}

#[test]
fn every_p3_acceptance_names_the_state_it_accepted() {
    let dto = acceptance_dto(&growth_finding_id());
    assert_eq!(dto.original_state, "REVIEW");
    let doc = reviews_document(std::slice::from_ref(&dto));
    let errors = schema_check::validate(&reviews_schema(), &doc);
    assert!(errors.is_empty(), "{errors:?}\n{doc}");
    assert_eq!(doc["acceptances"][0]["original_state"], "REVIEW");
    assert_eq!(doc["extensions"]["overall_effective_severity"], "PASS");
}

#[test]
fn an_acceptance_may_only_ever_record_a_review_as_what_it_accepted() {
    for forged in ["PASS", "BLOCK", "UNKNOWN", "N/A", "REVIEWED"] {
        let doc = json!({
            "schema_version": 1,
            "run_id": RUN_ID,
            "acceptances": [{
                "finding_id": format!("{RUN_ID}#diff.growth"),
                "actor": "release owner",
                "accepted_at": ACCEPTED_AT,
                "reason": "intended growth",
                "original_state": forged
            }]
        });
        let errors = schema_check::validate(&reviews_schema(), &doc);
        if forged == "REVIEW" {
            assert!(errors.is_empty(), "REVIEW must validate: {errors:?}");
        } else {
            assert!(
                !errors.is_empty(),
                "an acceptance claiming {forged} must be refused by the contract"
            );
        }
    }
}

#[test]
fn the_reviews_document_refuses_a_blank_person_a_blank_reason_and_a_local_timestamp() {
    let cases = [
        ("an empty actor", "actor", json!("")),
        ("an empty reason", "reason", json!("")),
        (
            "a local-time accepted_at",
            "accepted_at",
            json!("2026-09-30T07:14:52+08:00"),
        ),
        ("a date without a time", "accepted_at", json!("2026-09-30")),
        (
            "a space where the time separator belongs",
            "accepted_at",
            json!("2026-09-30 07:14:52Z"),
        ),
    ];
    for (label, key, value) in cases {
        let mut doc = reviews_document(&[acceptance_dto(&growth_finding_id())]);
        doc["acceptances"][0][key] = value;
        assert!(
            !schema_check::validate(&reviews_schema(), &doc).is_empty(),
            "{label} must be refused"
        );
    }

    let mut extra = reviews_document(&[acceptance_dto(&growth_finding_id())]);
    extra["acceptances"][0]["accepted_by_approval"] = json!("manager");
    assert!(
        !schema_check::validate(&reviews_schema(), &extra).is_empty(),
        "additionalProperties stays false on an acceptance record"
    );
}

// --------------------------------------------------------------------------- human form

#[test]
fn the_human_report_orders_the_states_for_a_reader_and_shows_both_words_when_they_differ() {
    let evaluation = evaluation();
    let out = gate_render::render_human(&evaluation, POLICY_SHA);

    // The group headers, not the summary counts: a counts line happens to list the same five words,
    // so matching those would pass even if the findings themselves were in the wrong order.
    let mut search_from = 0usize;
    for header in [
        "\nBLOCK\n",
        "\nREVIEW\n",
        "\nUNKNOWN\n",
        "\nPASS\n",
        "\nN/A\n",
    ] {
        let rest = &out[search_from..];
        let at = rest.find(header).unwrap_or_else(|| {
            panic!("group header `{header}` missing after offset {search_from}:\n{out}")
        });
        search_from += at + header.len() - 1;
    }

    // §37: where the factual state and the effective severity are different words, print both.
    assert!(
        out.contains("memory.ram_budget  [BLOCK]"),
        "an UNKNOWN whose disposition blocks must not read as soft:\n{out}"
    );
    assert!(out.contains(&format!("Run       {RUN_ID}")));
    assert!(out.contains(&format!("Policy    sha256 {POLICY_SHA}")));
    assert!(out.contains("next:"));
    assert!(out.contains("refs: "));
    assert!(out.contains("accepting it with a reason"));
    for rule in GateRuleId::ALL {
        assert!(
            out.contains(&format!("  {}\n", rule.as_str()))
                || out.contains(&format!("  {}  [", rule.as_str())),
            "`{}` must appear under its group:\n{out}",
            rule.as_str()
        );
    }
}

#[test]
fn a_run_without_a_baseline_says_so_instead_of_naming_an_absent_one() {
    let context = GateContext {
        growth: GateGrowthFacts::without_baseline(),
        policy: GatePolicy {
            flash_growth_review_bytes: Some(500),
            ..mixed_context().policy
        },
        ..mixed_context()
    };
    let evaluation = context.evaluate(RUN_ID);
    assert_eq!(
        evaluation
            .finding(GateRuleId::BaselineGrowth)
            .expect("every rule answers")
            .state,
        FindingState::Unknown,
        "a threshold with nothing to compare against is an evidence gap"
    );
    let doc = serde_json::to_value(GateResultsDto::from_evaluation(&evaluation, POLICY_SHA, 1))
        .expect("serializes");
    assert!(schema_check::validate(&gate_schema(), &doc).is_empty());
    assert_eq!(doc["extensions"]["baseline_snapshot_id"], Value::Null);
    let out = gate_render::render_human(&evaluation, POLICY_SHA);
    assert!(out.contains("none (no comparison in this run)"), "{out}");
}

#[test]
fn an_unknown_disposition_of_block_never_projects_the_gap_away() {
    let context = GateContext {
        git: GateGitFacts::unavailable("git could not be consulted on this machine"),
        policy: GatePolicy {
            on_unknown: UnknownPolicy {
                git_clean: UnknownDisposition::Block,
                ..UnknownPolicy::default()
            },
            ..mixed_context().policy
        },
        ..mixed_context()
    };
    let evaluation = context.evaluate(RUN_ID);
    let git = evaluation
        .finding(GateRuleId::GitClean)
        .expect("every rule answers");
    assert_eq!(git.state, FindingState::Unknown);
    assert_eq!(git.effective_severity, EffectiveSeverity::Block);

    let doc = serde_json::to_value(GateResultsDto::from_evaluation(&evaluation, POLICY_SHA, 1))
        .expect("serializes");
    assert!(
        schema_check::validate(&gate_schema(), &doc).is_empty(),
        "an unknown-with-a-blocking-disposition is still a v1 document: {doc}"
    );
    let git_row = doc["findings"]
        .as_array()
        .expect("findings")
        .iter()
        .find(|finding| finding["rule_id"] == "git.clean")
        .expect("git.clean");
    assert_eq!(git_row["state"], "UNKNOWN", "the gap keeps its own name");
    assert_eq!(git_row["effective_severity"], "BLOCK");
    assert_eq!(doc["extensions"]["overall_effective_severity"], "BLOCK");
}
