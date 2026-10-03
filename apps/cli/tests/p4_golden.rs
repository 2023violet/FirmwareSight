//! The P4 golden bundle: the ten files one release writes, and what a stranger can still do with them
//! after everything that produced them is deleted (prompt §45, §46, §47, §61).
//!
//! Same rule as the P0 and P2 goldens: a failing comparison never rewrites its own expectation.
//! Regeneration is the separate, reviewed `python scripts/update_goldens.py --confirm`, and this file
//! can only ever report a change.
//!
//! The subject is a throwaway Git repository holding the committed P4 project fixture
//! (`fixtures/project/p4-release`) over the committed P2 Compare pair, with the commit's author,
//! committer, both dates and message pinned. That is what makes the release id, the manifest and every
//! digest in it reproducible: an unpinned commit would move the workspace hash on every machine and take
//! the whole bundle with it. FirmwareSight's own repository is never the subject (§47).
//!
//! The bundle's documents are compared **byte for byte**, not by content. A digest was taken over those
//! exact bytes, so a re-ordered key or a re-indented line is a different bundle even when it means the
//! same thing — which is also why the golden holds the shipped text rather than a pretty-printing of it.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use firmwaresight_core::domain::release::{
    ACCEPTED_REVIEWS_DOC_NAME, ANALYSIS_DOC_NAME, ARTIFACTS_DIR, DIFF_DOC_NAME,
    GATE_RESULTS_DOC_NAME, MANIFEST_DOC_NAME, REPORT_DOC_NAME, SHA256SUMS_NAME,
    SUMS_EXCLUDED_NAMES,
};
use firmwaresight_project::{bundle, fingerprint};
use firmwaresight_report::schemas;

/// The documents the golden keeps. The two files under `artifacts/` are not among them: their bytes are
/// already committed as fixtures and already hashed there (§46).
const GOLDEN_FILES: [&str; 8] = [
    SHA256SUMS_NAME,
    ACCEPTED_REVIEWS_DOC_NAME,
    ANALYSIS_DOC_NAME,
    DIFF_DOC_NAME,
    GATE_RESULTS_DOC_NAME,
    MANIFEST_DOC_NAME,
    "release-notes.md",
    REPORT_DOC_NAME,
];

/// Every file one release of this pair writes, in the order Core's bundle-path rule sorts them.
const BUNDLE_FILES: [&str; 10] = [
    ACCEPTED_REVIEWS_DOC_NAME,
    ANALYSIS_DOC_NAME,
    "artifacts/firmware.elf",
    "artifacts/firmware.map",
    DIFF_DOC_NAME,
    GATE_RESULTS_DOC_NAME,
    MANIFEST_DOC_NAME,
    "release-notes.md",
    REPORT_DOC_NAME,
    SHA256SUMS_NAME,
];

/// The five documents a contract exists for. `release-notes.md` is the release owner's bytes and
/// `SHA256SUMS` has its own format rule, so neither is a schema's subject (§21, §24).
const CONTRACT_DOCUMENTS: [&str; 5] = [
    ACCEPTED_REVIEWS_DOC_NAME,
    ANALYSIS_DOC_NAME,
    DIFF_DOC_NAME,
    GATE_RESULTS_DOC_NAME,
    MANIFEST_DOC_NAME,
];

const PROJECT_FIXTURE: &str = "fixtures/project/p4-release";
const TARGET_ELF: &str = "fixtures/elf/p2-diff/target/firmware.elf";
const TARGET_MAP: &str = "fixtures/elf/p2-diff/target/firmware.map";
const BASE_ELF: &str = "fixtures/elf/p2-diff/base/firmware.elf";
const BASE_MAP: &str = "fixtures/elf/p2-diff/base/firmware.map";

/// The pinned facts of the release subject's only commit. They are recorded in
/// `fixtures/project/p4-release/fixture.toml` as part of the fixture, and `scripts/update_goldens.py`
/// builds the same subject from them.
const COMMIT: &str = "release candidate";
const COMMIT_DATE: &str = "2026-09-30T07:00:00+00:00";
const AUTHOR_NAME: &str = "Release Owner";
const AUTHOR_EMAIL: &str = "release@example.invalid";
const TAG: &str = "v1.2.3";

/// The release id and workspace commit the pinned subject produces. Spelled out because a bundle whose
/// identity silently changed is the exact thing this file exists to catch.
///
/// P5 moved this value. `canonical_release_text` folds in `app={fwsight_version}`, so unifying the
/// artifact version on 0.6.0 (the owner's D1 decision, recorded in
/// `P5_VALIDATION/P5_PRODUCTIZATION_AUDIT.md`) re-issued the id for the same firmware, the same notes
/// and the same commit. The id below is read from the golden the 0.6.0 binary wrote, not computed by
/// hand; gate and diff identities did not move because neither input carries the app version.
const RELEASE_ID: &str = "release-e400a8ac51a57d34e309d254536f8850044ef4dd53e26245045c24fdc86fdcc5";
const EXPECTED_HEAD: &str = "585dafd3eb59592563923818cdfec4e9a370a83b";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|parent| parent.parent())
        .expect("apps/cli lives at <root>/apps/cli")
        .to_path_buf()
}

fn scratch(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "fwsight-p4-golden-{label}-{}-{:x}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|value| value.as_nanos())
            .unwrap_or(0)
    ))
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("{} must read: {error}", path.display()))
}

fn golden(name: &str) -> String {
    read(&repo_root().join("golden/reports/p4-release").join(name))
}

fn fixture(relative: &str) -> PathBuf {
    repo_root().join(relative)
}

/// One release subject: a pinned workspace, its staged sources, and the folder a release is written into.
///
/// The sources are *copies* of the committed fixtures rather than the fixtures themselves, so that a
/// portability test can delete them without touching the repository (§45). A bundle's identity is the
/// content it hashed, so where those bytes came from changes nothing in them.
struct Subject {
    root: PathBuf,
}

impl Subject {
    fn new(label: &str) -> Self {
        let root = scratch(label);
        let project = root.join("project");
        std::fs::create_dir_all(project.join("docs")).expect("a temporary project directory");
        for relative in ["firmwaresight.toml", "docs/RELEASE_NOTES.md"] {
            let source = fixture(&format!("{PROJECT_FIXTURE}/{relative}"));
            let bytes = std::fs::read(&source)
                .unwrap_or_else(|error| panic!("the fixture {relative} must read: {error}"));
            assert!(
                !bytes.contains(&b'\r'),
                "{relative} carries a CR byte, so its digest is not reproducible"
            );
            let target = project.join(relative);
            std::fs::create_dir_all(
                target
                    .parent()
                    .unwrap_or_else(|| panic!("{relative} sits in a folder")),
            )
            .expect("the notes folder exists");
            std::fs::write(&target, bytes).expect("the fixture is copied byte for byte");
        }

        // Both sides of the pair are staged under their own folder, because the leaf name is what the
        // bundle ships the artifact under and the two sides share one (`firmware.elf`).
        for (side, elf, map) in [
            ("target", TARGET_ELF, TARGET_MAP),
            ("base", BASE_ELF, BASE_MAP),
        ] {
            let folder = root.join("source").join(side);
            std::fs::create_dir_all(&folder).expect("a source folder");
            for path in [elf, map] {
                let name = Path::new(path)
                    .file_name()
                    .unwrap_or_else(|| panic!("{path} has a leaf name"));
                std::fs::copy(fixture(path), folder.join(name)).expect("the side is staged");
            }
        }

        git(&root, &project, &["init", "-q"]);
        git(&root, &project, &["add", "-A"]);
        git(
            &root,
            &project,
            &[
                "-c",
                &format!("user.name={AUTHOR_NAME}"),
                "-c",
                &format!("user.email={AUTHOR_EMAIL}"),
                "commit",
                "-q",
                "-m",
                COMMIT,
            ],
        );
        git(&root, &project, &["tag", TAG]);
        assert_eq!(
            head(&project),
            EXPECTED_HEAD,
            "the pinned subject must produce the pinned commit"
        );
        Self { root }
    }

    fn project(&self) -> PathBuf {
        self.root.join("project")
    }

    fn staged(&self, side: &str, name: &str) -> PathBuf {
        self.root.join("source").join(side).join(name)
    }

    /// Run the shipped binary over this subject into `out`, and return the one bundle directory it wrote.
    fn prepare(&self, out: &Path) -> PathBuf {
        let output = Command::new(env!("CARGO_BIN_EXE_fwsight"))
            .args([
                "release",
                "prepare",
                "--project",
                &self.project().to_string_lossy(),
                "--artifact",
                &self.staged("target", "firmware.elf").to_string_lossy(),
                "--map",
                &self.staged("target", "firmware.map").to_string_lossy(),
                "--baseline",
                &self.staged("base", "firmware.elf").to_string_lossy(),
                "--baseline-map",
                &self.staged("base", "firmware.map").to_string_lossy(),
                "--out",
                &out.to_string_lossy(),
            ])
            .current_dir(repo_root())
            .output()
            .expect("the fwsight binary is built for its own tests");
        assert_eq!(
            output.status.code(),
            Some(0),
            "a PASS release exits 0. stdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let names: Vec<String> = std::fs::read_dir(out)
            .expect("the destination exists")
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names.len(), 1, "one release writes one bundle: {names:?}");
        out.join(&names[0])
    }

    /// §45's step 3: the workspace, the sources and anything the command line could have left in the
    /// project all stop existing. The committed fixtures are untouched — these are the copies.
    fn remove_inputs(&self) {
        std::fs::remove_dir_all(self.project()).expect("the project goes away");
        std::fs::remove_dir_all(self.root.join("source")).expect("the sources go away");
    }

    /// Move a finished bundle to another root, which is what a release owner does with a USB stick or a
    /// share, and delete the original.
    fn relocate(&self, bundle: &Path) -> PathBuf {
        let target = self.root.join("relocated").join(
            bundle
                .file_name()
                .unwrap_or_else(|| panic!("a bundle sits in a named folder")),
        );
        copy_tree(bundle, &target);
        std::fs::remove_dir_all(bundle)
            .expect("the original is gone, so nothing verifies by accident");
        target
    }
}

impl Drop for Subject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn git(root: &Path, dir: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_AUTHOR_DATE", COMMIT_DATE)
        .env("GIT_COMMITTER_DATE", COMMIT_DATE)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("HOME", root)
        .env("USERPROFILE", root)
        .output()
        .expect("these tests run against a real repository, so git must be installed");
    assert!(
        output.status.success(),
        "`git {}` failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn head(project: &Path) -> String {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(project)
        .output()
        .expect("git reads its own commit");
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("the destination folder");
    for entry in std::fs::read_dir(from).expect("the source folder is readable") {
        let entry = entry.expect("a readable entry");
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).expect("the file is copied");
        }
    }
}

/// Every file a bundle folder holds, as bundle-relative `/`-separated names.
fn tree(root: &Path) -> Vec<String> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<String>) {
        for entry in std::fs::read_dir(dir).expect("the folder is readable") {
            let entry = entry.expect("a readable entry");
            if entry.path().is_dir() {
                walk(root, &entry.path(), out);
            } else {
                out.push(
                    entry
                        .path()
                        .strip_prefix(root)
                        .expect("the file is inside the bundle")
                        .to_string_lossy()
                        .replace('\\', "/"),
                );
            }
        }
    }
    let mut names = Vec::new();
    walk(root, root, &mut names);
    names.sort();
    names
}

#[test]
fn the_golden_documents_are_the_bytes_the_shipped_binary_wrote() {
    let subject = Subject::new("bytes");
    let bundle = subject.prepare(&subject.root.join("out"));
    for name in GOLDEN_FILES {
        let actual = std::fs::read_to_string(bundle.join(name))
            .unwrap_or_else(|error| panic!("{name} must read: {error}"));
        assert_eq!(
            golden(name),
            actual,
            "{name} differs from the bytes this build writes"
        );
    }
}

#[test]
fn the_golden_holds_every_document_and_no_artifact_bytes() {
    let subject = Subject::new("layout");
    let bundle = subject.prepare(&subject.root.join("out"));
    let mut shipped: Vec<String> = BUNDLE_FILES.iter().map(|name| (*name).to_owned()).collect();
    shipped.sort();
    assert_eq!(
        tree(&bundle),
        shipped,
        "one release of this pair holds exactly these ten files"
    );

    let committed = tree(&repo_root().join("golden/reports/p4-release"));
    let mut documented: Vec<String> = GOLDEN_FILES.iter().map(|name| (*name).to_owned()).collect();
    documented.sort();
    assert_eq!(
        committed, documented,
        "the golden folder holds the eight documents and no artifact bytes"
    );

    let folder = bundle
        .file_name()
        .expect("a bundle sits in a named folder")
        .to_string_lossy()
        .into_owned();
    assert_eq!(
        folder,
        format!(
            "brake-node-1.2.3-{}",
            RELEASE_ID
                .trim_start_matches("release-")
                .get(..12)
                .expect("a release id carries hex")
        ),
        "the proposed folder is the project, the version and the release's own short digest (§29)"
    );
}

#[test]
fn the_shipped_artifact_bytes_are_the_committed_fixtures() {
    // §46 lets the golden skip the artifacts because their bytes are already recorded by hash. That is
    // only a real saving if the bundle's copies are those exact bytes, so they are checked here.
    let subject = Subject::new("fixtures");
    let bundle = subject.prepare(&subject.root.join("out"));
    let manifest =
        std::fs::read_to_string(fixture("fixtures/manifest.json")).expect("the manifest reads");
    let value: serde_json::Value = serde_json::from_str(&manifest).expect("the manifest is JSON");
    for (relative, shipped) in [
        (TARGET_ELF, "artifacts/firmware.elf"),
        (TARGET_MAP, "artifacts/firmware.map"),
    ] {
        let recorded = value["files"]
            .as_array()
            .expect("files is an array")
            .iter()
            .find(|entry| entry["path"] == relative)
            .unwrap_or_else(|| panic!("{relative} is recorded in fixtures/manifest.json"))["sha256"]
            .as_str()
            .expect("a recorded sha256 is a string")
            .to_owned();
        let bytes = std::fs::read(bundle.join(shipped)).expect("the artifact shipped");
        assert_eq!(
            fingerprint::sha256_hex(&bytes),
            recorded,
            "{shipped} is not the committed fixture"
        );
    }
}

#[test]
fn the_two_indexes_cover_each_other_the_way_the_integrity_model_states() {
    let subject = Subject::new("indexes");
    let bundle = subject.prepare(&subject.root.join("out"));

    let sums_text = read(&bundle.join(SHA256SUMS_NAME));
    let mut sums: BTreeMap<&str, &str> = BTreeMap::new();
    for line in sums_text.lines() {
        // The `sha256sum` text format: the digest, two spaces, then the name.
        let (digest, name) = line
            .split_once("  ")
            .expect("a sums line is digest then name");
        sums.insert(name, digest);
    }

    for name in BUNDLE_FILES {
        let covered = if SUMS_EXCLUDED_NAMES.contains(&name) {
            !sums.contains_key(name)
        } else {
            sums.contains_key(name)
        };
        assert!(covered, "`SHA256SUMS` coverage is wrong for `{name}`");
        if let Some(digest) = sums.get(name) {
            assert_eq!(
                *digest,
                fingerprint::sha256_hex(
                    &std::fs::read(bundle.join(name)).expect("a listed file is in the bundle")
                )
                .as_str(),
                "`SHA256SUMS` does not describe the bytes that are there: {name}"
            );
        }
    }

    let manifest: serde_json::Value =
        serde_json::from_str(&read(&bundle.join(MANIFEST_DOC_NAME))).expect("the manifest is JSON");
    let listed: Vec<&str> = manifest["files"]
        .as_array()
        .expect("files is an array")
        .iter()
        .map(|entry| entry["path"].as_str().expect("a path is a string"))
        .collect();
    assert!(
        !listed.contains(&MANIFEST_DOC_NAME),
        "the manifest does not hash itself"
    );
    assert!(
        listed.contains(&SHA256SUMS_NAME),
        "the manifest hashes SHA256SUMS"
    );
    for entry in manifest["files"].as_array().expect("files is an array") {
        let path = entry["path"].as_str().expect("a path is a string");
        assert_eq!(
            entry["sha256"].as_str().expect("a digest is a string"),
            fingerprint::sha256_hex(
                &std::fs::read(bundle.join(path)).expect("a listed file is there")
            ),
            "the manifest does not describe `{path}`"
        );
        assert_eq!(
            entry["size"].as_u64().expect("a size is a number"),
            std::fs::metadata(bundle.join(path))
                .expect("a listed file is there")
                .len(),
            "the manifest's size for `{path}` is not its length"
        );
    }
}

#[test]
fn every_composed_document_satisfies_the_contract_it_declares() {
    let subject = Subject::new("contracts");
    let bundle = subject.prepare(&subject.root.join("out"));
    for name in CONTRACT_DOCUMENTS {
        let text = read(&bundle.join(name));
        let document = schemas::parse(&text);
        let failures = schemas::validate(name, &document)
            .unwrap_or_else(|error| panic!("{name} must have a contract: {error}"));
        assert!(
            failures.is_empty(),
            "{name} breaks its contract: {failures:?}"
        );
    }
}

#[test]
fn a_relocated_bundle_verifies_with_its_sources_and_project_deleted() {
    // §45's whole point: the bundle is the record. Nothing below reads a database, a repository or a
    // source file, and the folder is not where it was written.
    let subject = Subject::new("portability");
    let bundle = subject.prepare(&subject.root.join("out"));
    let copy = subject.relocate(&bundle);
    subject.remove_inputs();

    let verification =
        bundle::verify_bundle(&copy).expect("the relocated bundle verifies from its bytes");
    assert_eq!(verification.release_id, RELEASE_ID);
    assert_eq!(verification.file_count, 10);
    assert_eq!(verification.artifact_count, 2);
    assert!(
        verification.carries_comparison,
        "a baseline was shipped, so diff.json is in it"
    );
    assert!(
        verification.carries_release_notes,
        "the policy required notes, so they are in it"
    );

    // §45's step 5 and 6, done through the same bytes a stranger would open.
    for name in CONTRACT_DOCUMENTS {
        let text = read(&copy.join(name));
        let document = serde_json::from_str::<serde_json::Value>(&text)
            .unwrap_or_else(|error| panic!("{name} parses without the app: {error}"));
        let failures = schemas::validate(name, &document)
            .unwrap_or_else(|error| panic!("{name} must have a contract: {error}"));
        assert!(
            failures.is_empty(),
            "{name} breaks its contract: {failures:?}"
        );
    }

    assert!(
        tree(&copy)
            .iter()
            .all(|name| !name.ends_with(".sqlite") && !name.ends_with(".db")),
        "a portable bundle holds no database"
    );
    assert!(
        bundle::is_recognizable_bundle(&copy).expect("the copy is readable"),
        "the relocated folder still reads as a FirmwareSight bundle"
    );
}

#[test]
fn the_relocated_report_needs_nothing_but_itself() {
    let subject = Subject::new("report");
    let bundle = subject.prepare(&subject.root.join("out"));
    let copy = subject.relocate(&bundle);
    subject.remove_inputs();

    let html = read(&copy.join(REPORT_DOC_NAME));
    assert!(
        html.starts_with("<!doctype html>"),
        "one HTML file, opened by name"
    );
    assert!(html.contains("<style>"), "the CSS is embedded");
    for external in ["<script", "http://", "https://", "<link", "@import", "cdn"] {
        assert!(
            !html.to_ascii_lowercase().contains(external),
            "the report reaches for {external}"
        );
    }
    // §59's steps 34 to 37, read out of the file rather than out of a window.
    assert!(html.contains("FirmwareSight"), "the tool is named");
    assert!(
        html.contains("1.2.3"),
        "the project's release version is named"
    );
    for state in ["PASS", "N/A"] {
        assert!(
            html.contains(state),
            "the finding states are printed, {state} among them"
        );
    }
    assert!(
        html.contains("memory.flash_budget") && html.contains("diff.growth"),
        "the rules the release was judged on are printed"
    );
    assert!(
        html.contains("integrity verified") || html.contains("SHA256SUMS"),
        "the report says how the bundle checks itself"
    );
}

#[test]
fn nothing_this_build_wrote_names_a_host_path_or_a_clock() {
    let subject = Subject::new("paths");
    let bundle = subject.prepare(&subject.root.join("out"));
    let scratch_name = subject
        .root
        .file_name()
        .expect("the scratch folder is named")
        .to_string_lossy()
        .into_owned();
    let root = repo_root().to_string_lossy().into_owned();

    // Only the files FirmwareSight composed. The two files under `artifacts/` and the Release Notes are
    // the release owner's bytes, copied byte for byte because hashing them is what proves the shipped
    // image is the built one (§22) - and this fixture's ELF really does carry the directory its compiler
    // ran in, inside its own DWARF. Redacting that would change the artifact and break the digest that
    // points at it, so the bundle keeps it and states it: see P4_VALIDATION's portability record.
    for name in [
        ACCEPTED_REVIEWS_DOC_NAME,
        ANALYSIS_DOC_NAME,
        DIFF_DOC_NAME,
        GATE_RESULTS_DOC_NAME,
        MANIFEST_DOC_NAME,
        REPORT_DOC_NAME,
        SHA256SUMS_NAME,
    ] {
        let text = read(&bundle.join(name));
        for marker in [
            scratch_name.as_str(),
            root.as_str(),
            "C:\\",
            "D:\\",
            "/home/",
            "/Users/",
            "/tmp/",
            TARGET_ELF,
            BASE_ELF,
            PROJECT_FIXTURE,
        ] {
            assert!(
                !text.contains(marker),
                "`{name}` carries the host path {marker}"
            );
        }
        assert!(
            !carries_a_date_value(&text),
            "`{name}` carries a date-shaped value, so it is not reproducible"
        );
    }
}

/// `YYYY-MM-DD`, optionally followed by a time. Enough to catch a generated timestamp, and cheap enough
/// to keep in a test. A bundle that changed when the clock changed could never be a golden.
fn carries_a_date_value(text: &str) -> bool {
    text.as_bytes().windows(10).any(|window| {
        window[0] == b'2'
            && window[1] == b'0'
            && window[2].is_ascii_digit()
            && window[3].is_ascii_digit()
            && window[4] == b'-'
            && window[5].is_ascii_digit()
            && window[6].is_ascii_digit()
            && window[7] == b'-'
            && window[8].is_ascii_digit()
            && window[9].is_ascii_digit()
    })
}

#[test]
fn the_bundle_answers_every_question_a_stranger_asks_of_it() {
    // §61: if any of these needs FirmwareSight reopened, US-004 is not complete. The answers are read out
    // of the bundle's own files, in the order a person picking it up cold would ask them.
    let subject = Subject::new("questions");
    let bundle = subject.prepare(&subject.root.join("out"));
    let copy = subject.relocate(&bundle);
    subject.remove_inputs();

    let manifest = schemas::parse(&read(&copy.join(MANIFEST_DOC_NAME)));
    assert_eq!(
        manifest["release"]["id"], RELEASE_ID,
        "what release is this"
    );
    assert_eq!(manifest["release"]["version"], "1.2.3", "which version");
    assert!(
        manifest["build"]["artifact_sha256"]
            .as_str()
            .is_some_and(|digest| digest.len() == 64),
        "what artifact ships, by digest"
    );
    assert!(
        manifest["files"]
            .as_array()
            .expect("files is an array")
            .iter()
            .any(|entry| entry["path"] == format!("{ARTIFACTS_DIR}/firmware.elf")),
        "where the shipped image sits inside the folder"
    );
    assert_eq!(
        manifest["extensions"]["integrity_model"]["sums_file"], SHA256SUMS_NAME,
        "how do I verify these files"
    );

    let analysis = schemas::parse(&read(&copy.join(ANALYSIS_DOC_NAME)));
    assert_eq!(
        analysis["schemaVersion"], 1,
        "the analysis says which contract it is"
    );
    assert!(
        !analysis["sections"]
            .as_array()
            .expect("the analysis lists its sections")
            .is_empty(),
        "what the analysis said"
    );
    assert!(
        analysis["memory"]["nonvolatileImageFootprint"]["bytes"] == 376,
        "the footprint the budgets were judged against is in the bundle"
    );
    let diff = schemas::parse(&read(&copy.join(DIFF_DOC_NAME)));
    assert_eq!(
        diff["counts"]["sectionsChanged"], 16,
        "what changed from the baseline"
    );
    assert_eq!(
        diff["memory"]["nonvolatile"]["delta"], 120,
        "the growth the Gate reported is the growth the diff measured"
    );
    let gate = schemas::parse(&read(&copy.join(GATE_RESULTS_DOC_NAME)));
    assert_eq!(
        gate["extensions"]["overall_effective_severity"], "PASS",
        "what the Gate said"
    );
    assert_eq!(
        gate["findings"]
            .as_array()
            .expect("the Gate's findings are an array")
            .len(),
        10,
        "every rule is in the record, not only the ones that failed"
    );
    let reviews = schemas::parse(&read(&copy.join(ACCEPTED_REVIEWS_DOC_NAME)));
    assert!(
        reviews["acceptances"].is_array(),
        "which reviews were accepted - an empty list is still that answer"
    );
    assert_eq!(
        read(&copy.join("release-notes.md")),
        read(&fixture(&format!(
            "{PROJECT_FIXTURE}/docs/RELEASE_NOTES.md"
        ))),
        "what the Release Notes said, byte for byte"
    );
}

#[test]
fn the_prose_this_build_wrote_never_claims_authenticity() {
    // §62: a SHA-256 proves the bytes match the manifest that lists them. It says nothing about who put
    // them there, so none of these words may appear in a document FirmwareSight composed. The shipped
    // artifact and the Release Notes are exempt for a reason rather than by omission: their bytes are the
    // release owner's, and an embedded debug string is not a claim this product made.
    let subject = Subject::new("claims");
    let bundle = subject.prepare(&subject.root.join("out"));
    let composed = [
        ACCEPTED_REVIEWS_DOC_NAME,
        ANALYSIS_DOC_NAME,
        DIFF_DOC_NAME,
        GATE_RESULTS_DOC_NAME,
        MANIFEST_DOC_NAME,
        REPORT_DOC_NAME,
    ];
    for name in composed {
        let text = read(&bundle.join(name)).to_ascii_lowercase();
        for claim in [
            "trusted",
            "authentic",
            "signed",
            "tamper-proof",
            "tamperproof",
        ] {
            assert!(!mentions_a_word(&text, claim), "`{name}` claims {claim}");
        }
    }
}

/// Whether `needle` appears as a word rather than inside one. `unsigned_offset_table` is a section name;
/// `unsigned` as a claim about provenance is the thing §62 forbids, and only a word boundary tells them
/// apart.
fn mentions_a_word(text: &str, needle: &str) -> bool {
    text.match_indices(needle).into_iter().any(|(at, _)| {
        let before = text[..at]
            .chars()
            .next_back()
            .is_none_or(|c| !c.is_alphanumeric());
        let after = text[at + needle.len()..]
            .chars()
            .next()
            .is_none_or(|c| !c.is_alphanumeric());
        before && after
    })
}

#[test]
fn the_cli_writes_no_database_and_leaves_the_subject_untouched() {
    // §56: `release prepare` is a packager, not a record keeper. The desktop owns the release record
    // because only the desktop holds a review history to attach it to.
    let subject = Subject::new("side-effects");
    let before = tree(&subject.project());
    let bundle = subject.prepare(&subject.root.join("out"));
    assert_eq!(
        before,
        tree(&subject.project()),
        "the release subject was not written to"
    );
    assert!(tree(&bundle).iter().all(|name| !name.ends_with(".sqlite")));
    assert!(
        !subject.root.join("out").join("out.sqlite").exists(),
        "no database beside the bundle"
    );
    let stray: Vec<String> = tree(&subject.root.join("out"))
        .into_iter()
        .filter(|name| !name.starts_with("brake-node-1.2.3-"))
        .collect();
    assert!(
        stray.is_empty(),
        "the destination holds one bundle and nothing else: {stray:?}"
    );
}
