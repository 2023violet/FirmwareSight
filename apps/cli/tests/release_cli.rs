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
