//! The contract test for `schemas/project-config.schema.json` (prompt §55): the published shape of
//! `firmwaresight.toml` and the loader that reads it must agree, and where they deliberately do not,
//! the difference has to be a named fact rather than an accident.
//!
//! Checks run through `firmwaresight_report::schema_check`, the same dependency-free subset the diff
//! and Gate contracts use, rather than a second copy of it (§55).

use std::path::{Path, PathBuf};

use firmwaresight_core::domain::gate::GatePolicy;
use firmwaresight_project::config::{LoadedProject, SUPPORTED_SCHEMA_VERSION, new_config};
use firmwaresight_report::schema_check::{self, Failure};
use serde_json::{Value, json};

const CONFIG_URN: &str = "urn:firmwaresight:schema:project-config:1";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|parent| parent.parent())
        .expect("crates/firmwaresight-project lives at <root>/crates/<name>")
        .to_path_buf()
}

fn schema() -> Value {
    let path = repo_root()
        .join("schemas")
        .join("project-config.schema.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} must be readable: {error}", path.display()));
    serde_json::from_str(&text)
        .unwrap_or_else(|error| panic!("project-config schema must parse: {error}"))
}

fn errors(doc: &Value) -> Vec<Failure> {
    schema_check::validate(&schema(), doc)
}

fn parse(text: &str) -> Result<LoadedProject, firmwaresight_project::error::ProjectError> {
    LoadedProject::parse(&repo_root().join("firmwaresight.toml"), text)
}

fn minimal_json() -> Value {
    json!({
        "schema_version": 1,
        "project": { "name": "brake-node" },
        "artifacts": { "required": ["elf"] }
    })
}

fn example_config() -> Value {
    let path = repo_root().join("examples").join("firmwaresight.toml");
    let loaded =
        parse_at(&path).unwrap_or_else(|error| panic!("{} must load: {error}", path.display()));
    serde_json::to_value(&loaded.config).expect("a config always serializes")
}

fn parse_at(path: &Path) -> Result<LoadedProject, firmwaresight_project::error::ProjectError> {
    LoadedProject::load_file(path)
}

#[test]
fn the_schema_is_the_stable_urn_at_major_one() {
    let schema = schema();
    assert_eq!(schema["$id"].as_str().expect("id"), CONFIG_URN);
    assert_eq!(schema["properties"]["schema_version"]["const"], json!(1));
    assert_eq!(SUPPORTED_SCHEMA_VERSION, 1);
    assert_eq!(
        schema["required"].as_array().expect("required").len(),
        3,
        "only schema_version, project and artifacts are required"
    );
}

#[test]
fn the_committed_example_validates_and_loads() {
    let doc = example_config();
    let errors = errors(&doc);
    assert!(errors.is_empty(), "{errors:?}\n{doc}");
    assert_eq!(doc["schema_version"], 1);
    assert_eq!(doc["artifacts"]["required"][0], "elf");
}

#[test]
fn a_minimal_config_and_a_policy_default_both_validate() {
    assert!(
        errors(&minimal_json()).is_empty(),
        "{:?}",
        errors(&minimal_json())
    );

    let default = serde_json::to_value(new_config("brake-node", &GatePolicy::default()))
        .expect("a config always serializes");
    assert!(
        errors(&default).is_empty(),
        "a config written by the Save dialog must satisfy the published contract: {default}"
    );
    assert!(
        default.get("version").is_none(),
        "an unconfigured [version] section is absent, never an empty or null table: {default}"
    );
}

#[test]
fn the_schema_and_the_loader_refuse_the_same_invalid_config() {
    // Each pair is the same mistake twice: once as a JSON document for the contract, once as TOML for
    // the loader. A config the schema accepts and the loader rejects — or the other way round — is a
    // contract that has quietly split in two, which is what this test exists to catch.
    let cases = [
        (
            "a schema_version this build does not know",
            json!({ "schema_version": 2, "project": { "name": "n" }, "artifacts": { "required": ["elf"] } }),
            "schema_version = 2\n[project]\nname = \"n\"\n[artifacts]\nrequired = [\"elf\"]\n",
        ),
        (
            "a required list without elf",
            json!({ "schema_version": 1, "project": { "name": "n" }, "artifacts": { "required": ["bin"] } }),
            "schema_version = 1\n[project]\nname = \"n\"\n[artifacts]\nrequired = [\"bin\"]\n",
        ),
        (
            "a requirement outside the v1 vocabulary",
            json!({ "schema_version": 1, "project": { "name": "n" }, "artifacts": { "required": ["elf", "pe"] } }),
            "schema_version = 1\n[project]\nname = \"n\"\n[artifacts]\nrequired = [\"elf\", \"pe\"]\n",
        ),
        (
            "a project name that is nothing",
            json!({ "schema_version": 1, "project": { "name": "" }, "artifacts": { "required": ["elf"] } }),
            "schema_version = 1\n[project]\nname = \"\"\n[artifacts]\nrequired = [\"elf\"]\n",
        ),
        (
            "a zero budget",
            json!({ "schema_version": 1, "project": { "name": "n" }, "artifacts": { "required": ["elf"] }, "memory": { "flash_budget": 0 } }),
            "schema_version = 1\n[project]\nname = \"n\"\n[artifacts]\nrequired = [\"elf\"]\n[memory]\nflash_budget = 0\n",
        ),
        (
            "an abbreviated expected_commit",
            json!({ "schema_version": 1, "project": { "name": "n" }, "artifacts": { "required": ["elf"] }, "release": { "expected_commit": "32b23aa" } }),
            "schema_version = 1\n[project]\nname = \"n\"\n[artifacts]\nrequired = [\"elf\"]\n[release]\nexpected_commit = \"32b23aa\"\n",
        ),
        (
            "an unknown on_unknown disposition",
            json!({ "schema_version": 1, "project": { "name": "n" }, "artifacts": { "required": ["elf"] }, "gate": { "on_unknown": { "git_clean": "ignore" } } }),
            "schema_version = 1\n[project]\nname = \"n\"\n[artifacts]\nrequired = [\"elf\"]\n[gate.on_unknown]\ngit_clean = \"ignore\"\n",
        ),
        (
            "a version source the MVP does not implement",
            json!({ "schema_version": 1, "project": { "name": "n" }, "artifacts": { "required": ["elf"] }, "version": { "source": "artifact_metadata" } }),
            "schema_version = 1\n[project]\nname = \"n\"\n[artifacts]\nrequired = [\"elf\"]\n[version]\nsource = \"artifact_metadata\"\n",
        ),
    ];

    for (label, doc, toml_text) in cases {
        assert!(
            !errors(&doc).is_empty(),
            "{label}: the contract accepted it"
        );
        let err = parse(toml_text).expect_err(&format!("{label}: the loader accepted it"));
        assert!(
            !err.to_string().is_empty(),
            "{label}: a rejection has to explain itself"
        );
    }
}

#[test]
fn a_whitespace_only_name_is_a_loader_rule_the_contract_cannot_express() {
    // Recorded rather than papered over: `minLength` counts characters, and JSON Schema has no notion
    // of trimming, so `"  "` passes the published contract while the loader refuses it. The loader is
    // the stricter of the two here, which is the safe direction — the contract never accepts something
    // the product would refuse to name a project by.
    let doc = json!({ "schema_version": 1, "project": { "name": "  " }, "artifacts": { "required": ["elf"] } });
    assert!(errors(&doc).is_empty(), "{:?}", errors(&doc));
    let err =
        parse("schema_version = 1\n[project]\nname = \"  \"\n[artifacts]\nrequired = [\"elf\"]\n")
            .expect_err("a name of only spaces names nothing");
    assert!(err.to_string().contains("project name"), "{err}");
}

#[test]
fn a_key_the_loader_only_warns_about_is_a_contract_violation_on_purpose() {
    // §10 and `04_TECH/08` split these deliberately: an unknown key in major 1 must not stop a release
    // owner from shipping, so the loader warns and preserves it, while the published contract describes
    // the documented shape only. The save path is what keeps the warning honest — it refuses a write it
    // cannot do without dropping that key (ERR-CONFIG-7007).
    let doc = json!({
        "schema_version": 1,
        "project": { "name": "n" },
        "artifacts": { "required": ["elf"] },
        "quantum": { "enabled": true }
    });
    let errors = errors(&doc);
    assert!(
        errors
            .iter()
            .any(|failure| failure.message.contains("not declared")),
        "the closed contract must refuse an undeclared section: {errors:?}"
    );

    let loaded = parse(
        "schema_version = 1\n[project]\nname = \"n\"\n[artifacts]\nrequired = [\"elf\"]\n[quantum]\nenabled = true\n",
    )
    .expect("an unknown key in major 1 is a warning, not a refusal");
    assert!(
        loaded
            .unknown_keys
            .iter()
            .any(|key| key.starts_with("quantum")),
        "{:?}",
        loaded.unknown_keys
    );
}

#[test]
fn every_disposition_the_contract_names_is_one_the_gate_can_answer() {
    let dispositions = schema()["properties"]["gate"]["properties"]["on_unknown"]["properties"]
        .as_object()
        .expect("on_unknown")
        .values()
        .map(|rule| rule["enum"].as_array().expect("enum").clone())
        .collect::<Vec<Vec<Value>>>();
    assert_eq!(
        dispositions.len(),
        7,
        "one disposition per rule that can lose its evidence"
    );
    for allowed in &dispositions {
        assert_eq!(allowed, &vec![json!("review"), json!("block")]);
    }
}

#[test]
fn a_document_the_schema_accepts_round_trips_through_the_loader_unchanged() {
    // The contract is only useful if a config it describes is the config the loader keeps. Written out
    // as TOML, loaded, serialized, validated, then loaded again: the two models must be equal.
    let toml_text =
        std::fs::read_to_string(repo_root().join("examples").join("firmwaresight.toml"))
            .expect("the example config is committed");
    let first = parse(&toml_text).expect("the example loads");
    let written = toml::to_string_pretty(&first.config).expect("the model writes as TOML");
    let second = parse(&written).expect("what was written loads back");
    assert_eq!(
        first.config, second.config,
        "a save must not change what the project means"
    );
    let doc = serde_json::to_value(&second.config).expect("serializes");
    assert!(errors(&doc).is_empty(), "{:?}", errors(&doc));
}
