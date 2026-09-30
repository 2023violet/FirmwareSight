//! P3 Release Gate over IPC: what the boundary is allowed to say about a Gate run.
//!
//! Core owns the states and the aggregate, storage owns the immutable rows, and the project adapter
//! owns `firmwaresight.toml`, so this file is about the seam between them:
//!
//! - a run is judged against a policy that is *named*, and a default-policy run never reads as a
//!   project's own policy (prompt §40);
//! - the facts a run is judged from are read out of SQLite, so a build whose files have gone is still
//!   gateable and still reviewable (prompt §44);
//! - no host path, no config text and no project root crosses the boundary in either direction
//!   (`AGENTS.md` 7, prompt §46, §53);
//! - the numbers under the findings are Rust's, they agree with the finding they belong to, and a row
//!   whose state is not a deterministic verdict carries no margin (prompt §51, §52);
//! - an acceptance is a separate immutable record: the finding keeps its `REVIEW`, the aggregate moves,
//!   and only a review is acceptable (prompt §48);
//! - and a save the shell refuses leaves the config on disk exactly as it was (prompt §42).
//!
//! These call the same `Session` methods the commands do, with no window and no runtime, the way the
//! P1, P2 and P0 files do. Every project here is a throwaway folder under the system temp directory:
//! FirmwareSight's own repository is never a Gate subject (prompt §26, §61).

use std::path::{Path, PathBuf};
use std::process::Command;

use firmwaresight_core::domain::gate::GatePolicy;
use firmwaresight_desktop::ipc::{
    AcceptReviewRequestDto, GateFindingRowDto, GateRunDto, GateRunRequestDto, ProjectContextDto,
    ProjectPolicyDto, UnknownPolicyDto,
};
use firmwaresight_desktop::{Session, service::FixtureCatalog};
use firmwaresight_project::policy_sha256;
use serde_json::Value;

/// The evidence locator schemes Core is allowed to emit. Anything else in a ref is a fact nobody
/// observed, and a Windows path would be a host leak (`04_TECH/27`, prompt §46).
const LOCATOR_SCHEMES: [&str; 6] = [
    "artifact:",
    "diff:",
    "evidence:",
    "file:",
    "git:",
    "policy:",
];

/// The marker in every temporary folder this file makes, so a leak of one is detectable by name.
const TEMP_MARKER: &str = "fwsight-p3-release";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent() // apps/desktop/src-tauri -> apps/desktop
        .and_then(Path::parent) // -> apps
        .and_then(Path::parent) // -> repo root
        .expect("the desktop shell lives at <root>/apps/desktop/src-tauri")
        .to_path_buf()
}

fn fixture(side: &str) -> (PathBuf, PathBuf) {
    let root = repo_root();
    (
        root.join(format!("fixtures/elf/p2-diff/{side}/firmware.elf")),
        root.join(format!("fixtures/elf/p2-diff/{side}/firmware.map")),
    )
}

/// A unique name under the system temp directory.
fn temp_name(label: &str, extension: &str) -> PathBuf {
    let unique = format!(
        "{TEMP_MARKER}-{label}-{}-{:x}{extension}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );
    std::env::temp_dir().join(unique)
}

struct TempDb(PathBuf);

impl TempDb {
    fn new(label: &str) -> Self {
        let path = temp_name(label, ".sqlite");
        let _ = std::fs::remove_file(&path);
        Self(path)
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

/// A `firmwaresight.toml`, assembled from the keys one test cares about.
///
/// One builder rather than concatenation: a TOML table may appear exactly once, so a test that
/// appended `[release]` to a config that already carried one would fail for a reason that has
/// nothing to do with what it is claiming.
struct Config {
    name: &'static str,
    required: &'static [&'static str],
    flash_budget: Option<u64>,
    ram_budget: Option<u64>,
    require_clean_git: bool,
    require_release_notes: bool,
    release_notes_path: Option<&'static str>,
    flash_growth_review_bytes: Option<u64>,
    ram_growth_review_bytes: Option<u64>,
    /// Verbatim lines inside `[release]`, for a key this build does not know.
    extra_release_keys: Vec<&'static str>,
}

impl Config {
    /// A policy that asks for the two files this fixture pair really holds and requires nothing a
    /// build could not answer. Each test raises the one knob it is about.
    fn permissive() -> Self {
        Self {
            name: "brake-node",
            required: &["elf", "map"],
            flash_budget: None,
            ram_budget: None,
            require_clean_git: false,
            require_release_notes: false,
            release_notes_path: None,
            flash_growth_review_bytes: None,
            ram_growth_review_bytes: None,
            extra_release_keys: Vec::new(),
        }
    }

    fn clean_git_required(mut self) -> Self {
        self.require_clean_git = true;
        self
    }

    fn notes_required(mut self) -> Self {
        self.require_release_notes = true;
        self.release_notes_path = Some("docs/RELEASE_NOTES.md");
        self
    }

    fn flash_budget(mut self, bytes: u64) -> Self {
        self.flash_budget = Some(bytes);
        self
    }

    fn ram_budget(mut self, bytes: u64) -> Self {
        self.ram_budget = Some(bytes);
        self
    }

    fn flash_growth(mut self, bytes: u64) -> Self {
        self.flash_growth_review_bytes = Some(bytes);
        self
    }

    fn ram_growth(mut self, bytes: u64) -> Self {
        self.ram_growth_review_bytes = Some(bytes);
        self
    }

    fn required(mut self, kinds: &'static [&'static str]) -> Self {
        self.required = kinds;
        self
    }

    fn with_unknown_release_key(mut self, line: &'static str) -> Self {
        self.extra_release_keys.push(line);
        self
    }

    fn text(&self) -> String {
        let mut text = format!(
            "schema_version = 1\n\n[project]\nname = \"{}\"\n\n[artifacts]\nrequired = [{}]\n",
            self.name,
            self.required
                .iter()
                .map(|kind| format!("\"{kind}\""))
                .collect::<Vec<_>>()
                .join(", ")
        );
        if self.flash_budget.is_some() || self.ram_budget.is_some() {
            text.push_str("\n[memory]\n");
            if let Some(bytes) = self.flash_budget {
                text.push_str(&format!("flash_budget = {bytes}\n"));
            }
            if let Some(bytes) = self.ram_budget {
                text.push_str(&format!("ram_budget = {bytes}\n"));
            }
        }
        text.push_str("\n[release]\n");
        text.push_str(&format!(
            "require_clean_git = {}\nrequire_release_notes = {}\n",
            self.require_clean_git, self.require_release_notes
        ));
        if let Some(path) = self.release_notes_path {
            text.push_str(&format!("release_notes_path = \"{path}\"\n"));
        }
        for line in &self.extra_release_keys {
            text.push_str(line);
            text.push('\n');
        }
        if self.flash_growth_review_bytes.is_some() || self.ram_growth_review_bytes.is_some() {
            text.push_str("\n[diff]\n");
            if let Some(bytes) = self.flash_growth_review_bytes {
                text.push_str(&format!("flash_growth_review_bytes = {bytes}\n"));
            }
            if let Some(bytes) = self.ram_growth_review_bytes {
                text.push_str(&format!("ram_growth_review_bytes = {bytes}\n"));
            }
        }
        text
    }
}

/// The throwaway project folder holding that config, and optionally the notes file and a real git
/// workspace.
struct TempProject(PathBuf);

impl TempProject {
    fn new(label: &str, config: &Config) -> Self {
        let dir = temp_name(label, "");
        std::fs::create_dir_all(&dir).expect("a temporary project directory");
        std::fs::write(dir.join("firmwaresight.toml"), config.text()).expect("a temporary config");
        Self(dir)
    }

    /// A folder holding a config this build cannot read, for the refusal paths.
    fn with_text(label: &str, text: &str) -> Self {
        let dir = temp_name(label, "");
        std::fs::create_dir_all(&dir).expect("a temporary project directory");
        std::fs::write(dir.join("firmwaresight.toml"), text).expect("a temporary config");
        Self(dir)
    }

    fn with_notes(self) -> Self {
        let docs = self.0.join("docs");
        std::fs::create_dir_all(&docs).expect("a docs folder");
        std::fs::write(
            docs.join("RELEASE_NOTES.md"),
            "# Notes\n\n- grew the bootloader\n",
        )
        .expect("a notes file");
        self
    }

    fn with_repository(self) -> Self {
        git(&["init", "-q"], &self.0);
        git(
            &["config", "user.email", "release@example.invalid"],
            &self.0,
        );
        git(&["config", "user.name", "Release Owner"], &self.0);
        git(&["add", "-A"], &self.0);
        git(&["commit", "-q", "-m", "release candidate"], &self.0);
        self
    }

    fn dirty_workspace(&self) {
        std::fs::write(self.0.join("uncommitted.txt"), "not committed\n")
            .expect("an uncommitted file");
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn config_path(&self) -> PathBuf {
        self.0.join("firmwaresight.toml")
    }

    fn read_config(&self) -> String {
        std::fs::read_to_string(self.config_path()).expect("the config is readable")
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        // A git object store keeps read-only files, so a removal can fail on Windows. The folder name
        // carries a pid and a nanosecond stamp, so a leftover can never be mistaken for another test's.
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn git(args: &[&str], dir: &Path) {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("these provenance tests run against a real repository, so git must be installed");
    assert!(
        output.status.success(),
        "`git {}` failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn session(label: &str) -> (TempDb, Session) {
    let db = TempDb::new(label);
    let session =
        Session::open(FixtureCatalog::from_environment(), &db.0).expect("a session opens");
    (db, session)
}

/// Stage and analyze one fixture side, returning the snapshot id the UI would hold.
fn store(session: &Session, side: &str) -> String {
    let (artifact, map) = fixture(side);
    let selection = session
        .stage_artifact(&artifact, "op-stage")
        .expect("the committed fixture stages");
    session
        .stage_map(&selection.selection_id, &map, "op-map")
        .expect("the fixture MAP stages");
    session
        .analyze_selection(&selection.selection_id, "op-analyze")
        .expect("the fixture analyzes and stores")
        .identity
        .snapshot_id
}

fn load(session: &Session, project: &TempProject) -> ProjectContextDto {
    session
        .load_project_from(Some(project.config_path()), "op-open")
        .expect("the config opens")
        .expect("a chosen path is not the cancel path")
}

fn run(session: &Session, target: &str, baseline: Option<&str>) -> GateRunDto {
    session
        .run_release_gate(
            &GateRunRequestDto {
                snapshot_id: target.to_owned(),
                baseline_snapshot_id: baseline.map(str::to_owned),
            },
            "op-gate",
        )
        .expect("the run completes")
}

fn finding<'a>(run: &'a GateRunDto, rule: &str) -> &'a GateFindingRowDto {
    run.findings
        .iter()
        .find(|row| row.rule_id == rule)
        .unwrap_or_else(|| panic!("no finding for {rule} in {}", run.run_id))
}

fn json(value: &impl serde::Serialize) -> Value {
    serde_json::to_value(value).expect("an IPC payload is JSON by construction")
}

/// A `Vec<String>` as the words a test expects, so a required-kind list reads as `["elf", "map"]`.
fn words(values: &[String]) -> Vec<&str> {
    values.iter().map(String::as_str).collect()
}

/// The form an operator touches before any config exists: every field states nothing, so the shell
/// resolves each one against `GatePolicy::default()` rather than the screen guessing it (prompt §41).
fn unstated(name: &str) -> ProjectPolicyDto {
    ProjectPolicyDto {
        project_name: name.to_owned(),
        required_artifact_kinds: None,
        flash_budget: None,
        ram_budget: None,
        require_clean_git: None,
        require_release_notes: None,
        release_notes_path: None,
        version_source: None,
        version_pattern: None,
        expected_version: None,
        expected_commit: None,
        flash_growth_review_bytes: None,
        ram_growth_review_bytes: None,
        unknown_evidence_review_count: None,
        on_unknown: UnknownPolicyDto {
            git_clean: None,
            commit_matches_release: None,
            version_match: None,
            flash_budget: None,
            ram_budget: None,
            baseline_growth: None,
            release_notes: None,
        },
    }
}

// --------------------------------------------------------------------------- the run

#[test]
fn a_gate_run_answers_every_rule_and_names_the_policy_it_used() {
    let project = TempProject::new("rules", &Config::permissive().notes_required()).with_notes();
    let (_db, session) = session("rules");
    let target = store(&session, "target");
    let context = load(&session, &project);

    let gate = run(&session, &target, None);
    // The ten MVP rules, one finding each, in the canonical order storage keeps them in (prompt §46).
    assert_eq!(gate.findings.len(), 10, "one finding per rule");
    assert_eq!(
        gate.findings
            .iter()
            .map(|row| row.rule_id.as_str())
            .collect::<Vec<_>>()
            .as_slice(),
        [
            "git.clean",
            "release.commit_matches_expected",
            "release.version_matches_policy",
            "artifacts.required",
            "artifacts.hashes",
            "memory.flash_budget",
            "memory.ram_budget",
            "diff.growth",
            "release.notes",
            "evidence.unknown_review",
        ]
    );
    assert!(gate.run_id.starts_with("gate-"));
    assert_eq!(gate.run_id.len(), 69, "gate- plus the 64-hex fingerprint");
    assert_eq!(gate.policy_sha256.len(), 64);
    assert_eq!(gate.policy_source, "firmwaresight.toml");
    assert_eq!(gate.project_name.as_deref(), Some("brake-node"));
    assert_eq!(gate.snapshot_id, target);
    assert!(gate.baseline_snapshot_id.is_none());
    assert_eq!(
        gate.created_at.matches(':').count(),
        2,
        "a UTC stamp: {}",
        gate.created_at
    );
    // The counts add up to the findings, and both come from Core.
    let counted = gate.counts.pass
        + gate.counts.review
        + gate.counts.block
        + gate.counts.unknown
        + gate.counts.not_applicable;
    assert_eq!(counted, gate.findings.len());
    assert_eq!(context.policy_sha256, gate.policy_sha256);
}

#[test]
fn a_config_warning_is_reported_and_never_becomes_a_finding() {
    // prompt §49: `[artifacts] required` with the same kind twice is a warning about the file, not a
    // condition of the build, so it must not reach the findings or change the aggregate.
    let config = Config::permissive().required(&["elf", "map", "map"]);
    let project = TempProject::new("warning", &config);
    let (_db, session) = session("warning");
    let target = store(&session, "target");
    let context = load(&session, &project);

    assert!(
        context
            .warnings
            .iter()
            .any(|warning| warning.contains("more than once")),
        "the duplicate must be named: {:?}",
        context.warnings
    );
    let gate = run(&session, &target, None);
    assert_eq!(gate.findings.len(), 10, "a warning adds no rule");
    assert_eq!(gate.warnings, context.warnings);
    assert!(
        !gate
            .findings
            .iter()
            .any(|row| row.summary.contains("more than once")),
        "a warning never appears as a finding"
    );
    // The duplicated requirement is evaluated once, so the run reads the same as without it.
    assert_eq!(finding(&gate, "artifacts.required").state, "PASS");
}

#[test]
fn the_run_is_judged_from_stored_rows_after_the_files_are_gone() {
    // Analyze from a copy, delete the copy, then gate: a reviewer must be able to reopen a run whose
    // artifact has moved, because the record lives in SQLite (prompt §44).
    let project = TempProject::new("gone", &Config::permissive());
    let (_db, session) = session("gone");
    let staging = project.path().join("build");
    std::fs::create_dir_all(&staging).expect("a staging folder");
    let (artifact, map) = fixture("target");
    let copied = staging.join("firmware.elf");
    let copied_map = staging.join("firmware.map");
    std::fs::copy(&artifact, &copied).expect("the artifact copies");
    std::fs::copy(&map, &copied_map).expect("the MAP copies");

    let selection = session.stage_artifact(&copied, "op-stage").expect("stages");
    session
        .stage_map(&selection.selection_id, &copied_map, "op-map")
        .expect("the MAP stages");
    let snapshot = session
        .analyze_selection(&selection.selection_id, "op-analyze")
        .expect("analyzes")
        .identity
        .snapshot_id;
    std::fs::remove_dir_all(&staging).expect("the build folder is deleted");
    assert!(!copied.exists());

    load(&session, &project);
    let gate = run(&session, &snapshot, None);
    assert_eq!(finding(&gate, "artifacts.required").state, "PASS");
    assert_eq!(finding(&gate, "artifacts.hashes").state, "PASS");
    let elf = gate
        .tables
        .as_ref()
        .expect("a fresh run carries its tables")
        .artifacts
        .iter()
        .find(|row| row.kind == "elf")
        .expect("the ELF row");
    assert!(elf.present);
    assert!(elf.sha256.as_deref().is_some_and(|sha| sha.len() == 64));
}

#[test]
fn nothing_about_the_project_or_the_build_crosses_as_a_path() {
    let project = TempProject::new("no-path", &Config::permissive().notes_required()).with_notes();
    let (_db, session) = session("no-path");
    let target = store(&session, "target");
    let context = load(&session, &project);
    let gate = run(&session, &target, None);

    let root = project.path().display().to_string();
    for payload in [json(&gate), json(&context)] {
        let text = payload.to_string();
        assert!(!text.contains(&root), "the project root leaked: {root}");
        assert!(!text.contains(TEMP_MARKER), "a temp folder name leaked");
        assert!(!text.contains('\\'), "a host path separator leaked: {text}");
        assert!(!has_drive_letter(&text), "an absolute Windows path leaked");
    }
    // Every evidence locator is one of the six schemes Core emits.
    for row in &gate.findings {
        for reference in &row.evidence_refs {
            assert!(
                LOCATOR_SCHEMES
                    .iter()
                    .any(|scheme| reference.starts_with(scheme)),
                "an unexpected locator reached the WebView: {reference}"
            );
        }
    }
}

/// `X:/` or `X:\` for a letter X — the shape a Windows path has that no locator of ours shares.
fn has_drive_letter(text: &str) -> bool {
    let bytes: Vec<char> = text.chars().collect();
    bytes.windows(3).any(|window| {
        window[0].is_ascii_alphabetic() && window[1] == ':' && matches!(window[2], '\\' | '/')
    })
}

#[test]
fn a_run_without_any_config_says_so_instead_of_naming_a_project() {
    let (_db, session) = session("default-policy");
    let target = store(&session, "target");
    let gate = run(&session, &target, None);

    assert!(gate.policy_source.contains("default"));
    assert!(gate.policy_source.contains("no project config loaded"));
    assert!(gate.project_name.is_none());
    // The resolved defaults are still shown, because what the run judged with is a fact (§41).
    let policy = gate
        .policy
        .as_ref()
        .expect("a default run states its policy");
    assert_eq!(policy.require_clean_git, Some(true));
    assert_eq!(
        words(policy.required_artifact_kinds.as_deref().unwrap_or(&[])),
        ["elf"]
    );
    // Nothing was observed for Git or for the notes file, because there was no project to look at.
    assert!(!gate.git.available);
    assert_eq!(gate.git.summary, "No workspace facts were observed");
    assert!(gate.git.head_commit.is_none());
    assert_eq!(gate.git.dirty, None, "unread is not the same as clean");
    assert_eq!(finding(&gate, "git.clean").state, "UNKNOWN");
    let notes = finding(&gate, "release.notes");
    assert_eq!(notes.state, "UNKNOWN");
    // The default requires release notes and the run could not look, so a human has to dispose of it.
    assert_eq!(notes.effective_severity, "REVIEW");
    let table = gate.tables.as_ref().expect("tables");
    assert_eq!(
        table.notes.present, None,
        "an unobserved file is not a missing one"
    );
    assert!(
        table
            .notes
            .reason
            .as_ref()
            .expect("a reason")
            .contains("project folder")
    );
    // Both unknowns ask a human, so the aggregate is a review and nothing more.
    assert_eq!(gate.overall_effective_severity, "REVIEW");
}

#[test]
fn the_same_build_twice_is_refused_rather_than_answered_with_nothing() {
    // A Gate run over one build twice has no growth to threshold. Compare's refusal is the answer
    // here too, rather than a run whose growth row silently reads as zero change (prompt §18).
    let (_db, session) = session("same-pair");
    let target = store(&session, "target");
    let refused = session
        .run_release_gate(
            &GateRunRequestDto {
                snapshot_id: target.clone(),
                baseline_snapshot_id: Some(target),
            },
            "op-gate",
        )
        .expect_err("a pair of one is refused");
    assert_eq!(refused.code, "ERR-DIFF-5001");
    assert_eq!(refused.details, None);
}

#[test]
fn a_snapshot_id_that_was_never_stored_is_a_typed_failure() {
    let (_db, session) = session("no-build");
    let refused = session
        .run_release_gate(
            &GateRunRequestDto {
                snapshot_id: "snap-never-analyzed".to_owned(),
                baseline_snapshot_id: None,
            },
            "op-gate",
        )
        .expect_err("an unknown build is not an empty run");
    assert_eq!(refused.code, "ERR-STORAGE-4005");
    assert!(refused.message.contains("snap-never-analyzed"));
}

// --------------------------------------------------------------------------- the tables

#[test]
fn a_budget_row_carries_the_state_of_its_finding_and_only_a_verdict_has_a_margin() {
    let config = Config::permissive().flash_budget(1).ram_budget(1 << 40);
    let project = TempProject::new("budget", &config);
    let (_db, session) = session("budget");
    let target = store(&session, "target");
    load(&session, &project);
    let gate = run(&session, &target, None);

    let tables = gate
        .tables
        .as_ref()
        .expect("a fresh run carries its tables");
    assert_eq!(tables.budgets.len(), 2, "one row per side, always");
    for row in &tables.budgets {
        let found = finding(&gate, &row.rule_id);
        // The table never disagrees with the rule it belongs to (§52).
        assert_eq!(&row.state, &found.state);
        assert_eq!(&row.effective_severity, &found.effective_severity);
        assert_eq!(row.budget_bytes.is_some(), row.state != "N/A");
        let deterministic = row.state == "PASS" || row.state == "BLOCK";
        assert_eq!(
            row.headroom_bytes.is_some() || row.over_bytes.is_some(),
            deterministic,
            "a margin exists exactly when Core reached a verdict: {row:?}"
        );
        assert!(
            !(row.headroom_bytes.is_some() && row.over_bytes.is_some()),
            "one row cannot be both under and over its budget"
        );
    }
    // The fixture pair measures exactly these two totals (P2, prompt §23).
    let flash = tables
        .budgets
        .iter()
        .find(|row| row.side == "flash")
        .expect("the flash row");
    assert_eq!(flash.budget_bytes, Some(1));
    assert_eq!(flash.actual_bytes, Some(376));
    assert!(flash.exact && flash.admissible);
    // A footprint this size cannot fit a one-byte budget, so the row is a block with an excess.
    assert_eq!(flash.state, "BLOCK");
    assert_eq!(flash.over_bytes, Some(375));
    assert_eq!(flash.headroom_bytes, None);

    let ram = tables
        .budgets
        .iter()
        .find(|row| row.side == "ram")
        .expect("the ram row");
    assert_eq!(ram.actual_bytes, Some(76));
    assert_eq!(ram.state, "PASS");
    assert_eq!(ram.headroom_bytes, Some((1 << 40) - 76));
    assert_eq!(ram.over_bytes, None);
}

#[test]
fn an_unconfigured_budget_reads_as_n_a_and_shows_no_margin() {
    let project = TempProject::new("no-budget", &Config::permissive());
    let (_db, session) = session("no-budget");
    let target = store(&session, "target");
    load(&session, &project);
    let gate = run(&session, &target, None);

    for row in &gate.tables.as_ref().expect("tables").budgets {
        assert_eq!(row.state, "N/A", "{}", row.rule_id);
        assert_eq!(row.effective_severity, "PASS", "N/A is neutral (ADR-0023)");
        assert_eq!(row.budget_bytes, None);
        assert_eq!(row.headroom_bytes, None);
        assert_eq!(row.over_bytes, None);
        // The measured total is still a fact the reader is entitled to, with no verdict on it.
        assert!(row.actual_bytes.is_some());
    }
}

#[test]
fn a_growth_row_moves_the_delta_core_computed_and_never_a_second_one() {
    let config = Config::permissive().flash_growth(100);
    let project = TempProject::new("growth", &config);
    let (_db, session) = session("growth");
    let base = store(&session, "base");
    let target = store(&session, "target");
    load(&session, &project);

    let gate = run(&session, &target, Some(&base));
    let tables = gate
        .tables
        .as_ref()
        .expect("a fresh run carries its tables");
    // Only the side with a threshold has a row: a second row would imply a rule nobody configured.
    assert_eq!(tables.growth.len(), 1);
    let row = &tables.growth[0];
    assert_eq!(row.side, "flash");
    assert_eq!(row.threshold_bytes, Some(100));
    // The fixture pair is designed to grow by exactly this much (P2, prompt §23).
    assert_eq!(row.old_bytes, Some(256));
    assert_eq!(row.new_bytes, Some(376));
    assert_eq!(row.delta_bytes, Some(120));
    assert_eq!(row.comparability, "exact");
    assert_eq!(row.state, "REVIEW");
    // The budget table's own reading of the same total is the growth row's new side: one figure,
    // moved to two places, never recomputed in either.
    assert_eq!(
        row.new_bytes,
        tables
            .budgets
            .iter()
            .find(|budget| budget.side == "flash")
            .expect("the flash row")
            .actual_bytes
    );
    assert_eq!(finding(&gate, "diff.growth").state, "REVIEW");
    assert!(
        finding(&gate, "diff.growth")
            .evidence_refs
            .iter()
            .any(|reference| reference.starts_with("diff:")),
        "the growth finding must point at the diff it read"
    );
}

#[test]
fn a_growth_threshold_without_a_baseline_is_unknown_and_shows_no_number() {
    let config = Config::permissive().ram_growth(1);
    let project = TempProject::new("no-baseline", &config);
    let (_db, session) = session("no-baseline");
    let target = store(&session, "target");
    load(&session, &project);

    let gate = run(&session, &target, None);
    // §44: the run still happens, and the rule reports what is missing rather than a zero.
    assert_eq!(finding(&gate, "diff.growth").state, "UNKNOWN");
    let row = &gate.tables.as_ref().expect("tables").growth[0];
    assert_eq!(row.side, "ram");
    assert_eq!(row.state, "UNKNOWN");
    assert!(row.old_bytes.is_none() && row.new_bytes.is_none() && row.delta_bytes.is_none());
    assert_eq!(row.threshold_bytes, Some(1));
    assert_eq!(row.comparability, "no baseline");
    assert!(row.reason.as_ref().expect("a reason").contains("baseline"));
}

#[test]
fn a_required_kind_the_build_does_not_hold_is_listed_without_an_invented_hash() {
    let config = Config::permissive().required(&["elf", "bin"]);
    let project = TempProject::new("bin", &config);
    let (_db, session) = session("bin");
    let target = store(&session, "target");
    load(&session, &project);
    let gate = run(&session, &target, None);

    let bin = gate
        .tables
        .as_ref()
        .expect("tables")
        .artifacts
        .iter()
        .find(|row| row.kind == "bin")
        .expect("the required BIN row");
    assert!(bin.required && !bin.present);
    assert!(bin.sha256.is_none() && bin.byte_size.is_none());
    assert_eq!(finding(&gate, "artifacts.required").state, "BLOCK");
    assert!(finding(&gate, "artifacts.required").summary.contains("bin"));
    // The MAP the build does hold is listed too, marked as not required (§51).
    let map = gate
        .tables
        .as_ref()
        .expect("tables")
        .artifacts
        .iter()
        .find(|row| row.kind == "map")
        .expect("the MAP row");
    assert!(!map.required && map.present);
}

#[test]
fn release_notes_are_shown_by_relative_path_and_never_by_root() {
    let project = TempProject::new("notes", &Config::permissive().notes_required()).with_notes();
    let (_db, session) = session("notes");
    let target = store(&session, "target");
    load(&session, &project);
    let gate = run(&session, &target, None);

    let notes = &gate.tables.as_ref().expect("tables").notes;
    assert!(notes.required);
    assert_eq!(
        notes.relative_path.as_deref(),
        Some("docs/RELEASE_NOTES.md")
    );
    assert_eq!(notes.present, Some(true));
    assert_eq!(notes.sha256.as_deref().map(str::len), Some(64));
    assert_eq!(finding(&gate, "release.notes").state, "PASS");
    assert!(
        finding(&gate, "release.notes")
            .evidence_refs
            .iter()
            .any(|reference| reference == "file:docs/RELEASE_NOTES.md")
    );
}

#[test]
fn a_missing_release_notes_file_blocks_and_says_where_to_put_it() {
    let project = TempProject::new("no-notes", &Config::permissive().notes_required());
    let (_db, session) = session("no-notes");
    let target = store(&session, "target");
    load(&session, &project);
    let gate = run(&session, &target, None);

    let found = finding(&gate, "release.notes");
    assert_eq!(found.state, "BLOCK");
    assert_eq!(
        gate.tables.as_ref().expect("tables").notes.present,
        Some(false)
    );
    assert!(found.summary.contains("docs/RELEASE_NOTES.md"));
    assert!(
        found
            .remediation
            .as_ref()
            .expect("a next step")
            .contains("release_notes_path")
    );
    // A block is never acceptable as a review (prompt §48).
    assert!(!found.acceptable);
}

// --------------------------------------------------------------------------- workspace wording

#[test]
fn a_clean_workspace_and_a_dirty_one_are_described_as_the_workspace() {
    let config = Config::permissive().clean_git_required();
    let project = TempProject::new("git-clean", &config).with_repository();
    let (_db, session) = session("git-clean");
    let target = store(&session, "target");
    load(&session, &project);
    let gate = run(&session, &target, None);
    assert!(gate.git.available);
    assert_eq!(gate.git.dirty, Some(false));
    assert_eq!(gate.git.summary, "Workspace HEAD");
    assert_eq!(gate.git.head_commit.as_deref().map(str::len), Some(40));
    assert_eq!(finding(&gate, "git.clean").state, "PASS");
    // §50: nothing may read as a claim that the artifact was built from this commit.
    let text = json(&gate).to_string();
    for forbidden in ["built from", "Built from", "artifact commit"] {
        assert!(
            !text.contains(forbidden),
            "the payload over-claimed: {forbidden}"
        );
    }

    project.dirty_workspace();
    let dirty = run(&session, &target, None);
    assert_eq!(dirty.git.dirty, Some(true));
    assert_eq!(dirty.git.summary, "Workspace dirty");
    assert_eq!(finding(&dirty, "git.clean").state, "BLOCK");
    // A new attempt is a new run, and the earlier one is still identifiable.
    assert_ne!(dirty.run_id, gate.run_id);
    assert_eq!(
        finding(&dirty, "git.clean").state,
        "BLOCK",
        "the earlier run's record is untouched"
    );
    let earlier = session
        .gate_run(&gate.run_id, "op-read")
        .expect("readable")
        .expect("stored");
    assert_eq!(finding(&earlier, "git.clean").state, "PASS");
}

#[test]
fn a_project_that_is_not_a_repository_reports_unavailable_rather_than_clean() {
    let config = Config::permissive().clean_git_required();
    let project = TempProject::new("not-a-repo", &config);
    let (_db, session) = session("not-a-repo");
    let target = store(&session, "target");
    load(&session, &project);
    let gate = run(&session, &target, None);

    assert!(!gate.git.available);
    assert_eq!(gate.git.summary, "No workspace facts were observed");
    assert_eq!(gate.git.dirty, None);
    assert!(
        gate.git
            .reason
            .as_ref()
            .expect("a reason")
            .contains("repository")
    );
    let found = finding(&gate, "git.clean");
    assert_eq!(found.state, "UNKNOWN");
    // The default disposition for this rule asks a human; it does not block and it is not a pass.
    assert_eq!(found.effective_severity, "REVIEW");
    assert!(
        !found.acceptable,
        "an UNKNOWN is never accepted away (ADR-0023)"
    );
}

// --------------------------------------------------------------------------- acceptance

#[test]
fn an_accepted_review_stays_review_and_moves_only_the_aggregate() {
    let config = Config::permissive().flash_growth(100);
    let project = TempProject::new("accept", &config);
    let (_db, session) = session("accept");
    let base = store(&session, "base");
    let target = store(&session, "target");
    load(&session, &project);
    let gate = run(&session, &target, Some(&base));

    let growth = finding(&gate, "diff.growth");
    assert_eq!(growth.state, "REVIEW");
    assert!(growth.acceptable);
    assert_eq!(gate.overall_effective_severity, "REVIEW");
    assert_eq!(gate.disposition_effective_severity, "REVIEW");
    // Everything else in this run passed or did not apply, so the aggregate has one thing to move on.
    for row in &gate.findings {
        if row.rule_id != "diff.growth" {
            assert!(
                row.state == "PASS" || row.state == "N/A",
                "{} answered {}",
                row.rule_id,
                row.state
            );
        }
    }

    let outcome = session
        .accept_gate_review(
            &AcceptReviewRequestDto {
                run_id: gate.run_id.clone(),
                finding_id: growth.id.clone(),
                actor: "rosa".to_owned(),
                reason: "the bootloader grew on purpose".to_owned(),
            },
            "op-accept",
        )
        .expect("the acceptance is recorded");
    assert_eq!(outcome.run_id, gate.run_id);
    assert_eq!(outcome.finding_id, growth.id);
    assert_eq!(outcome.state, "REVIEW", "the finding keeps its own state");
    assert_eq!(outcome.disposition_effective_severity, "PASS");
    assert_eq!(outcome.acceptance.actor, "rosa");
    assert_eq!(outcome.acceptance.original_state, "REVIEW");
    assert!(outcome.acceptance.accepted_at.contains('T'));

    // Read back from the database: the row is unchanged and the record beside it is not.
    let stored = session
        .gate_run(&gate.run_id, "op-read")
        .expect("the run is stored")
        .expect("the id this session issued is in history");
    let row = finding(&stored, "diff.growth");
    assert_eq!(row.state, "REVIEW");
    assert_eq!(row.effective_severity, "REVIEW");
    assert!(!row.acceptable, "an accepted review is not offered again");
    let acceptance = row.acceptance.as_ref().expect("the acceptance is readable");
    assert_eq!(acceptance.reason, "the bootloader grew on purpose");
    assert_eq!(stored.disposition_effective_severity, "PASS");
    assert_eq!(
        stored.overall_effective_severity, "REVIEW",
        "the un-dispositioned aggregate is still the run's own answer"
    );
    assert_eq!(
        stored.counts.review, 1,
        "the counts state facts, not dispositions"
    );
}

#[test]
fn only_a_review_can_be_accepted() {
    let project = TempProject::new("only-review", &Config::permissive());
    let (_db, session) = session("only-review");
    let target = store(&session, "target");
    load(&session, &project);
    let gate = run(&session, &target, None);

    for rule in [
        "git.clean",
        "artifacts.required",
        "artifacts.hashes",
        "release.notes",
        "evidence.unknown_review",
    ] {
        let found = finding(&gate, rule);
        if found.state == "REVIEW" {
            continue;
        }
        assert!(!found.acceptable, "{rule} must not offer an acceptance");
        let refused = session
            .accept_gate_review(
                &AcceptReviewRequestDto {
                    run_id: gate.run_id.clone(),
                    finding_id: found.id.clone(),
                    actor: "rosa".to_owned(),
                    reason: "trying to accept a non-review".to_owned(),
                },
                "op-accept",
            )
            .expect_err("storage refuses it");
        assert_eq!(
            refused.code, "ERR-STORAGE-4008",
            "{rule} is {}",
            found.state
        );
        assert!(refused.message.contains(&found.state));
    }
}

#[test]
fn an_acceptance_needs_a_name_and_a_reason_and_a_second_one_is_refused() {
    let config = Config::permissive().flash_growth(100);
    let project = TempProject::new("twice", &config);
    let (_db, session) = session("twice");
    let base = store(&session, "base");
    let target = store(&session, "target");
    load(&session, &project);
    let gate = run(&session, &target, Some(&base));
    let growth = finding(&gate, "diff.growth");

    for (actor, reason, finding_id, expected) in [
        ("", "a reason", growth.id.as_str(), "ERR-STORAGE-4010"),
        ("rosa", "   ", growth.id.as_str(), "ERR-STORAGE-4010"),
        ("rosa", "a reason", "f-nope", "ERR-STORAGE-4007"),
    ] {
        let refused = session
            .accept_gate_review(
                &AcceptReviewRequestDto {
                    run_id: gate.run_id.clone(),
                    finding_id: finding_id.to_owned(),
                    actor: actor.to_owned(),
                    reason: reason.to_owned(),
                },
                "op-accept",
            )
            .expect_err("refused");
        assert_eq!(
            refused.code, expected,
            "{actor:?} / {reason:?} / {finding_id}"
        );
    }

    session
        .accept_gate_review(
            &AcceptReviewRequestDto {
                run_id: gate.run_id.clone(),
                finding_id: growth.id.clone(),
                actor: "rosa".to_owned(),
                reason: "the growth is intended".to_owned(),
            },
            "op-accept",
        )
        .expect("the first acceptance lands");
    let twice = session
        .accept_gate_review(
            &AcceptReviewRequestDto {
                run_id: gate.run_id.clone(),
                finding_id: growth.id.clone(),
                actor: "someone-else".to_owned(),
                reason: "a second opinion".to_owned(),
            },
            "op-accept",
        )
        .expect_err("one review is accepted once");
    assert_eq!(twice.code, "ERR-STORAGE-4009");
    assert!(
        twice.message.contains("rosa"),
        "it names who already answered"
    );
    // The first record is the one still standing.
    let stored = session
        .gate_run(&gate.run_id, "op-read")
        .expect("readable")
        .expect("stored");
    assert_eq!(
        finding(&stored, "diff.growth")
            .acceptance
            .as_ref()
            .expect("the acceptance")
            .actor,
        "rosa"
    );
}

#[test]
fn a_refused_acceptance_leaves_the_run_readable_and_unchanged() {
    let project = TempProject::new("refused", &Config::permissive());
    let (_db, session) = session("refused");
    let target = store(&session, "target");
    load(&session, &project);
    let gate = run(&session, &target, None);

    let refused = session
        .accept_gate_review(
            &AcceptReviewRequestDto {
                run_id: gate.run_id.clone(),
                finding_id: finding(&gate, "artifacts.hashes").id.clone(),
                actor: "  ".to_owned(),
                reason: String::new(),
            },
            "op-accept",
        )
        .expect_err("an empty actor is refused");
    assert_eq!(refused.code, "ERR-STORAGE-4010");

    let after = session
        .gate_run(&gate.run_id, "op-read")
        .expect("the run is still readable")
        .expect("and still stored");
    // §49: a rejected acceptance does not invalidate the run it was attempted against.
    assert_eq!(after.run_id, gate.run_id);
    assert_eq!(after.findings.len(), gate.findings.len());
    assert_eq!(after.policy_sha256, gate.policy_sha256);
    assert_eq!(
        after.disposition_effective_severity,
        gate.disposition_effective_severity
    );
    assert!(
        after.findings.iter().all(|row| row.acceptance.is_none()),
        "nothing was written"
    );
}

#[test]
fn an_id_that_was_never_stored_is_no_record_rather_than_a_failure() {
    let (_db, session) = session("unknown-id");
    let missing = session
        .gate_run(
            "gate-0000000000000000000000000000000000000000000000000000000000000000",
            "op-read",
        )
        .expect("reading history is not a failure");
    assert!(missing.is_none());
}

#[test]
fn a_stored_run_survives_a_restart_because_the_record_is_in_sqlite() {
    let config = Config::permissive().flash_growth(100);
    let project = TempProject::new("restart", &config);
    let (db, session) = session("restart");
    let base = store(&session, "base");
    let target = store(&session, "target");
    load(&session, &project);
    let gate = run(&session, &target, Some(&base));
    session
        .accept_gate_review(
            &AcceptReviewRequestDto {
                run_id: gate.run_id.clone(),
                finding_id: finding(&gate, "diff.growth").id.clone(),
                actor: "rosa".to_owned(),
                reason: "intended growth".to_owned(),
            },
            "op-accept",
        )
        .expect("the acceptance is written");
    drop(session);

    // A fresh Session over the same database, with no policy loaded: the run and its audit row are
    // still there, because they were never held only in memory (prompt §49).
    let reopened = Session::open(FixtureCatalog::from_environment(), &db.0).expect("reopens");
    let stored = reopened
        .gate_run(&gate.run_id, "op-read")
        .expect("readable")
        .expect("stored");
    assert_eq!(stored.findings.len(), 10);
    assert_eq!(stored.disposition_effective_severity, "PASS");
    assert!(
        stored.record_note.is_some(),
        "a record says what a record is"
    );
    assert!(
        stored.tables.is_none(),
        "no table is rebuilt against a policy that is not loaded"
    );
    assert!(stored.policy.is_none());
    assert!(stored.project_name.is_none());
    assert!(
        stored.policy_source.contains("read back from history"),
        "{}",
        stored.policy_source
    );
    assert_eq!(stored.policy_sha256, gate.policy_sha256);
    assert_eq!(stored.snapshot_id, gate.snapshot_id);
    assert_eq!(stored.git.summary, "Workspace not re-observed");
    assert_eq!(
        finding(&stored, "diff.growth")
            .acceptance
            .as_ref()
            .expect("the audit row")
            .actor,
        "rosa"
    );
}

// --------------------------------------------------------------------------- the config file

#[test]
fn a_cancelled_dialog_is_a_decision_and_not_a_failure() {
    let project = TempProject::new("cancel", &Config::permissive());
    let (_db, session) = session("cancel");
    let first = load(&session, &project);
    assert_eq!(first.project_name, "brake-node");
    assert_eq!(first.config_file_name, "firmwaresight.toml");
    assert_eq!(first.config_schema_version, 1);

    let cancelled = session
        .load_project_from(None, "op-cancel")
        .expect("cancel is not an error");
    assert!(cancelled.is_none());
    // The page keeps the project it had (prompt §40).
    assert!(session.has_project());
    let context = session
        .save_policy_in_project(&first.policy, "op-save")
        .expect("saving to the kept project still works")
        .expect("a project is loaded, so there is a file to write to");
    assert_eq!(context.project_name, "brake-node");
}

#[test]
fn a_config_this_build_cannot_read_is_refused_without_losing_the_one_in_memory() {
    let good = TempProject::new("bad-config", &Config::permissive());
    let broken = TempProject::with_text("broken", "schema_version = 1\n[project]\nname = ");
    let (_db, session) = session("bad-config");
    load(&session, &good);

    let refused = session
        .load_project_from(Some(broken.config_path()), "op-open")
        .expect_err("a malformed config is a typed failure");
    assert!(
        refused.code.starts_with("ERR-CONFIG-7"),
        "unexpected code {}",
        refused.code
    );
    assert!(refused.message.contains("firmwaresight.toml"));
    assert!(refused.remediation.is_some());
    assert!(
        refused.details.is_none(),
        "the message is the whole statement"
    );
    // The previously loaded project is still the policy the Release page runs against.
    assert!(session.has_project());
    let target = store(&session, "target");
    assert_eq!(
        run(&session, &target, None).project_name.as_deref(),
        Some("brake-node")
    );
}

#[test]
fn unknown_keys_are_listed_and_a_save_that_would_drop_them_is_refused() {
    let config = Config::permissive().with_unknown_release_key("expect_nothing_at_all = true");
    let project = TempProject::new("unknown-keys", &config);
    let before = project.read_config();
    let (_db, session) = session("unknown-keys");
    let context = load(&session, &project);

    assert!(
        context
            .unknown_keys
            .iter()
            .any(|key| key.contains("expect_nothing_at_all")),
        "the unknown key must be named: {:?}",
        context.unknown_keys
    );
    let refused = session
        .save_policy_in_project(&context.policy, "op-save")
        .expect_err("the save is refused");
    assert_eq!(refused.code, "ERR-CONFIG-7007");
    assert_eq!(
        project.read_config(),
        before,
        "a refused save rewrites nothing"
    );
    // The refused config is still the session's policy, and a run still judges against it.
    assert!(session.has_project());
    let target = store(&session, "target");
    assert_eq!(
        run(&session, &target, None).policy_source,
        "firmwaresight.toml"
    );
}

#[test]
fn a_policy_saved_with_no_project_creates_the_file_the_loader_reads() {
    let dir = temp_name("create", "");
    std::fs::create_dir_all(&dir).expect("a temporary folder");
    let path = dir.join("firmwaresight.toml");
    let (_db, session) = session("create");

    // The form an operator touches before any config exists states nothing, and Rust supplies the
    // documented defaults rather than the screen guessing them (prompt §41).
    assert!(!session.has_project());
    let context = session
        .save_policy_as_new(Some(path.clone()), &unstated("first-release"), "op-save")
        .expect("the config is written and read back")
        .expect("a chosen path is not the cancel path");
    assert_eq!(context.project_name, "first-release");
    assert!(path.exists());
    assert!(
        std::fs::read_to_string(&path)
            .expect("the written config is readable")
            .contains("name = \"first-release\""),
        "the written file names the project"
    );

    // What was written resolves to the documented default policy, so the screen and the file agree.
    assert_eq!(context.policy_sha256, policy_sha256(&GatePolicy::default()));
    assert_eq!(context.policy.require_clean_git, Some(true));
    assert_eq!(
        context.policy.release_notes_path.as_deref(),
        Some("RELEASE_NOTES.md")
    );
    assert_eq!(
        words(
            context
                .policy
                .required_artifact_kinds
                .as_deref()
                .unwrap_or(&[])
        ),
        ["elf"]
    );
    // Every on-unknown disposition is stated once the file exists, because a resolved policy has no
    // unset fields left to interpret.
    assert_eq!(
        context.policy.on_unknown.flash_budget,
        Some(firmwaresight_desktop::ipc::UnknownDispositionDto::Block)
    );

    let target = store(&session, "target");
    let gate = run(&session, &target, None);
    assert_eq!(gate.policy_source, "firmwaresight.toml");
    assert_eq!(gate.project_name.as_deref(), Some("first-release"));
    assert_eq!(gate.policy_sha256, context.policy_sha256);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_cancelled_save_writes_no_file_and_leaves_the_session_without_a_project() {
    let (_db, session) = session("cancel-save");
    let cancelled = session
        .save_policy_as_new(None, &unstated("never-written"), "op-save")
        .expect("cancel is not an error");
    assert!(cancelled.is_none());
    assert!(
        !session.has_project(),
        "nothing was chosen, so nothing was loaded"
    );

    // A run without a project is still an honest run: it names the default policy instead (§40).
    let target = store(&session, "target");
    let gate = run(&session, &target, None);
    assert!(gate.policy_source.contains("default"));
}
