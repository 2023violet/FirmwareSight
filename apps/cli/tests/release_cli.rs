//! The shipped `fwsight release prepare` surface: which code a release run earns, what it writes, what it
//! refuses to write, and what it never touches (prompt §33, §34, §35, §56).
//!
//! These run the real binary for the same reason `gate_cli.rs` does — what is under test is a process: its
//! exit code, the split between stdout and stderr, and the bytes that end up in a release owner's folder.
//!
//! Each release subject is a throwaway Git repository holding a `firmwaresight.toml` and its Release Notes,
//! committed and tagged so the version policy resolves, with the build it ships named absolutely. The absolute
//! path is deliberate: §22 re-reads a shipped file from the path the analysis recorded, and a CLI run from
//! this repository's root has no project-relative way to name `fixtures/…`. FirmwareSight's own repository is
//! never the release subject — it is dirty or clean for reasons that have nothing to do with a release.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The engine, called directly from the test process: a release that ships attachments has to verify against
/// bytes the CLI's own exit code cannot vouch for, so the proof does not use the runner that produced them.
use firmwaresight_project::{fingerprint, verify_bundle};

const TARGET_ELF: &str = "fixtures/elf/p2-diff/target/firmware.elf";
const TARGET_MAP: &str = "fixtures/elf/p2-diff/target/firmware.map";
const BASE_ELF: &str = "fixtures/elf/p2-diff/base/firmware.elf";
const NOTES: &str = "# 1.4.2\n\n- the modem driver is now in the image\n";

/// A release the Gate can pass on its own terms: provenance required, notes required, and a version policy
/// whose pattern the workspace tag satisfies. No budget and no growth threshold, so those rules answer `N/A`
/// rather than passing silently.
const PASSING: &str = r#"schema_version = 1

[project]
name = "motor-controller"

[artifacts]
required = ["elf", "map"]

[version]
source = "git_tag"
pattern = '^v(?P<version>\d+\.\d+\.\d+)$'

[release]
require_clean_git = true
require_release_notes = true
release_notes_path = "RELEASE_NOTES.md"
expected_version = "1.4.2"
"#;

/// `D-N10-1`. The same release, its notes reached through a `..` that comes straight back inside the
/// project: `config::require_project_relative` has accepted this spelling since `f1516b8`, and it names the
/// very file `PASSING` names. Cleanliness is not required here because one case writes an untracked file
/// into the project on purpose, and the workspace-state rule is a separate fact from the notes rule.
const CLIMBING_NOTES: &str = r#"schema_version = 1

[project]
name = "motor-controller"

[artifacts]
required = ["elf", "map"]

[version]
source = "git_tag"
pattern = '^v(?P<version>\d+\.\d+\.\d+)$'

[release]
require_clean_git = false
require_release_notes = true
release_notes_path = "docs/../RELEASE_NOTES.md"
expected_version = "1.4.2"
"#;

/// The same policy with a growth threshold the fixture pair is over: one REVIEW finding, which the command
/// line has no way to accept (§33).
const REVIEWING: &str = r#"schema_version = 1

[project]
name = "motor-controller"

[artifacts]
required = ["elf", "map"]

[version]
source = "git_tag"
pattern = '^v(?P<version>\d+\.\d+\.\d+)$'

[release]
require_clean_git = true
require_release_notes = true
release_notes_path = "RELEASE_NOTES.md"
expected_version = "1.4.2"

[diff]
flash_growth_review_bytes = 100
"#;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|parent| parent.parent())
        .expect("apps/cli lives at <root>/apps/cli")
        .to_path_buf()
}

/// One release subject, the folder it exports into, and a path that is a regular file where a folder would
/// have to be.
struct World {
    project: PathBuf,
    out: PathBuf,
    blocked: PathBuf,
}

impl World {
    /// A tagged, clean repository holding `config` and its Release Notes, so every rule the policy requires
    /// has a real answer. The notes are written before the commit so they are tracked, not merely present.
    fn new(label: &str, config: &str) -> Self {
        let scratch = std::env::temp_dir().join(format!(
            "fwsight-p4-{label}-{}-{:?}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|value| value.as_nanos())
                .unwrap_or(0)
        ));
        let project = scratch.join("project");
        std::fs::create_dir_all(&project).expect("a temporary project directory");
        std::fs::write(project.join("firmwaresight.toml"), config).expect("a temporary config");
        std::fs::write(project.join("RELEASE_NOTES.md"), NOTES).expect("the notes are written");
        init_repository(&project);
        commit_all(&project, "release candidate");
        git(&["tag", "v1.4.2"], &project);

        let blocked = scratch.join("not-a-folder");
        std::fs::write(&blocked, "a file where a folder should be\n")
            .expect("a blocked destination");
        Self {
            project,
            out: scratch.join("out"),
            blocked,
        }
    }

    /// The same subject with its notes deleted and the deletion committed, which is the BLOCK the policy
    /// cannot answer around. The tag is moved onto the new HEAD so the version rule keeps passing and the
    /// notes are the only thing that changed.
    fn without_notes(self) -> Self {
        std::fs::remove_file(self.project.join("RELEASE_NOTES.md")).expect("the notes are removed");
        commit_all(&self.project, "the notes are gone");
        git(&["tag", "-f", "v1.4.2"], &self.project);
        self
    }

    /// Replace `firmwaresight.toml` with something this build must refuse, after the repository was built.
    fn with_broken_config(&self) {
        std::fs::write(
            self.project.join("firmwaresight.toml"),
            "schema_version = 1\n[project]\nname = 7\n",
        )
        .expect("the config is replaced");
    }

    /// §33's frozen invocation, with the two required paths filled in and the export folder left where the
    /// test put it.
    fn prepare(&self, extra: &[&str]) -> Output {
        self.export_to(&self.out.clone(), extra)
    }

    /// The same invocation exported into `out` instead, for the runs whose destination is the subject.
    fn export_to(&self, out: &Path, extra: &[&str]) -> Output {
        let mut args = vec![
            "release".to_owned(),
            "prepare".to_owned(),
            "--project".to_owned(),
            self.project.to_string_lossy().into_owned(),
            "--artifact".to_owned(),
            absolute(TARGET_ELF),
            "--map".to_owned(),
            absolute(TARGET_MAP),
            "--out".to_owned(),
            out.to_string_lossy().into_owned(),
        ];
        args.extend(extra.iter().map(|arg| (*arg).to_owned()));
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        run(&refs)
    }

    /// The bundle directory the last successful run wrote into `out`.
    fn published(&self) -> PathBuf {
        only_bundle(&self.out)
    }
}

impl Drop for World {
    fn drop(&mut self) {
        let scratch = self
            .project
            .parent()
            .unwrap_or(self.project.as_path())
            .to_path_buf();
        let _ = std::fs::remove_dir_all(&scratch);
    }
}

/// The one directory under `parent`, which is where a successful release put its bundle. The name is
/// proposed by the engine (§29), so no test here spells it out.
fn only_bundle(parent: &Path) -> PathBuf {
    let names: Vec<String> = std::fs::read_dir(parent)
        .unwrap_or_else(|error| panic!("{} is not readable: {error}", parent.display()))
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        names.len(),
        1,
        "one release writes one bundle directory, found {names:?}"
    );
    parent.join(&names[0])
}

fn absolute(relative: &str) -> String {
    repo_root().join(relative).to_string_lossy().into_owned()
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

fn code(output: &Output) -> Option<i32> {
    output.status.code()
}

/// The one JSON document a failing run put on stdout, with its `--json` envelope read as a document.
fn envelope(output: &Output) -> serde_json::Value {
    serde_json::from_str(&text(output)).unwrap_or_else(|error| {
        panic!(
            "stdout must hold exactly one document under --json ({error}):\n{}",
            text(output)
        )
    })
}

/// Every file under one root, relative and sorted, with `/` separators.
fn tree(root: &Path) -> Vec<String> {
    let mut names = Vec::new();
    collect(root, root, &mut names);
    names.sort();
    names
}

fn collect(root: &Path, directory: &Path, out: &mut Vec<String>) {
    for entry in std::fs::read_dir(directory).expect("the directory is readable") {
        let entry = entry.expect("a readable directory entry");
        let relative = entry
            .path()
            .strip_prefix(root)
            .expect("the entry is inside the tree")
            .to_string_lossy()
            .replace('\\', "/");
        if entry.path().is_dir() {
            collect(root, &entry.path(), out);
        } else {
            out.push(relative);
        }
    }
}

fn git(args: &[&str], dir: &Path) {
    let status = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect(
            "the release provenance tests run against a real repository, so git must be installed",
        );
    assert!(
        status.status.success(),
        "`git {}` failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&status.stderr)
    );
}

/// A throwaway repository with an identity configured, so a commit does not depend on the machine's global
/// git config.
fn init_repository(dir: &Path) {
    git(&["init", "-q"], dir);
    git(&["config", "user.email", "release@example.invalid"], dir);
    git(&["config", "user.name", "Release Owner"], dir);
}

fn commit_all(dir: &Path, message: &str) {
    git(&["add", "-A"], dir);
    git(&["commit", "-q", "-m", message], dir);
}

/// True when no export happened: either the folder was never created or it holds nothing. §31's promise is
/// that a refused run leaves no partial bundle, and the engine stages in a sibling that it discards.
fn wrote_nothing(out: &Path) -> bool {
    !std::fs::exists(out).unwrap_or(false) || tree(out).is_empty()
}

// --------------------------------------------------------------------------- the passing release

#[test]
fn a_clean_passing_release_writes_the_canonical_bundle_and_exits_zero() {
    let world = World::new("pass", PASSING);
    let output = world.prepare(&[]);
    assert_eq!(code(&output), Some(0), "{}", errors(&output));

    let bundle = world.published();
    assert_eq!(
        tree(&bundle),
        vec![
            "SHA256SUMS",
            "accepted-reviews.json",
            "analysis.json",
            "artifacts/firmware.elf",
            "artifacts/firmware.map",
            "gate-results.json",
            "release-manifest.json",
            "release-notes.md",
            "release-report.html",
        ],
        "§7's layout, and no baseline means no diff.json"
    );
    let stdout = text(&output);
    assert!(stdout.contains("Release bundle written"), "{stdout}");

    // The engine's own verifier over the directory the CLI wrote: both indexes, the five contracts, and the
    // release id re-derived from the bytes (§44).
    let verification = firmwaresight_project::verify_bundle(&bundle)
        .unwrap_or_else(|error| panic!("the bundle the CLI wrote does not verify: {error}"));
    assert_eq!(verification.artifact_count, 2);
    assert!(!verification.carries_comparison);
    assert!(verification.carries_release_notes);
    assert!(
        stdout.contains(&verification.release_id),
        "the summary names the release the bundle says it is"
    );
    assert!(
        stdout.contains("artifacts   2"),
        "the summary counts what the bundle holds: {stdout}"
    );
}

#[test]
fn a_baseline_adds_the_comparison_the_bundle_ships() {
    let world = World::new("baseline", PASSING);
    let base = absolute(BASE_ELF);
    let output = world.prepare(&["--baseline", base.as_str()]);
    assert_eq!(code(&output), Some(0), "{}", errors(&output));

    let bundle = world.published();
    assert!(
        tree(&bundle).contains(&"diff.json".to_owned()),
        "{:?}",
        tree(&bundle)
    );
    assert!(text(&output).contains("comparison  shipped"));
    assert!(
        firmwaresight_project::verify_bundle(&bundle).is_ok(),
        "a bundle with a comparison verifies the same way"
    );
    assert!(errors(&output).contains("diagnostics: baseline firmware.elf"));
}

#[test]
fn json_mode_prints_exactly_the_manifest_the_bundle_carries() {
    let world = World::new("json", PASSING);
    let output = world.prepare(&["--json"]);
    assert_eq!(code(&output), Some(0), "{}", errors(&output));

    let stdout = text(&output);
    assert_eq!(
        stdout.matches('\n').count(),
        1,
        "stdout must carry one compact document, got: {stdout}"
    );
    assert!(stdout.ends_with("}\n"), "{stdout}");
    let document = envelope(&output);
    assert_eq!(document["schema_version"], 1);
    assert_eq!(document["generated_by"]["product"], "FirmwareSight");

    let written = std::fs::read(world.published().join("release-manifest.json"))
        .expect("the bundle carries its manifest");
    assert_eq!(
        stdout.as_bytes(),
        written.as_slice(),
        "§34: the stdout copy and the bundle copy are one document"
    );

    // Everything the run has to say about itself is a diagnostic, and diagnostics are stderr's.
    let stderr = errors(&output);
    assert!(stderr.contains("diagnostics: map"), "{stderr}");
    assert!(stderr.contains("diagnostics: git"), "{stderr}");
    assert!(stderr.contains("operation op-"), "{stderr}");
    assert!(
        !stdout.contains("diagnostics:"),
        "stdout holds one document"
    );
}

#[test]
fn the_release_output_carries_no_host_path_of_the_machine_that_wrote_it() {
    let world = World::new("host-path", PASSING);
    let output = world.prepare(&["--json"]);
    assert_eq!(code(&output), Some(0), "{}", errors(&output));

    let scratch = std::env::temp_dir().to_string_lossy().into_owned();
    let project = world.project.to_string_lossy().into_owned();
    let out = world.out.to_string_lossy().into_owned();
    let root = repo_root().to_string_lossy().into_owned();
    let stdout = text(&output);
    for forbidden in [
        scratch,
        project.clone(),
        out,
        root,
        "\\".to_owned(),
        "RELEASE_NOTES.md".to_owned(),
    ] {
        assert!(
            !stdout.contains(&forbidden),
            "the manifest names {forbidden}, which is a path on this machine rather than in the bundle"
        );
    }
    assert!(stdout.contains("firmware.elf"), "{stdout}");

    for name in [
        "release-manifest.json",
        "analysis.json",
        "gate-results.json",
        "release-report.html",
    ] {
        let bytes = std::fs::read(world.published().join(name)).expect("the file is written");
        let body = String::from_utf8_lossy(&bytes).into_owned();
        assert!(
            !body.contains(&project),
            "`{name}` leaks the project folder"
        );
    }
}

#[test]
fn two_runs_over_the_same_inputs_write_the_same_bytes() {
    // §42 read through the command line: nothing about a release depends on when it was packaged, so the
    // same folder released twice is the same bundle, down to the digest in its manifest.
    let world = World::new("determinism", PASSING);
    let first = world.prepare(&[]);
    assert_eq!(code(&first), Some(0), "{}", errors(&first));
    let first_bundle = world.published();

    let second_parent = world
        .out
        .parent()
        .expect("the export folder sits in the scratch directory")
        .join("second-out");
    let second = world.export_to(&second_parent, &[]);
    assert_eq!(code(&second), Some(0), "{}", errors(&second));
    let second_bundle = only_bundle(&second_parent);

    assert_eq!(
        first_bundle.file_name(),
        second_bundle.file_name(),
        "the same release proposes the same directory name (§29)"
    );
    assert_eq!(
        tree(&second_bundle),
        tree(&first_bundle),
        "the same release writes the same layout"
    );
    for path in tree(&second_bundle) {
        let left = std::fs::read(first_bundle.join(&path)).expect("the first bundle's file");
        let right = std::fs::read(second_bundle.join(&path)).expect("the second bundle's file");
        assert_eq!(left, right, "`{path}` differs between two identical runs");
    }
}

// --------------------------------------------------------------------------- what stops a release

#[test]
fn a_gate_that_aggregates_to_review_writes_nothing_and_exits_four() {
    // §33: the command line holds no review history and accepts nothing interactively, so one unaccepted
    // REVIEW finding ends the run. Accepting it is the desktop's audit action.
    let world = World::new("review", REVIEWING);
    let base = absolute(BASE_ELF);
    let output = world.prepare(&["--baseline", base.as_str(), "--json"]);
    assert_eq!(code(&output), Some(4), "{}", errors(&output));

    let document = envelope(&output);
    assert_eq!(document["code"], "ERR-BUNDLE-6101");
    let message = document["message"].as_str().unwrap_or_default();
    assert!(message.contains("REVIEW"), "{message}");
    assert!(
        message.contains("only a PASS disposition"),
        "the refusal says which disposition it needed: {message}"
    );
    assert!(document["remediation"].is_string());
    assert!(
        wrote_nothing(&world.out),
        "a refused release writes no bundle"
    );
}

#[test]
fn a_gate_that_blocks_writes_nothing_and_exits_five() {
    // Required Release Notes that are not there: Core BLOCKs the provenance rule rather than shipping a
    // release with nothing said about it, and the CLI does not bundle around a BLOCK.
    let world = World::new("block", PASSING).without_notes();
    let output = world.prepare(&["--json"]);
    assert_eq!(code(&output), Some(5), "{}", errors(&output));
    assert_eq!(envelope(&output)["code"], "ERR-BUNDLE-6101");
    assert!(wrote_nothing(&world.out));
}

#[test]
fn a_missing_source_artifact_is_an_import_failure_and_exits_three() {
    let world = World::new("import", PASSING);
    let missing = repo_root()
        .join("fixtures/elf/p2-diff/target/never-built.elf")
        .to_string_lossy()
        .into_owned();
    let project = world.project.to_string_lossy().into_owned();
    let out = world.out.to_string_lossy().into_owned();
    // Spelled out instead of layered onto `prepare`, because clap refuses a repeated `--artifact`: the point
    // of this run is a build that is not there, not a second use of one flag.
    let output = run(&[
        "release",
        "prepare",
        "--project",
        project.as_str(),
        "--artifact",
        missing.as_str(),
        "--out",
        out.as_str(),
        "--json",
    ]);
    assert_eq!(code(&output), Some(3), "{}", errors(&output));
    assert_eq!(envelope(&output)["code"], "ERR-INPUT-0001");
    assert!(wrote_nothing(&world.out));
}

#[test]
fn a_config_this_build_cannot_read_is_a_usage_error_and_exits_two() {
    // The policy is read before anything is analyzed: there is no release to score until it can be read,
    // and a defaulted gap would say otherwise (§34's code 2 covers config).
    let world = World::new("bad-config", PASSING);
    world.with_broken_config();
    let output = world.prepare(&["--json"]);
    assert_eq!(code(&output), Some(2), "{}", errors(&output));
    let document = envelope(&output);
    assert!(
        document["code"]
            .as_str()
            .unwrap_or_default()
            .starts_with("ERR-CONFIG-"),
        "{document}"
    );
    assert!(wrote_nothing(&world.out));
}

// --------------------------------------------------------------------------- the destination (§30, §31)

#[test]
fn an_existing_destination_is_refused_until_the_release_owner_forces_it() {
    let world = World::new("existing", PASSING);
    let first = world.prepare(&[]);
    assert_eq!(code(&first), Some(0), "{}", errors(&first));
    let bundle = world.published();
    let before = std::fs::read(bundle.join("release-manifest.json")).expect("the first manifest");

    let second = world.prepare(&["--json"]);
    assert_eq!(code(&second), Some(6), "{}", errors(&second));
    let document = serde_json::from_str::<serde_json::Value>(&text(&second)).expect("one envelope");
    assert_eq!(document["code"], "ERR-BUNDLE-6106");
    assert_eq!(
        std::fs::read(bundle.join("release-manifest.json")).expect("the untouched manifest"),
        before,
        "a refused export leaves the bundle that was there alone"
    );

    let forced = world.prepare(&["--force"]);
    assert_eq!(code(&forced), Some(0), "{}", errors(&forced));
    assert!(
        errors(&forced).contains("has been replaced"),
        "the run says what it did: {}",
        errors(&forced)
    );
    assert!(
        firmwaresight_project::verify_bundle(&bundle).is_ok(),
        "the replacement is a bundle in its own right"
    );
}

#[test]
fn force_never_replaces_a_directory_firmwaresight_did_not_write() {
    // §31's hardest rule, read off the command line: `--force` authorizes replacing *a bundle*, not clearing
    // a folder that happens to sit where the bundle would go.
    let world = World::new("arbitrary", PASSING);
    let first = world.prepare(&[]);
    assert_eq!(code(&first), Some(0), "{}", errors(&first));
    let name = world
        .published()
        .file_name()
        .expect("the bundle has a directory name")
        .to_string_lossy()
        .into_owned();
    std::fs::remove_dir_all(world.out.join(&name)).expect("the bundle is cleared away");
    std::fs::create_dir_all(world.out.join(&name)).expect("somebody else's folder is in the way");
    let kept = world.out.join(&name).join("tax-returns.txt");
    std::fs::write(&kept, "not a release\n").expect("a file that is not a bundle");

    let output = world.prepare(&["--force", "--json"]);
    assert_eq!(code(&output), Some(6), "{}", errors(&output));
    assert_eq!(envelope(&output)["code"], "ERR-BUNDLE-6107");
    assert_eq!(
        std::fs::read_to_string(&kept).expect("the stranger's file"),
        "not a release\n",
        "the refused run deleted nothing"
    );
    assert_eq!(
        tree(&world.out.join(&name)),
        vec!["tax-returns.txt"],
        "and left the folder as it found it"
    );
}

#[test]
fn a_destination_that_cannot_be_created_is_reported_as_a_bundle_error() {
    let world = World::new("blocked", PASSING);
    let inside = world.blocked.join("dist/release");
    let output = world.export_to(&inside, &["--json"]);
    assert_eq!(code(&output), Some(6), "{}", errors(&output));
    assert_eq!(envelope(&output)["code"], "ERR-BUNDLE-6108");
    assert!(
        !inside.exists(),
        "a run that could not make a folder made no files"
    );
}

// --------------------------------------------------------------------------- the boundaries

#[test]
fn a_release_writes_nothing_into_the_project_it_released() {
    // §38's rule for `gate`, restated for the bundle: the command line persists nothing. There is no stored
    // run to write a release record against, and a project folder that gained a database would say otherwise.
    let world = World::new("no-db", PASSING);
    let before = tree(&world.project);
    let output = world.prepare(&[]);
    assert_eq!(code(&output), Some(0), "{}", errors(&output));

    assert_eq!(
        tree(&world.project),
        before,
        "the project holds its config, its notes and its repository, and nothing the release added"
    );
    assert!(
        !before.iter().any(|name| name.ends_with(".db")),
        "the release subject started with no database: {before:?}"
    );
    assert!(errors(&output).contains("this run is not stored"));
}

#[test]
fn the_bundle_a_release_writes_claims_no_signature() {
    // §62's own vocabulary: a bundle states integrity and consistency, and never a provenance promise. The
    // report is allowed to *disclaim* signing — that sentence says there is no such claim to lean on — so the
    // ban is on the four words §62 refuses, not on the topic.
    let world = World::new("no-signing", PASSING);
    let output = world.prepare(&["--json"]);
    assert_eq!(code(&output), Some(0), "{}", errors(&output));

    let mut bodies = vec![text(&output), errors(&output)];
    let bundle = world.published();
    for name in tree(&bundle) {
        if name.starts_with("artifacts/") {
            continue;
        }
        let bytes = std::fs::read(bundle.join(&name)).expect("the file is written");
        bodies.push(String::from_utf8_lossy(&bytes).into_owned());
    }
    for body in &bodies {
        let lower = body.to_lowercase();
        for forbidden in ["trusted", "authentic", "signed", "tamper"] {
            assert!(
                !lower.contains(forbidden),
                "a release output claims {forbidden:?}"
            );
        }
    }
}

#[test]
fn the_release_help_names_prepare_and_its_frozen_options() {
    let help = run(&["release", "--help"]);
    assert_eq!(code(&help), Some(0), "{}", errors(&help));
    assert!(
        text(&help).contains("prepare"),
        "`release --help` should list its one verb: {}",
        text(&help)
    );

    let help = run(&["release", "prepare", "--help"]);
    assert_eq!(code(&help), Some(0), "{}", errors(&help));
    let body = text(&help);
    for option in [
        "--artifact",
        "--map",
        "--baseline",
        "--baseline-map",
        "--project",
        "--out",
        "--force",
        "--json",
    ] {
        assert!(
            body.contains(option),
            "`{option}` is missing from the help:\n{body}"
        );
    }
}

#[test]
fn release_requires_both_an_artifact_and_a_destination() {
    let world = World::new("required", PASSING);
    let out = world.out.to_string_lossy().into_owned();
    let artifact = absolute(TARGET_ELF);
    for missing in [
        vec!["release", "prepare", "--out", out.as_str()],
        vec!["release", "prepare", "--artifact", artifact.as_str()],
        vec!["release", "prepare"],
    ] {
        let output = run(&missing);
        assert_eq!(code(&output), Some(2), "{missing:?}: {}", errors(&output));
        assert!(
            !std::fs::exists(&world.out).unwrap_or(false),
            "a usage error created a folder: {missing:?}"
        );
    }
}

#[test]
fn an_unregistered_release_verb_is_a_usage_error() {
    // §33 froze one verb. A second would be a product decision nobody made, so the family stays closed —
    // including the tempting `release verify`, which is a library call, not a command.
    let world = World::new("verbs", PASSING);
    let project = world.project.to_string_lossy().into_owned();
    let artifact = absolute(TARGET_ELF);
    let out = world.out.to_string_lossy().into_owned();
    for verb in ["publish", "list", "verify"] {
        let output = run(&[
            "release",
            verb,
            "--project",
            project.as_str(),
            "--artifact",
            artifact.as_str(),
            "--out",
            out.as_str(),
        ]);
        assert_eq!(code(&output), Some(2), "`release {verb}` must not exist");
    }
    assert!(
        !std::fs::exists(&world.out).unwrap_or(false),
        "a rejected verb created a folder"
    );
}

// ------------------------------------------------------------------ release attachments (`C1-U4`)

/// The passing policy with the two kinds nothing in this build analyzes added to the requirement list: the
/// release cannot pass until a person attaches them, and cannot pass on an analyzed stand-in either.
const ATTACHING: &str = r#"schema_version = 1

[project]
name = "motor-controller"

[artifacts]
required = ["elf", "map", "bin", "hex"]

[version]
source = "git_tag"
pattern = '^v(?P<version>\d+\.\d+\.\d+)$'

[release]
require_clean_git = true
require_release_notes = true
release_notes_path = "RELEASE_NOTES.md"
expected_version = "1.4.2"
"#;

/// The same policy with `hex` left out. Two files whose leaves differ only by case are both BIN, so a policy
/// that also demanded a HEX would stop at the Gate and never reach the naming question — a BLOCK is refused
/// before any collision is looked at, which is adjudication D's order and this case is here to show rather
/// than to work around.
const ATTACH_TWO_BINS: &str = r#"schema_version = 1

[project]
name = "motor-controller"

[artifacts]
required = ["elf", "map", "bin"]

[version]
source = "git_tag"
pattern = '^v(?P<version>\d+\.\d+\.\d+)$'

[release]
require_clean_git = true
require_release_notes = true
release_notes_path = "RELEASE_NOTES.md"
expected_version = "1.4.2"
"#;

/// One file a release owner could point `--attach` at. It is written beside the project and never inside it,
/// because this world's policy requires a clean repository: bytes dropped into the workspace would fail the
/// release for a reason that has nothing to do with attachments.
fn attachable(world: &World, folder: &str, name: &str, bytes: &[u8]) -> String {
    let dir = world
        .project
        .parent()
        .expect("the project sits in a scratch folder")
        .join(folder);
    std::fs::create_dir_all(&dir).expect("an attachment folder");
    let path = dir.join(name);
    std::fs::write(&path, bytes).expect("an attachment");
    path.to_string_lossy().into_owned()
}

/// `ATTACHING` with the clean-repository rule lifted. `N01-F01` puts bytes under the project root on purpose,
/// so the release has to be able to answer for a dirty workspace; without this the run would fail for a
/// reason that has nothing to do with which directory a relative path starts from.
const ATTACHING_DIRTY: &str = r#"schema_version = 1

[project]
name = "motor-controller"

[artifacts]
required = ["elf", "map", "bin", "hex"]

[version]
source = "git_tag"
pattern = '^v(?P<version>\d+\.\d+\.\d+)$'

[release]
require_clean_git = false
require_release_notes = true
release_notes_path = "RELEASE_NOTES.md"
expected_version = "1.4.2"
"#;

/// The process started in `current_dir` rather than in this repository, which is the whole question: whose
/// directory does a relative `--attach` read against.
fn run_from(current_dir: &Path, args: &[&str]) -> Output {
    let output = Command::new(env!("CARGO_BIN_EXE_fwsight"))
        .args(args)
        .current_dir(current_dir)
        .output()
        .expect("the fwsight binary is built for its own tests");
    let stderr = String::from_utf8_lossy(&output.stderr).to_lowercase();
    assert!(!stderr.contains("panicked"), "the CLI panicked:\n{stderr}");
    output
}

/// §33's invocation run from `current_dir`, with every path but the attachment's named absolutely.
fn prepare_from(world: &World, current_dir: &Path, out: &Path, extra: &[&str]) -> Output {
    let mut args = vec![
        "release".to_owned(),
        "prepare".to_owned(),
        "--project".to_owned(),
        world.project.to_string_lossy().into_owned(),
        "--artifact".to_owned(),
        absolute(TARGET_ELF),
        "--map".to_owned(),
        absolute(TARGET_MAP),
        "--out".to_owned(),
        out.to_string_lossy().into_owned(),
    ];
    args.extend(extra.iter().map(|arg| (*arg).to_owned()));
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    run_from(current_dir, &refs)
}

/// Help text wraps at the terminal's width, so a phrase is looked for with the line breaks folded away.
fn flattened(body: &str) -> String {
    body.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// One document read out of a written bundle.
fn bundle_document(bundle: &Path, name: &str) -> serde_json::Value {
    let bytes = std::fs::read(bundle.join(name))
        .unwrap_or_else(|error| panic!("`{name}` is not in the bundle: {error}"));
    serde_json::from_slice(&bytes).unwrap_or_else(|error| panic!("`{name}` is not JSON: {error}"))
}

/// The report the person reads, as text.
fn bundle_report(bundle: &Path) -> String {
    std::fs::read_to_string(bundle.join("release-report.html")).expect("the report is written")
}

#[test]
fn a_release_that_attaches_two_files_ships_their_bytes_and_verifies_from_them() {
    let world = World::new("attach-ship", ATTACHING);
    let bin = attachable(&world, "build", "app.bin", b"APP-BYTES");
    let hex = attachable(&world, "build", "boot.hex", b":00000001FF");

    let output = world.prepare(&["--attach", &bin, "--attach", &hex]);
    assert_eq!(code(&output), Some(0), "{}", errors(&output));
    let bundle = world.published();

    // Independent of the process that just ran: the engine re-hashes every byte the bundle carries against
    // the index it carries, and that is what "verified" means in this product (`04_TECH/17`).
    let verification =
        verify_bundle(&bundle).expect("a bundle that ships attachments still verifies");

    let manifest = bundle_document(&bundle, "release-manifest.json");
    let rows = manifest["extensions"]["attachments"]
        .as_array()
        .expect("the manifest discloses what it shipped")
        .clone();
    assert_eq!(rows.len(), 2, "{rows:?}");
    assert_eq!(
        verification.release_id,
        manifest["release"]["id"].as_str().expect("a release id"),
        "the id the engine recomputed is the id the manifest claims"
    );

    // The disclosure half of the six-link chain: each row names a bundle-relative path, the bytes at that
    // path re-hash to the row's digest, and nothing about them is claimed beyond what was measured.
    for row in &rows {
        let path = row["path"].as_str().expect("a bundle-relative locator");
        let shipped = bundle.join(path);
        assert!(shipped.is_file(), "{path} is indexed but was not shipped");
        assert_eq!(
            fingerprint::file_sha256(&shipped).expect("the shipped bytes re-hash"),
            row["sha256"].as_str().expect("a digest"),
            "{path} ships bytes that are not the ones the manifest disclosed"
        );
        assert_eq!(row["provenance"], serde_json::json!("unknown"), "{row}");
        assert_eq!(row["kind_basis"], serde_json::json!("declared"), "{row}");
    }

    // The same two sets in the gate document, with the grammar they were bound under, and one run id that
    // both documents agree on — a bundle cannot gate one run and ship another's verdict.
    let gate = bundle_document(&bundle, "gate-results.json");
    assert_eq!(
        gate["extensions"]["canonical_input_label"],
        serde_json::json!("firmwaresight-gate-input/2")
    );
    let disclosed = gate["extensions"]["attachments"]
        .as_array()
        .expect("the gate discloses what it bound");
    assert_eq!(disclosed.len(), 2, "{disclosed:?}");
    for row in disclosed {
        let digest = row["sha256"].as_str().expect("a digest");
        assert!(
            rows.iter().any(|ship| ship["sha256"] == digest),
            "the gate bound {digest}, which the manifest does not ship"
        );
    }
    assert_eq!(
        gate["run_id"], manifest["extensions"]["gate_run_id"],
        "one bundle, one gate run"
    );
}

#[test]
fn a_release_with_no_attach_flag_ships_the_bundle_it_shipped_before_the_flag_existed() {
    let world = World::new("attach-none", ATTACHING);
    // The requirement list is the same on both runs, so the only difference is the flag: without any
    // attachment the release must BLOCK (a required kind nothing covers), which is the honest half of this
    // case — an unchanged bundle shape cannot be shown passing on a policy the bytes do not satisfy.
    let blocked = world.prepare(&[]);
    assert_eq!(code(&blocked), Some(5), "{}", errors(&blocked));
    assert!(
        wrote_nothing(&world.out),
        "a BLOCK stops before the write: {:?}",
        tree(&world.out)
    );

    // The unchanged-shape proof therefore runs against a policy that asks for nothing new.
    let plain = World::new("attach-none-plain", PASSING);
    let output = plain.prepare(&[]);
    assert_eq!(code(&output), Some(0), "{}", errors(&output));
    let bundle = plain.published();
    let manifest = bundle_document(&bundle, "release-manifest.json");
    let gate = bundle_document(&bundle, "gate-results.json");
    for (document, name) in [(&manifest, "release-manifest"), (&gate, "gate-results")] {
        let extensions = document["extensions"]
            .as_object()
            .unwrap_or_else(|| panic!("`{name}` carries extensions"));
        for key in ["attachments", "canonical_input_label"] {
            assert!(
                !extensions.contains_key(key),
                "`{name}` grew a C1 entry although nothing was attached: {extensions:?}"
            );
        }
    }
    assert!(
        !bundle_report(&bundle).contains("byte evidence"),
        "a bundle with no attachment has nothing to say about attachments"
    );
}

#[test]
fn two_attachments_that_fold_onto_one_leaf_but_differ_in_bytes_ship_as_two_entries() {
    // Found by writing the opposite expectation and running it: a shipped attachment is named by
    // `disambiguated_name(kind, sha256, leaf)`, so two selections whose leaves differ only by case land on
    // two different files and the release proceeds. That is the behaviour, so it gets a test — a reader of
    // the case below needs to know which of the two folded shapes is refused and why.
    let world = World::new("attach-fold-distinct", ATTACH_TWO_BINS);
    let lower = attachable(&world, "build", "app.bin", b"FIRST");
    let upper = attachable(&world, "ship", "APP.BIN", b"SECOND");

    let output = world.prepare(&["--attach", &lower, "--attach", &upper]);
    assert_eq!(code(&output), Some(0), "{}", errors(&output));
    let bundle = world.published();
    let manifest = bundle_document(&bundle, "release-manifest.json");
    let rows = manifest["extensions"]["attachments"]
        .as_array()
        .expect("rows");
    assert_eq!(rows.len(), 2, "{rows:?}");
    let paths: Vec<&str> = rows
        .iter()
        .map(|row| row["path"].as_str().expect("a locator"))
        .collect();
    assert_ne!(paths[0], paths[1], "{paths:?}");
    verify_bundle(&bundle).expect("both folded-spelling entries still verify");
}

#[test]
fn two_attachments_that_fold_onto_one_leaf_and_carry_one_digest_are_refused_and_write_nothing() {
    // The shape that cannot ship: the same bytes selected twice under names that differ only by case
    // collapse to one bundle entry. §B's point is that the person is told about *their selection*, so this
    // asserts the sentence a release owner reads, not just the code.
    let world = World::new("attach-fold", ATTACH_TWO_BINS);
    let lower = attachable(&world, "build", "app.bin", b"SHARED");
    let upper = attachable(&world, "ship", "APP.BIN", b"SHARED");

    let output = world.prepare(&["--attach", &lower, "--attach", &upper]);
    assert_eq!(code(&output), Some(6), "{}", errors(&output));
    let stderr = errors(&output);
    assert!(stderr.contains("ERR-BUNDLE-6109"), "{stderr}");
    assert!(
        stderr.contains("is attached twice") && stderr.contains("differ only in the case"),
        "the refusal has to be about the two names the owner selected, not about a manifest they never \
         wrote: {stderr}"
    );
    assert!(
        wrote_nothing(&world.out),
        "a refused selection wrote a partial bundle: {:?}",
        tree(&world.out)
    );
}

#[test]
fn a_relative_attach_is_read_against_the_project_directory_and_not_the_current_one() {
    // `N01-F01`, the other half of the pair `gate_cli.rs` pins: the same flag spelled the same way starts
    // from the project here, so the file a release owner shipped is the one under their `--project`.
    let world = World::new("attach-base", ATTACHING_DIRTY);
    let scratch = world
        .project
        .parent()
        .expect("the project sits in a scratch folder");
    let out = scratch.join("out-relative");
    let under_project = world.project.join("app.bin");
    let decoy_bin = scratch.join("app.bin");
    std::fs::write(&under_project, b"UNDER-THE-PROJECT-ROOT").expect("under the project");
    // The same leaf name one level up, in the directory the process runs in, holding different bytes.
    std::fs::write(&decoy_bin, b"UNDER-THE-CURRENT-DIRECTORY").expect("beside the project");
    let hex = attachable(&world, "build", "boot.hex", b":00000001FF");

    let output = prepare_from(
        &world,
        scratch,
        &out,
        &["--attach", "app.bin", "--attach", &hex],
    );
    assert_eq!(code(&output), Some(0), "{}", errors(&output));
    let rows =
        bundle_document(&only_bundle(&out), "release-manifest.json")["extensions"]["attachments"]
            .as_array()
            .expect("the manifest lists what it shipped")
            .clone();
    let sizes: Vec<u64> = rows.iter().filter_map(|row| row["size"].as_u64()).collect();
    let project_copy: u64 = b"UNDER-THE-PROJECT-ROOT".len() as u64;
    let workdir_copy: u64 = b"UNDER-THE-CURRENT-DIRECTORY".len() as u64;
    assert!(
        sizes.contains(&project_copy),
        "the bytes under the project root were not the ones shipped: {rows:?}"
    );
    assert!(
        !sizes.contains(&workdir_copy),
        "the current directory's copy was shipped instead: {rows:?}"
    );

    // A name that only the current directory holds is refused, which is the same fact seen from the other
    // side, and the refusal writes nothing.
    let decoy = scratch.join("solo.hex");
    std::fs::write(&decoy, b":00000001FF").expect("a file only the current directory holds");
    let under_project_text = under_project.to_string_lossy().into_owned();
    let refused = prepare_from(
        &world,
        scratch,
        &scratch.join("out-decoy"),
        &["--attach", &under_project_text, "--attach", "solo.hex"],
    );
    assert_eq!(code(&refused), Some(6), "{}", errors(&refused));
    assert!(
        errors(&refused).contains("ERR-BUNDLE-6115"),
        "{}",
        errors(&refused)
    );
    assert!(
        !scratch.join("out-decoy").exists(),
        "a refused relative path created a destination folder"
    );

    // A relative name that climbs out of the project is refused too, rather than shipping a file from
    // above the release the policy describes.
    let escaped = prepare_from(
        &world,
        scratch,
        &scratch.join("out-escaped"),
        &["--attach", "../solo.hex", "--attach", &hex],
    );
    assert_eq!(code(&escaped), Some(6), "{}", errors(&escaped));
    assert!(
        errors(&escaped).contains("ERR-BUNDLE-6115"),
        "{}",
        errors(&escaped)
    );
    assert!(
        !scratch.join("out-escaped").exists(),
        "a path outside the project left a destination folder behind"
    );
    assert!(
        !errors(&escaped).contains(scratch.to_string_lossy().as_ref()),
        "the refusal quoted the host directory it refused in: {}",
        errors(&escaped)
    );
}

#[test]
fn a_selection_that_climbs_out_of_the_project_is_refused_when_a_file_of_the_fallback_name_exists() {
    // `D-N09-2`'s safety question, which is not about wording. The engine used to answer an illegal relative
    // component by resolving it onto one name it assumed could never exist. A project that happens to hold a
    // file of that name therefore shipped bytes nobody chose, so the refusal has to come from the selection
    // itself, before anything is stat-ed, hashed or packed.
    let world = World::new("attach-marker-present", ATTACHING_DIRTY);
    let scratch = world
        .project
        .parent()
        .expect("the project sits in a scratch folder");
    std::fs::write(scratch.join("escape.bin"), b"OUTSIDE-THE-PROJECT")
        .expect("a file one level above the project");
    let hex = attachable(&world, "build", "boot.hex", b":00000001FF");
    let marker = world.project.join("firmwaresight-unresolvable-path");
    std::fs::write(&marker, b"MARKER-BYTES-NEVER-CHOSEN")
        .expect("the fallback name exists on disk");
    let marker_digest = fingerprint::file_sha256(&marker).expect("the fallback file hashes");
    let out = scratch.join("out-marker");

    let refused = prepare_from(
        &world,
        scratch,
        &out,
        &["--attach", "../escape.bin", "--attach", &hex],
    );
    let shipped = shipped_digests(&out);
    assert_eq!(
        code(&refused),
        Some(6),
        "an illegal climb out of the project was accepted, and it shipped {} as the release's \
         attachment: {}",
        if shipped.contains(&marker_digest) {
            "the bytes of the file nobody chose"
        } else {
            "bytes that are not the fallback file's"
        },
        shipped.join(", ")
    );
    assert!(
        errors(&refused).contains("ERR-BUNDLE-6115"),
        "the refusal keeps the frozen observation code: {}",
        errors(&refused)
    );
    assert!(
        !out.exists(),
        "an illegal selection wrote a destination directory: {:?}",
        tree(&out)
    );

    // The bytes that were almost shipped are still the file the owner never named, sitting where it was.
    let written: Vec<u8> = std::fs::read(&marker).expect("the fallback file is left alone");
    assert_eq!(
        written, b"MARKER-BYTES-NEVER-CHOSEN",
        "the refusal modified the file it was refusing to read"
    );
    assert!(
        !shipped_digests(&out).contains(&marker_digest),
        "the fallback file was packed into a bundle"
    );
}

#[test]
fn a_refused_climb_out_of_the_project_names_neither_the_host_directory_nor_an_internal_marker() {
    // The same selection with nothing at the fallback name: the refusal is still correct, and it is the
    // *sentence* that is on trial. A release owner reads this in a terminal and pastes it into a report, so
    // it may not quote a directory (`04_TECH/28` §4.4) and may not name a marker that only exists inside
    // this crate's source (`D-N09-2`).
    let world = World::new("attach-marker-absent", ATTACHING_DIRTY);
    let scratch = world
        .project
        .parent()
        .expect("the project sits in a scratch folder");
    std::fs::write(scratch.join("escape.bin"), b"OUTSIDE-THE-PROJECT")
        .expect("a file one level above the project");
    let hex = attachable(&world, "build", "boot.hex", b":00000001FF");
    let out = scratch.join("out-marker-absent");

    let refused = prepare_from(
        &world,
        scratch,
        &out,
        &["--attach", "../escape.bin", "--attach", &hex],
    );
    assert_eq!(code(&refused), Some(6), "{}", errors(&refused));
    let stderr = errors(&refused);
    assert!(stderr.contains("ERR-BUNDLE-6115"), "{stderr}");
    assert!(
        !stderr.contains(&scratch.to_string_lossy().into_owned()),
        "the refusal quoted the host directory it refused in: {stderr}"
    );
    assert!(
        !stderr.contains("firmwaresight-unresolvable-path"),
        "the refusal quoted this crate's internal fallback name to the release owner: {stderr}"
    );
    assert!(
        !out.exists(),
        "a refused selection left a destination behind"
    );
}

#[test]
fn a_file_legitimately_named_like_the_internal_marker_is_still_attachable() {
    // The other half of the fix: refusing a component a person never typed must not turn the marker into a
    // banned file name. A project may hold a file of that exact name, and its owner may choose it.
    let world = World::new("attach-marker-legal", ATTACHING_DIRTY);
    let marker = world.project.join("firmwaresight-unresolvable-path");
    std::fs::write(&marker, b"CHOSEN-ON-PURPOSE").expect("a file whose owner named it that");
    let bin = attachable(&world, "build", "app.bin", b"APP-BYTES");
    let hex = attachable(&world, "build", "boot.hex", b":00000001FF");
    let marker_text = marker.to_string_lossy().into_owned();

    // Project-relative, the spelling most likely to be caught by a refusal that matches on the name.
    let relative = prepare_from(
        &world,
        &world.project,
        &world.out,
        &[
            "--attach",
            "firmwaresight-unresolvable-path",
            "--attach",
            &bin,
            "--attach",
            &hex,
        ],
    );
    assert_eq!(code(&relative), Some(0), "{}", errors(&relative));
    let bundle = world.published();
    let shipped = bundle_document(&bundle, "release-manifest.json")["extensions"]["attachments"]
        .as_array()
        .expect("the manifest discloses what it shipped")
        .clone();
    assert_eq!(
        shipped
            .iter()
            .filter(|row| row["kind"] == "unknown")
            .count(),
        1,
        "the chosen file was not shipped as the unidentified attachment it is: {shipped:?}"
    );
    std::fs::remove_dir_all(&bundle).expect("the first bundle is cleared for the second run");

    let absolute = prepare_from(
        &world,
        &world.project,
        &world.out,
        &["--attach", &marker_text, "--attach", &bin, "--attach", &hex],
    );
    assert_eq!(code(&absolute), Some(0), "{}", errors(&absolute));
    verify_bundle(&world.published())
        .expect("a bundle carrying the marker-named file still verifies");
}

/// Every digest one release shipped, so a test can prove a file was packed rather than merely refused. A
/// destination that never became a bundle ships none.
fn shipped_digests(out: &Path) -> Vec<String> {
    let bundle = match std::fs::read_dir(out) {
        Ok(entries) => {
            let dirs: Vec<PathBuf> = entries
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .filter(|path| path.is_dir())
                .collect();
            match dirs.as_slice() {
                [only] => only.clone(),
                _ => return Vec::new(),
            }
        }
        Err(_) => return Vec::new(),
    };
    bundle_document(&bundle, "release-manifest.json")["extensions"]["attachments"]
        .as_array()
        .map(|rows| {
            rows.iter()
                .filter_map(|row| row["sha256"].as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn the_release_surface_says_which_directory_a_relative_attach_starts_from() {
    let help = run(&["release", "prepare", "--help"]);
    assert_eq!(code(&help), Some(0), "{}", errors(&help));
    let body = flattened(&text(&help));
    assert!(
        body.contains("--attach <FILE>"),
        "the flag is not in the help:\n{body}"
    );
    assert!(
        body.contains("A relative path is read against the project directory named by `--project`"),
        "the help does not name this command's base:\n{body}"
    );
    assert!(
        !body.contains("the same single observation path as `gate --attach`"),
        "the help still claims the two verbs share one path, which `N01-F01` measured as false:\n{body}"
    );

    let world = World::new("attach-base-hint", ATTACHING);
    let output = world.prepare(&["--attach", "nowhere.bin"]);
    assert_eq!(code(&output), Some(6), "{}", errors(&output));
    let stderr = errors(&output);
    assert!(
        stderr.contains("hint: `release prepare` read that path from the project directory"),
        "{stderr}"
    );
    assert!(
        !stderr.contains(&world.project.to_string_lossy().to_string()),
        "the hint carried the host directory: {stderr}"
    );
}

#[test]
fn an_attachment_that_cannot_be_read_refuses_the_release_with_the_export_code() {
    let world = World::new("attach-absent", ATTACHING);
    let absent = world
        .project
        .parent()
        .expect("a scratch folder")
        .join("nowhere.bin")
        .to_string_lossy()
        .into_owned();

    let output = world.prepare(&["--attach", &absent]);
    // A file that cannot be read stops a release for the reason §34 freezes: the export could not be
    // produced. The refusal keeps the engine's own observation code inside it, so one refusal carries one
    // code wherever a person meets it.
    assert_eq!(code(&output), Some(6), "{}", errors(&output));
    let stderr = errors(&output);
    assert!(stderr.contains("ERR-BUNDLE-6115"), "{stderr}");
    assert!(
        wrote_nothing(&world.out),
        "an unreadable attachment left a bundle behind: {:?}",
        tree(&world.out)
    );
}

#[test]
fn a_chosen_name_that_a_portable_folder_cannot_carry_ships_under_a_safe_one_and_keeps_its_own_bytes()
 {
    // `N01-F02`'s archive half, measured through the real CLI: the leaf a release owner chose in Chinese is
    // what they read on the Release page, and what crosses into the folder is the name `sanitize_leaf_name`
    // allows. Two roles, one file, and no document that was not already written before this test existed.
    let world = World::new("attach-unicode", ATTACHING);
    let bin = attachable(&world, "build", "固件镜像.bin", b"APP-BYTES");
    let hex = attachable(&world, "build", "boot.hex", b":00000001FF");
    let output = world.prepare(&["--attach", &bin, "--attach", &hex]);
    assert_eq!(code(&output), Some(0), "{}", errors(&output));

    let bundle = world.published();
    let rows = bundle_document(&bundle, "release-manifest.json")["extensions"]["attachments"]
        .as_array()
        .expect("the manifest lists what it shipped")
        .clone();
    let paths: Vec<&str> = rows.iter().filter_map(|row| row["path"].as_str()).collect();
    assert!(
        paths.contains(&"artifacts/bin"),
        "the chosen `固件镜像.bin` is filed under its archive name: {rows:?}"
    );
    assert!(
        std::fs::metadata(bundle.join("artifacts").join("bin")).is_ok(),
        "the folder carries no file at the name the manifest names: {:?}",
        tree(&bundle)
    );

    // The two names are two roles, so the one that only ever lived on this machine stays out of every
    // document a stranger can read — the report and the manifest included.
    for name in tree(&bundle) {
        if name.starts_with("artifacts/") {
            continue;
        }
        let body = std::fs::read(bundle.join(&name)).expect("the file the tree just listed");
        assert!(
            !String::from_utf8_lossy(&body).contains("固件镜像"),
            "`{name}` carries the host's chosen leaf"
        );
    }
}

#[test]
fn two_chosen_names_that_fold_onto_one_archive_name_ship_as_two_entries_the_reader_can_tell_apart()
{
    // The collision rule already existed for ASCII leaves that fold (`FIRMWARE.BIN` and `firmware.bin`);
    // a Chinese pair reaches it far more often, and the bundle must still carry both files.
    let world = World::new("attach-unicode-pair", ATTACH_TWO_BINS);
    let first = attachable(&world, "build", "固件.bin", b"FIRST-IMAGE");
    let second = attachable(&world, "build", "镜像.bin", b"SECOND-IMAGE");
    let output = world.prepare(&["--attach", &first, "--attach", &second]);
    assert_eq!(code(&output), Some(0), "{}", errors(&output));

    let bundle = world.published();
    let rows = bundle_document(&bundle, "release-manifest.json")["extensions"]["attachments"]
        .as_array()
        .expect("the manifest lists what it shipped")
        .clone();
    let paths: Vec<&str> = rows.iter().filter_map(|row| row["path"].as_str()).collect();
    assert_eq!(paths.len(), 2, "{rows:?}");
    assert_ne!(paths[0], paths[1], "one entry per folded name: {rows:?}");
    for path in &paths {
        assert!(
            std::fs::metadata(bundle.join(path)).is_ok(),
            "`{path}` is indexed but not shipped: {:?}",
            tree(&bundle)
        );
    }
}

#[test]
fn a_bundle_that_ships_attachments_says_so_in_the_report_it_ships() {
    let world = World::new("attach-report", ATTACHING);
    let bin = attachable(&world, "build", "app.bin", b"APP-BYTES");
    let hex = attachable(&world, "build", "boot.hex", b":00000001FF");
    assert_eq!(
        code(&world.prepare(&["--attach", &bin, "--attach", &hex])),
        Some(0)
    );
    let bundle = world.published();
    let report = bundle_report(&bundle);

    // §9's one line, in the limits section, with the count the bundle actually carries. It says what was
    // verified and what was not, in that order, and it does not say `analyzed`.
    assert!(
        report.contains("2 release attachment(s) ship as byte evidence"),
        "{report}"
    );
    assert!(
        report.contains("their structure and their build provenance are not verified"),
        "{report}"
    );
    assert!(
        !report.contains("BIN and Intel HEX are analyzed"),
        "the report must not promise the analysis this release does not do"
    );
}

// ------------------------------------------------------ D-N10-1: the configured notes path binds the file it names

/// The crate's own fallback leaf. A configured path the resolver could not place used to land here, which
/// made the refusal itself a readable file — so a project holding any file of this name could have its
/// bytes read as somebody's Release Notes.
const FALLBACK_LEAF: &str = "firmwaresight-unresolvable-path";

fn shipped_notes(bundle: &Path) -> Vec<u8> {
    std::fs::read(bundle.join("release-notes.md"))
        .unwrap_or_else(|error| panic!("`release-notes.md` is not in the bundle: {error}"))
}

/// A release owner may spell their notes `docs/../RELEASE_NOTES.md`, and it is the same file as
/// `RELEASE_NOTES.md`. The resolver used to have no `..` rule at all: it fell through to the fallback leaf,
/// found nothing there, and reported existing notes as missing — a `BLOCK` no edit to the notes can fix.
#[test]
fn a_configured_notes_path_that_climbs_and_returns_is_read_at_the_file_it_names() {
    let world = World::new("notes-climb", CLIMBING_NOTES);
    let output = world.prepare(&[]);
    assert_eq!(code(&output), Some(0), "{}", errors(&output));

    let bundle = world.published();
    assert_eq!(
        shipped_notes(&bundle),
        NOTES.as_bytes(),
        "the bundle ships the notes the config named, which live at the project root"
    );
    verify_bundle(&bundle).expect("a bundle bound to the real notes still verifies");
}

/// The adversarial half, and the reason this was a correctness defect rather than a wording one: with a file
/// of the fallback name sitting in the project, the same configuration hashed *that* file, passed, and
/// shipped its bytes as the release's Release Notes. The project's own commit is untouched here, so the two
/// runs differ only by the presence of a file nobody configured.
#[test]
fn the_internal_fallback_name_never_becomes_the_configured_release_notes() {
    let world = World::new("notes-fallback", CLIMBING_NOTES);
    std::fs::write(
        world.project.join(FALLBACK_LEAF),
        "bytes no release ever configured\n",
    )
    .expect("the fallback-named file is written");

    let held = world.export_to(&world.out.join("fallback-held"), &[]);
    assert_eq!(code(&held), Some(0), "{}", errors(&held));
    let held_bundle = only_bundle(&world.out.join("fallback-held"));
    assert_eq!(
        shipped_notes(&held_bundle),
        NOTES.as_bytes(),
        "a file of the fallback name must not become the release's Release Notes"
    );

    // The same project, the same commit, the fallback name now free: the answer has to be identical.
    std::fs::remove_file(world.project.join(FALLBACK_LEAF))
        .expect("the fallback-named file is removed");
    let free = world.export_to(&world.out.join("fallback-free"), &[]);
    assert_eq!(code(&free), Some(0), "{}", errors(&free));
    assert_eq!(
        shipped_notes(&only_bundle(&world.out.join("fallback-free"))),
        shipped_notes(&held_bundle),
        "the presence of a file the release did not choose cannot change which bytes are the Release Notes"
    );
    verify_bundle(&held_bundle).expect("the bundle over the occupied fallback name verifies too");
}

/// The refused half of the matrix, at the surface a release owner hits it. Each of these was refused before
/// this round and is refused after — by the same walk the resolver uses, so there is no second list of legal
/// shapes to fall out of step. What is asserted is that a refusal writes nothing and quotes neither the
/// host's directory nor an internal fallback name.
#[test]
fn a_configured_notes_path_that_cannot_be_placed_writes_nothing_and_leaks_no_path() {
    for (index, configured) in [
        "../escape.md",
        "docs/../../escape.md",
        "/etc/passwd",
        // Written the way TOML has to spell it: the value the config ends up holding is
        // `C:\Windows\win.ini`, and an unescaped backslash is a parse error rather than a path.
        "C:\\\\Windows\\\\win.ini",
        "C:/Windows/win.ini",
        "docs/..",
        "",
    ]
    .into_iter()
    .enumerate()
    {
        let config = CLIMBING_NOTES.replace("docs/../RELEASE_NOTES.md", configured);
        let world = World::new(&format!("notes-refused-{index}"), &config);
        let output = world.prepare(&[]);
        let shown = format!("{}{}", text(&output), errors(&output));
        assert_ne!(
            code(&output),
            Some(0),
            "`{configured}` cannot be placed, so no release may be built on it: {shown}"
        );
        assert!(
            shown.contains("is not project-relative"),
            "`{configured}` was not refused as a path problem, so something else answered it: {shown}"
        );
        assert!(
            wrote_nothing(&world.out),
            "a refused notes path still exported a bundle: {:?}",
            tree(&world.out)
        );
        assert!(
            !shown.contains(FALLBACK_LEAF),
            "the refusal for `{configured}` named the internal fallback: {shown}"
        );
        assert!(
            !shown.contains(&world.project.to_string_lossy().into_owned()),
            "the refusal for `{configured}` quoted the host directory it refused to read: {shown}"
        );
    }
}
