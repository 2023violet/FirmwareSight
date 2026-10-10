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

// ------------------------------------------------------------------ release attachments (`C1-U4`)

/// A policy that asks for the two kinds nothing in this build analyzes. `elf` stays in the list because the
/// config requires it (`config.rs:434`) and an attachment is never allowed to stand in for it
/// (`ADR-0030` D-3) — which is what the last two cases below pin.
const BIN_HEX_POLICY: &str = "schema_version = 1\n\
     [project]\nname = \"brake-node\"\n\
     [artifacts]\nrequired = [\"elf\", \"bin\", \"hex\"]\n\
     [release]\nrequire_clean_git = false\nrequire_release_notes = false\n";

/// As above, with `hex` left out: a required kind that one name can declare and another cannot.
const BIN_POLICY: &str = "schema_version = 1\n\
     [project]\nname = \"brake-node\"\n\
     [artifacts]\nrequired = [\"elf\", \"bin\"]\n\
     [release]\nrequire_clean_git = false\nrequire_release_notes = false\n";

/// One file the release owner could have pointed `--attach` at, and the path as the process receives it.
fn attachable(project: &TempProject, name: &str, bytes: &[u8]) -> String {
    let path = project.path().join(name);
    std::fs::write(&path, bytes).expect("an attachment fixture");
    path.to_string_lossy().into_owned()
}

fn json_document(output: &Output) -> serde_json::Value {
    serde_json::from_str(&text(output)).expect("exactly one JSON document on stdout")
}

#[test]
fn two_attached_files_satisfy_kinds_no_analyzed_artifact_can() {
    let project = TempProject::new("attach-pass", BIN_HEX_POLICY);
    let bin = attachable(&project, "app.bin", b"APP-BYTES");
    let hex = attachable(&project, "boot.hex", b":00000001FF");

    let output = gate(&project, &["--attach", &bin, "--attach", &hex, "--json"]);
    assert_eq!(output.status.code(), Some(0), "{}", errors(&output));
    let document = json_document(&output);
    assert_eq!(state_of(&document, "artifacts.required"), "PASS");
    assert_eq!(state_of(&document, "artifacts.hashes"), "PASS");
    assert!(
        summary_of(&document, "artifacts.required").contains("release attachments"),
        "the sentence that credits a PASS to bytes must say so: {}",
        summary_of(&document, "artifacts.required")
    );

    let extensions = document["extensions"].as_object().expect("extensions");
    assert_eq!(
        extensions["canonical_input_label"],
        serde_json::json!("firmwaresight-gate-input/2"),
        "a run that bound bytes must name the grammar its id was taken over"
    );
    let rows = extensions["attachments"]
        .as_array()
        .expect("attachment rows");
    assert_eq!(rows.len(), 2, "{rows:?}");
    // Identity order, not typing order: the rows are sorted by kind word then digest.
    assert_eq!(rows[0]["kind"], serde_json::json!("bin"));
    assert_eq!(rows[1]["kind"], serde_json::json!("hex"));
    for row in rows {
        assert_eq!(
            row["sha256"].as_str().expect("digest").len(),
            64,
            "a full digest, not a shortened one: {row}"
        );
        assert!(row["byte_size"].as_u64().expect("size") > 0);
    }

    // The stderr half of the `--json` contract, and the only place the CLI says which kind each *name*
    // declared — the document carries the rows, this line carries the reading of the owner's words.
    assert!(
        errors(&output).contains("diagnostics: attachments app.bin=bin, boot.hex=hex"),
        "{}",
        errors(&output)
    );
}

#[test]
fn a_gate_with_no_attach_flag_writes_the_document_it_wrote_before_the_flag_existed() {
    let project = TempProject::new("attach-none", &quiet_policy());
    let output = gate(&project, &["--json"]);
    assert_eq!(output.status.code(), Some(0), "{}", errors(&output));
    let document = json_document(&output);
    let extensions = document["extensions"].as_object().expect("extensions");
    for key in ["canonical_input_label", "attachments"] {
        assert!(
            !extensions.contains_key(key),
            "an attachment-free run carries neither half of the pair: {extensions:?}"
        );
    }
    assert!(
        !errors(&output).contains("diagnostics: attachments"),
        "a line about named files must not appear when none were: {}",
        errors(&output)
    );
}

#[test]
fn a_required_kind_no_attachment_covers_blocks_and_names_both_sets() {
    let project = TempProject::new("attach-partial", BIN_HEX_POLICY);
    let bin = attachable(&project, "app.bin", b"APP-BYTES");

    let output = gate(&project, &["--attach", &bin, "--json"]);
    assert_eq!(output.status.code(), Some(5), "{}", errors(&output));
    let document = json_document(&output);
    assert_eq!(state_of(&document, "artifacts.required"), "BLOCK");
    let summary = summary_of(&document, "artifacts.required");
    assert!(
        summary.contains("neither the analyzed build nor the release attachments")
            && summary.contains("hex"),
        "the refusal must say which two sets it looked in: {summary}"
    );
}

#[test]
fn a_path_that_does_not_exist_refuses_before_any_verdict_exists() {
    let project = TempProject::new("attach-missing", BIN_HEX_POLICY);
    let absent = project
        .path()
        .join("no-such.bin")
        .to_string_lossy()
        .into_owned();

    // Human mode: a refusal prints no verdict on stdout, and it is the engine's own code, not a CLI wording.
    let human = gate(&project, &["--attach", &absent]);
    assert_eq!(human.status.code(), Some(3), "{}", errors(&human));
    assert!(text(&human).is_empty(), "{}", text(&human));
    let stderr = errors(&human);
    assert!(stderr.contains("ERR-BUNDLE-6115"), "{stderr}");
    assert!(stderr.contains("not a readable regular file"), "{stderr}");

    // Machine mode: the envelope on stdout is the same refusal, and still no gate document.
    let machine = gate(&project, &["--attach", &absent, "--json"]);
    assert_eq!(machine.status.code(), Some(3), "{}", errors(&machine));
    let envelope = json_document(&machine);
    assert_eq!(
        envelope["code"],
        serde_json::json!("ERR-BUNDLE-6115"),
        "{envelope}"
    );
    assert!(
        envelope.get("findings").is_none(),
        "a refused run writes no findings: {envelope}"
    );
}

#[test]
fn an_empty_file_is_refused_as_carrying_nothing_to_attach() {
    let project = TempProject::new("attach-empty", BIN_HEX_POLICY);
    let empty = attachable(&project, "zero.bin", b"");

    let output = gate(&project, &["--attach", &empty]);
    assert_eq!(output.status.code(), Some(3), "{}", errors(&output));
    let stderr = errors(&output);
    assert!(stderr.contains("ERR-BUNDLE-6117"), "{stderr}");
    assert!(stderr.contains("empty"), "{stderr}");
    assert!(text(&output).is_empty(), "{}", text(&output));
}

#[test]
fn a_name_this_rule_does_not_know_is_declared_unknown_and_satisfies_nothing() {
    let project = TempProject::new("attach-unknown", BIN_POLICY);
    // `.ihex` is a real Intel HEX spelling this CLI rule deliberately does not claim: the flag carries no
    // kind, so the honest declaration is `unknown`, and `unknown` satisfies no required kind.
    let img = attachable(&project, "payload.ihex", b"BYTES");

    let output = gate(&project, &["--attach", &img, "--json"]);
    assert_eq!(output.status.code(), Some(5), "{}", errors(&output));
    let document = json_document(&output);
    assert_eq!(state_of(&document, "artifacts.required"), "BLOCK");
    let rows = document["extensions"]["attachments"]
        .as_array()
        .expect("rows");
    assert_eq!(rows[0]["kind"], serde_json::json!("unknown"), "{rows:?}");
    assert!(
        errors(&output).contains("payload.ihex=unknown"),
        "{}",
        errors(&output)
    );
}

#[test]
fn a_mixed_case_name_declares_the_kind_a_lowercase_name_declares() {
    let project = TempProject::new("attach-case", BIN_POLICY);
    let shouted = attachable(&project, "FIRMWARE.BIN", b"BYTES");

    let output = gate(&project, &["--attach", &shouted, "--json"]);
    assert_eq!(output.status.code(), Some(0), "{}", errors(&output));
    let document = json_document(&output);
    let rows = document["extensions"]["attachments"]
        .as_array()
        .expect("rows");
    assert_eq!(rows[0]["kind"], serde_json::json!("bin"), "{rows:?}");
}

#[test]
fn an_elf_pointed_at_by_attach_is_recorded_as_declared_and_never_as_an_analyzed_build() {
    let project = TempProject::new("attach-elf", &quiet_policy());
    let elf = repo_root().join(ARTIFACT).to_string_lossy().into_owned();

    let output = gate(&project, &["--attach", &elf, "--json"]);
    assert_eq!(output.status.code(), Some(0), "{}", errors(&output));
    let document = json_document(&output);
    let rows = document["extensions"]["attachments"]
        .as_array()
        .expect("rows");
    assert_eq!(
        rows[0]["kind"],
        serde_json::json!("unknown"),
        "a name rule can never declare the kind only analysis produces: {rows:?}"
    );
    assert_eq!(
        state_of(&document, "artifacts.required"),
        "PASS",
        "and the requirement is still met by the analyzed artifact, not by the attached copy"
    );
}

#[test]
fn changing_only_an_attached_files_bytes_changes_the_run_id() {
    // `T-C1-02` at the edge a person stands at: the bytes a release owner dropped into a folder are part of
    // what the run identity binds, so replacing one is a different run and not the same verdict.
    let project = TempProject::new("attach-growth", BIN_POLICY);
    let bin = attachable(&project, "app.bin", b"FIRST");

    let first = json_document(&gate(&project, &["--attach", &bin, "--json"]));
    attachable(&project, "app.bin", b"SECOND-BYTES");
    let second = json_document(&gate(&project, &["--attach", &bin, "--json"]));

    assert_ne!(
        first["run_id"], second["run_id"],
        "the same name over different bytes cannot produce the same id"
    );
    // Measured against the fixture bytes themselves rather than a hand count of them, which is how this
    // assertion got its first number wrong.
    assert_eq!(
        first["extensions"]["attachments"][0]["byte_size"],
        serde_json::json!(b"FIRST".len() as u64)
    );
    assert_eq!(
        second["extensions"]["attachments"][0]["byte_size"],
        serde_json::json!(b"SECOND-BYTES".len() as u64)
    );
}
