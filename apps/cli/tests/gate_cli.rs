//! The shipped `fwsight gate` surface: exit codes, stream discipline, portable output and the
//! boundaries the prompt fixes for the command line (§38, §39, §59).
//!
//! These run the real binary, because what is under test is what a release owner gets from a process:
//! which code it exits with, what lands on stdout versus stderr, and whether the document it prints
//! satisfies the published contract.
//!
//! Every project here is a throwaway directory holding a `firmwaresight.toml` and paths into the
//! committed fixture pair. FirmwareSight's own repository is never the Gate subject: this workspace is
//! dirty or clean for reasons that have nothing to do with a release (§26, §61).

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const ARTIFACT: &str = "fixtures/elf/p2-diff/target/firmware.elf";
const ARTIFACT_MAP: &str = "fixtures/elf/p2-diff/target/firmware.map";
const BASELINE: &str = "fixtures/elf/p2-diff/base/firmware.elf";
const BASELINE_MAP: &str = "fixtures/elf/p2-diff/base/firmware.map";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|parent| parent.parent())
        .expect("apps/cli lives at <root>/apps/cli")
        .to_path_buf()
}

struct TempProject(PathBuf);

impl TempProject {
    /// A project directory holding one config. Nothing else is created: the Gate reads what it is
    /// pointed at, and writes nothing it was not asked to.
    fn new(label: &str, config: &str) -> Self {
        let mut dir = std::env::temp_dir();
        dir.push(format!(
            "fwsight-p3-gate-{label}-{}-{:?}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&dir).expect("a temporary project directory");
        std::fs::write(dir.join("firmwaresight.toml"), config).expect("a temporary config");
        Self(dir)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn run(args: &[&str]) -> Output {
    let output = Command::new(env!("CARGO_BIN_EXE_fwsight"))
        .args(args)
        .current_dir(repo_root())
        .output()
        .expect("the fwsight binary is built for its own tests");
    let stderr = String::from_utf8_lossy(&output.stderr).to_lowercase();
    assert!(!stderr.contains("panicked"), "the CLI panicked:\n{stderr}");
    output
}

fn text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn errors(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// The policy every rule can answer without Git, a baseline or a notes file: those three are the parts
/// of a release a project may choose not to involve, and a Gate must stay readable either way.
fn quiet_policy() -> String {
    "schema_version = 1\n\
     [project]\nname = \"brake-node\"\n\
     [artifacts]\nrequired = [\"elf\"]\n\
     [release]\nrequire_clean_git = false\nrequire_release_notes = false\n"
        .to_owned()
}

fn gate(project: &TempProject, extra: &[&str]) -> Output {
    let dir = project.path().to_string_lossy().into_owned();
    let mut args = vec!["gate", "--project", &dir, "--artifact", ARTIFACT];
    args.extend_from_slice(extra);
    run(&args)
}

fn state_of(document: &serde_json::Value, rule: &str) -> String {
    document["findings"]
        .as_array()
        .expect("findings")
        .iter()
        .find(|finding| finding["rule_id"] == rule)
        .unwrap_or_else(|| panic!("no finding for {rule} in {document}"))["state"]
        .as_str()
        .expect("state")
        .to_owned()
}

fn summary_of(document: &serde_json::Value, rule: &str) -> String {
    document["findings"]
        .as_array()
        .expect("findings")
        .iter()
        .find(|finding| finding["rule_id"] == rule)
        .expect("finding")["summary"]
        .as_str()
        .expect("summary")
        .to_owned()
}

// --------------------------------------------------------------------------- refusals

#[test]
fn a_project_without_a_config_exits_two_and_names_the_file_it_needed() {
    let empty = TempProject::new("no-config", "");
    std::fs::remove_file(empty.path().join("firmwaresight.toml")).expect("remove the config");
    let dir = empty.path().to_string_lossy().into_owned();

    let output = run(&["gate", "--project", &dir, "--artifact", ARTIFACT]);
    assert_eq!(output.status.code(), Some(2), "{}", errors(&output));
    let stderr = errors(&output);
    assert!(stderr.contains("ERR-CONFIG-7001"), "{stderr}");
    assert!(
        stderr.contains("firmwaresight.toml"),
        "the refusal has to say what is missing: {stderr}"
    );
    assert!(
        text(&output).is_empty(),
        "a refusal prints no verdict: {}",
        text(&output)
    );
}

#[test]
fn an_invalid_toml_exits_two_rather_than_guessing_a_policy() {
    let project = TempProject::new("malformed", "schema_version = 1\n[project\nname = ");
    let output = gate(&project, &[]);
    assert_eq!(output.status.code(), Some(2), "{}", errors(&output));
    assert!(errors(&output).contains("ERR-CONFIG-7002"));
}

#[test]
fn a_schema_version_this_build_does_not_know_exits_two_and_refuses_to_reset() {
    let project = TempProject::new(
        "future",
        "schema_version = 7\n[project]\nname = \"brake-node\"\n[artifacts]\nrequired = [\"elf\"]\n",
    );
    let output = gate(&project, &[]);
    assert_eq!(output.status.code(), Some(2), "{}", errors(&output));
    let stderr = errors(&output);
    assert!(stderr.contains("ERR-CONFIG-7003"), "{stderr}");
    assert!(
        stderr.contains("7"),
        "the refusal quotes the version it found: {stderr}"
    );
}

#[test]
fn an_artifact_that_is_not_a_firmware_exits_three() {
    let project = TempProject::new("bad-artifact", &quiet_policy());
    let output = gate_at(&project, "schemas/gate-results.schema.json", &["--json"]);
    assert_eq!(output.status.code(), Some(3), "{}", errors(&output));
    assert!(
        !text(&output).contains("\"findings\""),
        "a run that never read an artifact produces no verdict: {}",
        text(&output)
    );
}

/// `gate` with the shipped artifact swapped for another file.
fn gate_at(project: &TempProject, artifact: &str, extra: &[&str]) -> Output {
    let dir = project.path().to_string_lossy().into_owned();
    let mut args = vec!["gate", "--project", &dir, "--artifact", artifact];
    args.extend_from_slice(extra);
    run(&args)
}

#[test]
fn a_gate_writes_nothing_into_the_project_it_evaluates() {
    // §38: no hidden database. A command line that left a `.sqlite` beside a release would make the
    // CLI and the desktop disagree about where the history lives.
    let project = TempProject::new("no-db", &quiet_policy());
    let output = gate(&project, &["--map", ARTIFACT_MAP]);
    assert!(output.status.success(), "{}", errors(&output));

    let written: Vec<_> = std::fs::read_dir(project.path())
        .expect("the project directory still exists")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        written,
        vec!["firmwaresight.toml".to_owned()],
        "the only file in the project is the one the release owner wrote"
    );
}

// --------------------------------------------------------------------------- verdicts

#[test]
fn a_policy_every_rule_can_answer_passes_with_exit_zero() {
    let project = TempProject::new("pass", &quiet_policy());
    let output = gate(&project, &["--map", ARTIFACT_MAP, "--json"]);
    assert_eq!(output.status.code(), Some(0), "{}", errors(&output));

    let document: serde_json::Value = serde_json::from_str(&text(&output)).expect("one document");
    assert_eq!(state_of(&document, "git.clean"), "N/A");
    assert_eq!(state_of(&document, "artifacts.required"), "PASS");
    assert_eq!(state_of(&document, "artifacts.hashes"), "PASS");
    assert_eq!(state_of(&document, "memory.flash_budget"), "N/A");
    assert_eq!(state_of(&document, "release.notes"), "N/A");
    assert_eq!(document["extensions"]["overall_effective_severity"], "PASS");
}

#[test]
fn growth_over_the_configured_review_threshold_exits_four() {
    let project = TempProject::new(
        "growth-review",
        &format!(
            "{}\n[diff]\nflash_growth_review_bytes = 100\n",
            quiet_policy()
        ),
    );
    let output = gate(
        &project,
        &[
            "--map",
            ARTIFACT_MAP,
            "--baseline",
            BASELINE,
            "--baseline-map",
            BASELINE_MAP,
            "--json",
        ],
    );
    assert_eq!(output.status.code(), Some(4), "{}", errors(&output));
    let document: serde_json::Value = serde_json::from_str(&text(&output)).expect("one document");
    assert_eq!(state_of(&document, "diff.growth"), "REVIEW");
    assert_eq!(
        document["extensions"]["overall_effective_severity"],
        "REVIEW"
    );
}

#[test]
fn the_growth_the_gate_reports_is_the_number_the_diff_command_reports() {
    // One calculation, two surfaces (§23): the Gate reads P2's Core diff rather than re-measuring, so
    // the byte count a release owner sees in Compare is the byte count the Gate scored.
    let project = TempProject::new(
        "growth-agrees",
        &format!(
            "{}\n[diff]\nflash_growth_review_bytes = 100\n",
            quiet_policy()
        ),
    );
    let gate_output = gate(
        &project,
        &[
            "--map",
            ARTIFACT_MAP,
            "--baseline",
            BASELINE,
            "--baseline-map",
            BASELINE_MAP,
            "--json",
        ],
    );
    let diff_output = run(&[
        "diff",
        BASELINE,
        ARTIFACT,
        "--old-map",
        BASELINE_MAP,
        "--new-map",
        ARTIFACT_MAP,
        "--json",
    ]);

    let diff: serde_json::Value = serde_json::from_str(&text(&diff_output)).expect("diff json");
    let delta = diff["memory"]["nonvolatile"]["delta"]
        .as_i64()
        .expect("delta");
    assert_eq!(delta, 120, "the committed fixture pair grows by 120 bytes");
    let summary = summary_of(
        &serde_json::from_str(&text(&gate_output)).expect("gate json"),
        "diff.growth",
    );
    assert!(
        summary.contains(&format!("+{delta} B")),
        "the Gate reports the same growth: {summary}"
    );
}

#[test]
fn a_growth_threshold_without_a_baseline_is_an_unknown_and_not_a_pass() {
    // §59: a threshold with nothing to compare against is a missing fact. It is not "no growth", and
    // reporting it as one would let an un-baselined release read as a clean build.
    let project = TempProject::new(
        "threshold-no-baseline",
        &format!(
            "{}\n[diff]\nflash_growth_review_bytes = 100\n",
            quiet_policy()
        ),
    );
    let output = gate(&project, &["--map", ARTIFACT_MAP, "--json"]);
    let document: serde_json::Value = serde_json::from_str(&text(&output)).expect("one document");
    assert_eq!(state_of(&document, "diff.growth"), "UNKNOWN");
    assert_eq!(
        document["extensions"]["overall_effective_severity"], "REVIEW",
        "the rule's own disposition decides the code"
    );
    assert_eq!(output.status.code(), Some(4), "{}", errors(&output));
    assert!(
        document["extensions"]["baseline_snapshot_id"].is_null(),
        "and the document says there was no baseline: {document}"
    );
}

#[test]
fn a_missing_release_notes_file_blocks_with_exit_five() {
    let project = TempProject::new("notes-missing", &notes_required_policy());
    let output = gate(&project, &["--json"]);
    assert_eq!(output.status.code(), Some(5), "{}", errors(&output));
    let document: serde_json::Value = serde_json::from_str(&text(&output)).expect("one document");
    assert_eq!(state_of(&document, "release.notes"), "BLOCK");
}

/// A required notes file. Absence is a policy failure the Gate can state deterministically, which is
/// why it blocks rather than reporting a gap.
fn notes_required_policy() -> String {
    "schema_version = 1\n\
     [project]\nname = \"brake-node\"\n\
     [artifacts]\nrequired = [\"elf\"]\n\
     [release]\nrequire_clean_git = false\nrequire_release_notes = true\n"
        .to_owned()
}

#[test]
fn an_unreadable_release_notes_file_is_an_unknown_whose_disposition_decides_the_code() {
    // The same gap, two policies: ADR-0023 keeps the factual state `UNKNOWN` and lets `on_unknown`
    // choose whether a release owner has to answer it or the release stops.
    for (disposition, expected) in [("review", 4), ("block", 5)] {
        let project = TempProject::new(
            &format!("notes-unreadable-{disposition}"),
            &format!(
                "schema_version = 1\n[project]\nname = \"brake-node\"\n[artifacts]\nrequired = \
                 [\"elf\"]\n[release]\nrequire_clean_git = false\nrequire_release_notes = \
                 true\nrelease_notes_path = \"notes\"\n[gate.on_unknown]\nrelease_notes = \"{disposition}\"\n",
            ),
        );
        std::fs::create_dir(project.path().join("notes"))
            .expect("a directory where the notes are named");

        let output = gate(&project, &["--json"]);
        assert_eq!(output.status.code(), Some(expected), "{}", errors(&output));
        let document: serde_json::Value =
            serde_json::from_str(&text(&output)).expect("one document");
        assert_eq!(
            state_of(&document, "release.notes"),
            "UNKNOWN",
            "a read failure is never recorded as a missing file, still less as a pass"
        );
        assert_eq!(
            document["findings"]
                .as_array()
                .expect("findings")
                .iter()
                .find(|finding| finding["rule_id"] == "release.notes")
                .expect("finding")["effective_severity"],
            if expected == 5 { "BLOCK" } else { "REVIEW" }
        );
    }
}

#[test]
fn a_dirty_workspace_blocks_and_never_claims_the_artifact_was_built_from_that_commit() {
    // §50's wording rule, checked on a real repository: the Gate reports facts about the *directory it
    // was pointed at*. "Firmware was built from this commit" is forbidden, because the workspace says
    // nothing about how the binary was produced.
    let project = TempProject::new("dirty-repo", &quiet_policy_with_git());
    init_repository(project.path());
    // Everything the project holds is tracked, so the workspace is dirty because of the change below
    // and not because a test fixture was left untracked.
    std::fs::write(project.path().join("SOURCE.txt"), "first\n").expect("a tracked file");
    git(&["add", "-A"], project.path());
    git(&["commit", "-q", "-m", "release candidate"], project.path());
    std::fs::write(project.path().join("SOURCE.txt"), "second\n").expect("an uncommitted change");

    let output = gate(&project, &["--json"]);
    assert_eq!(output.status.code(), Some(5), "{}", errors(&output));
    let document: serde_json::Value = serde_json::from_str(&text(&output)).expect("one document");
    assert_eq!(state_of(&document, "git.clean"), "BLOCK");

    let rendered = document.to_string().to_lowercase();
    for forbidden in ["built from", "builds from", "was built"] {
        assert!(
            !rendered.contains(forbidden),
            "workspace provenance cannot prove how the binary was produced: `{forbidden}`"
        );
    }
    assert!(
        document["findings"]
            .as_array()
            .expect("findings")
            .iter()
            .any(|finding| finding["state"] == "BLOCK"
                && finding["summary"]
                    .as_str()
                    .expect("summary")
                    .contains("uncommitted changes")),
        "{document}"
    );
}

#[test]
fn a_clean_workspace_passes_the_provenance_rule_it_can_answer() {
    let project = TempProject::new("clean-repo", &quiet_policy_with_git());
    init_repository(project.path());
    std::fs::write(project.path().join("SOURCE.txt"), "shipped\n").expect("a tracked file");
    git(&["add", "-A"], project.path());
    git(&["commit", "-q", "-m", "release candidate"], project.path());

    let output = gate(&project, &["--json"]);
    assert_eq!(output.status.code(), Some(0), "{}", errors(&output));
    let document: serde_json::Value = serde_json::from_str(&text(&output)).expect("one document");
    assert_eq!(state_of(&document, "git.clean"), "PASS");
    // No expected commit is declared, so the identity rule does not apply rather than failing.
    assert_eq!(
        state_of(&document, "release.commit_matches_expected"),
        "N/A"
    );
}

fn quiet_policy_with_git() -> String {
    "schema_version = 1\n\
     [project]\nname = \"brake-node\"\n\
     [artifacts]\nrequired = [\"elf\"]\n\
     [release]\nrequire_clean_git = true\nrequire_release_notes = false\n"
        .to_owned()
}

fn git(args: &[&str], dir: &Path) {
    let status = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect(
            "the Gate provenance tests run against a real repository, so git must be installed",
        );
    assert!(
        status.status.success(),
        "`git {}` failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&status.stderr)
    );
}

/// A throwaway repository with an identity configured, so a commit does not depend on the machine's
/// global git config.
fn init_repository(dir: &Path) {
    git(&["init", "-q"], dir);
    git(&["config", "user.email", "release@example.invalid"], dir);
    git(&["config", "user.name", "Release Owner"], dir);
}

// --------------------------------------------------------------------------- the document

#[test]
fn json_mode_prints_exactly_one_gate_document_to_stdout_and_diagnostics_to_stderr() {
    let project = TempProject::new("json-shape", &quiet_policy());
    let output = gate(&project, &["--map", ARTIFACT_MAP, "--json"]);
    let stdout = text(&output);
    assert_eq!(
        stdout.matches('\n').count(),
        1,
        "stdout must carry one compact document, got: {stdout}"
    );
    assert!(stdout.ends_with("}\n"));
    let stderr = errors(&output);
    assert!(stderr.contains("diagnostics: map"), "{stderr}");
    assert!(stderr.contains("diagnostics: git"), "{stderr}");
    assert!(stderr.contains("operation op-"), "{stderr}");
}

#[test]
fn the_gate_document_validates_against_the_published_gate_results_v1() {
    let project = TempProject::new(
        "schema",
        &format!(
            "{}\n[diff]\nflash_growth_review_bytes = 100\n",
            quiet_policy()
        ),
    );
    let output = gate(
        &project,
        &[
            "--map",
            ARTIFACT_MAP,
            "--baseline",
            BASELINE,
            "--baseline-map",
            BASELINE_MAP,
            "--json",
        ],
    );
    let document: serde_json::Value = serde_json::from_str(&text(&output)).expect("one document");

    let schema_path = repo_root().join("schemas").join("gate-results.schema.json");
    let schema: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&schema_path).expect("the schema is committed"),
    )
    .expect("the schema parses");
    let failures = firmwaresight_report::schema_check::validate(&schema, &document);
    assert!(failures.is_empty(), "{failures:?}");
    assert_eq!(document["schema_version"], 1);
    assert_eq!(
        document["run_id"].as_str().expect("run id").len(),
        69,
        "a run id is `gate-` plus a 64-hex digest"
    );
    assert!(
        document["run_id"]
            .as_str()
            .expect("run id")
            .starts_with("gate-")
    );
    assert_eq!(document["findings"].as_array().expect("findings").len(), 10);
}

#[test]
fn the_same_input_produces_byte_identical_json() {
    let project = TempProject::new("determinism", &quiet_policy());
    let args = ["--map", ARTIFACT_MAP, "--json"];
    let first = text(&gate(&project, &args));
    let second = text(&gate(&project, &args));
    assert_eq!(first, second, "a run id is a fingerprint, not a clock");

    // The document carries no timestamp of its own: a Gate run is identified by what was evaluated.
    assert!(
        !first.contains("2026-"),
        "a wall clock leaked into the document: {first}"
    );
}

#[test]
fn no_host_path_appears_in_the_gate_document() {
    let project = TempProject::new("no-host-path", &quiet_policy());
    let output = gate(&project, &["--map", ARTIFACT_MAP, "--json"]);
    let stdout = text(&output);
    assert!(
        !stdout.contains('\\'),
        "a Windows separator cannot appear in a portable document: {stdout}"
    );
    assert!(
        !stdout.contains(&project.path().to_string_lossy().into_owned()),
        "the project directory is a host path and must not be quoted"
    );
    let root = repo_root().to_string_lossy().into_owned();
    assert!(
        !stdout.contains(&root),
        "the repository root escaped into the document"
    );
}

#[test]
fn the_human_report_reads_as_a_verdict_and_the_json_carries_the_same_states() {
    let project = TempProject::new(
        "human",
        &format!(
            "{}\n[diff]\nflash_growth_review_bytes = 100\n",
            quiet_policy()
        ),
    );
    let compared = [
        "--map",
        ARTIFACT_MAP,
        "--baseline",
        BASELINE,
        "--baseline-map",
        BASELINE_MAP,
    ];
    let human = gate(&project, &compared);
    let mut machine_args = compared.to_vec();
    machine_args.push("--json");
    let machine: serde_json::Value =
        serde_json::from_str(&text(&gate(&project, &machine_args))).expect("one document");
    let stdout = text(&human);

    assert!(stdout.starts_with("Release Gate: REVIEW"), "{stdout}");
    assert!(stdout.contains("diff.growth"), "{stdout}");
    assert_eq!(
        state_of(&machine, "diff.growth"),
        "REVIEW",
        "the two surfaces must not disagree"
    );
    assert_eq!(human.status.code(), Some(4), "{}", errors(&human));
}
