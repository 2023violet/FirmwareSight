//! P4 Release Bundle over IPC: what the boundary may say, what it writes, and what it refuses to touch.
//!
//! The engine that assembles, stages, publishes and verifies a bundle is `firmwaresight-project`'s and is
//! tested there at length; the CLI drives the same engine over the same facts. What this file proves is the
//! seam specific to the desktop shell (§57):
//!
//! - a plan is prepared against *this* session's run, so a moved workspace, config or notes changes the run
//!   id and the refusal arrives as `ERR-BUNDLE-6102` rather than as a written bundle;
//! - the preview is bounded and path-free in both directions, and the plan id and destination token are
//!   session-local handles the WebView cannot resolve into a folder (`AGENTS.md` 7, prompt §27, §29);
//! - an export writes only after a person chose a folder, replaces only a recognizable bundle, and records
//!   the release only once the bytes are on disk (prompt §30, §31, §57);
//! - and every refusal leaves the destination exactly as it found it.
//!
//! These call the same `Session` methods the commands do, with no window and no runtime — the way the P0,
//! P1, P2 and P3 files do. The folder a dialog would have returned is handed straight to
//! `choose_bundle_destination`, which is the pure half of that command. Every project here is a throwaway
//! repository under the system temp directory: FirmwareSight's own repository is never a release subject.

use std::path::{Path, PathBuf};
use std::process::Command;

use firmwaresight_artifact::pipeline::FWSIGHT_VERSION;
use firmwaresight_desktop::Session;
use firmwaresight_desktop::ipc::{
    AcceptReviewRequestDto, BundleDestinationDto, BundleExportDto, BundlePlanRequestDto,
    BundlePreviewDto, ErrorEnvelopeDto, GateFindingRowDto, GateRunDto, GateRunRequestDto,
};
use firmwaresight_desktop::preferences::{
    ApplicationPreferences, LanguagePref, SETTINGS_FILE_NAME, SizeUnitPref, StartupPagePref, Store,
    ThemePref,
};
use firmwaresight_desktop::service::FixtureCatalog;
use firmwaresight_project::{fingerprint, verify_bundle};
use firmwaresight_storage::{
    Database, ReleaseRecordDraft, ReleaseRecordWrite, StoredReleaseRecord,
};
use serde_json::Value;

/// The marker in every temporary folder this file makes, so a leak of one is detectable by name.
const TEMP_MARKER: &str = "fwsight-p4-bundle";

/// The order the preview lists the bundle's files in: Core's single bundle-path rule, which is what lets a UI
/// table be stable without sorting anything itself. The *write* order is §41's, and the two indexes the
/// manifest and `SHA256SUMS` use are this same order.
const PREVIEW_ORDER: [&str; 10] = [
    "accepted-reviews.json",
    "analysis.json",
    "artifacts/firmware.elf",
    "artifacts/firmware.map",
    "diff.json",
    "gate-results.json",
    "release-manifest.json",
    "release-notes.md",
    "release-report.html",
    "SHA256SUMS",
];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
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

fn temp_name(label: &str, extension: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "{TEMP_MARKER}-{label}-{}-{:x}{extension}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|value| value.as_nanos())
            .unwrap_or(0)
    ))
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

/// A project whose policy a bundle can be released under: both fixture kinds required, a version policy the
/// workspace tag answers, an expected version the Gate endorses, and Release Notes the policy names.
fn release_policy() -> String {
    policy_with_growth_review(None)
}

/// The same policy with a flash-growth threshold, which makes `diff.growth` report `REVIEW` for this fixture
/// pair and leaves the release owner one finding to accept before a bundle exists.
fn policy_with_growth_review(review_bytes: Option<u64>) -> String {
    let mut text = r#"schema_version = 1

[project]
name = "brake-node"

[artifacts]
required = ["elf", "map"]

[version]
source = "git_tag"
pattern = '^v(?P<version>\d+\.\d+\.\d+)$'

[release]
require_clean_git = true
require_release_notes = true
release_notes_path = "docs/RELEASE_NOTES.md"
expected_version = "1.2.3"
"#
    .to_owned();
    if let Some(bytes) = review_bytes {
        text.push_str(&format!("\n[diff]\nflash_growth_review_bytes = {bytes}\n"));
    }
    text
}

/// The same policy with a different `[artifacts] required` line, for a run whose requirements can only be
/// satisfied by something other than the analyzed build.
fn policy_requiring(kinds: &[&str]) -> String {
    let quoted: Vec<String> = kinds.iter().map(|kind| format!("\"{kind}\"")).collect();
    release_policy().replace(
        r#"required = ["elf", "map"]"#,
        &format!("required = [{}]", quoted.join(", ")),
    )
}

/// One release subject: a repository holding a policy and its notes, and the folder the release owner would
/// have chosen in the dialog.
struct Release {
    project: PathBuf,
    destination: PathBuf,
}

impl Release {
    fn new(label: &str, policy: &str) -> Self {
        let project = temp_name(label, "");
        std::fs::create_dir_all(project.join("docs")).expect("a temporary project directory");
        std::fs::write(project.join("firmwaresight.toml"), policy).expect("a temporary config");
        std::fs::write(
            project.join("docs/RELEASE_NOTES.md"),
            "# 1.2.3\n\n- the modem driver is now in the image\n",
        )
        .expect("a notes file");
        git(&["init", "-q"], &project);
        git(
            &["config", "user.email", "release@example.invalid"],
            &project,
        );
        git(&["config", "user.name", "Release Owner"], &project);
        git(&["add", "-A"], &project);
        git(&["commit", "-q", "-m", "release candidate"], &project);
        git(&["tag", "v1.2.3"], &project);
        Self {
            project,
            destination: temp_name(&format!("{label}-out"), ""),
        }
    }

    fn config_path(&self) -> PathBuf {
        self.project.join("firmwaresight.toml")
    }

    /// What a release owner did after the preview: the workspace moved.
    fn dirty(&self) {
        std::fs::write(self.project.join("uncommitted.txt"), "not committed\n")
            .expect("a new file");
    }

    /// The notes the Gate read are replaced with other bytes.
    fn rewrite_notes(&self) {
        std::fs::write(
            self.project.join("docs/RELEASE_NOTES.md"),
            "# 1.2.3\n\n- and the modem driver was reverted again\n",
        )
        .expect("the notes are rewritten");
    }
}

impl Drop for Release {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.project);
        let _ = std::fs::remove_dir_all(&self.destination);
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

/// A session, its database, the project whose policy it runs under, and the two builds it analyzed.
struct World {
    db: TempDb,
    session: Session,
    release: Release,
    target: String,
    baseline: String,
}

impl World {
    /// One complete release subject: target and baseline analyzed in this session, policy loaded, and one
    /// Gate run stored — the state a release owner is standing on when they press Prepare.
    fn new(label: &str, policy: &str) -> Self {
        let db = TempDb::new(label);
        let session =
            Session::open(FixtureCatalog::from_environment(), &db.0).expect("a session opens");
        let release = Release::new(label, policy);
        load(&session, &release);
        let target = store(&session, "target");
        let baseline = store(&session, "base");
        let _ = run(&session, &target, Some(&baseline));
        Self {
            db,
            session,
            release,
            target,
            baseline,
        }
    }

    fn gate_run(&self) -> GateRunDto {
        run(&self.session, &self.target, Some(&self.baseline))
    }

    /// §26 through the same method the command calls.
    fn prepare(&self) -> Result<BundlePreviewDto, ErrorEnvelopeDto> {
        self.session.prepare_bundle(
            &BundlePlanRequestDto {
                snapshot_id: self.target.clone(),
                baseline_snapshot_id: Some(self.baseline.clone()),
                gate_run_id: self.gate_run().run_id,
            },
            "op-prepare",
        )
    }

    /// The dialog half, with the folder a person would have chosen handed to the pure function.
    fn choose(&self, plan_id: &str) -> Option<BundleDestinationDto> {
        self.session
            .choose_bundle_destination(plan_id, Some(self.release.destination.clone()), "op-choose")
            .expect("choosing a folder is a decision, not a failure")
    }

    fn export(
        &self,
        plan_id: &str,
        token: &str,
        overwrite: bool,
    ) -> Result<BundleExportDto, ErrorEnvelopeDto> {
        self.session
            .export_bundle(plan_id, token, overwrite, "op-export")
    }

    /// Prepare, choose and export in the order the screen offers them.
    fn publish(&self, overwrite: bool) -> BundleExportDto {
        let preview = self.prepare().expect("the release prepares");
        let chosen = self
            .choose(&preview.plan_id)
            .expect("a chosen folder is not the cancel path");
        self.export(&preview.plan_id, &chosen.destination_token, overwrite)
            .expect("the bundle exports")
    }

    fn bundle_dir(&self, outcome: &BundleExportDto) -> PathBuf {
        self.release.destination.join(&outcome.folder_display_name)
    }

    fn record(&self, release_id: &str) -> Option<StoredReleaseRecord> {
        let db = Database::open(&self.db.0).expect("the same database reopens");
        db.release_record_by_id(release_id)
            .expect("the release record is readable")
    }
}

/// A session with no project policy loaded, for the refusal that has to happen first.
fn bare_session(label: &str) -> (TempDb, Session) {
    let db = TempDb::new(label);
    let session =
        Session::open(FixtureCatalog::from_environment(), &db.0).expect("a session opens");
    (db, session)
}

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

fn load(session: &Session, release: &Release) {
    session
        .load_project_from(Some(release.config_path()), "op-open")
        .expect("the config opens")
        .expect("a chosen path is not the cancel path");
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

/// Every shape a host path takes on this machine: the temp folder these releases live in, the repository the
/// fixtures came from, and a separator only a native Windows path uses.
fn host_path_in(text: &str) -> Option<String> {
    [
        std::env::temp_dir().to_string_lossy().into_owned(),
        repo_root().to_string_lossy().into_owned(),
        "\\".to_owned(),
    ]
    .into_iter()
    .find(|marker| text.contains(marker))
}

fn entries(parent: &Path) -> Vec<String> {
    std::fs::read_dir(parent)
        .expect("the folder is readable")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect()
}

// --------------------------------------------------------------------------- prepare (§26, §27)

#[test]
fn a_passing_run_prepares_a_bounded_preview_and_writes_nothing() {
    let world = World::new("prepare", &release_policy());
    let preview = world.prepare().expect("a PASS run prepares");

    assert_eq!(preview.disposition, "PASS");
    assert_eq!(preview.release_version, "1.2.3");
    assert_eq!(preview.snapshot_id, world.target);
    assert_eq!(
        preview.baseline_snapshot_id.as_deref(),
        Some(world.baseline.as_str())
    );
    assert!(
        preview.release_id.starts_with("release-"),
        "{}",
        preview.release_id
    );
    assert_eq!(preview.release_id.chars().count(), 8 + 64);
    assert!(preview.gate_run_id.starts_with("gate-"));
    assert_eq!(preview.accepted_review_count, 0);
    assert!(
        preview.plan_id.starts_with("bundle-"),
        "{}",
        preview.plan_id
    );

    let paths: Vec<&str> = preview.files.iter().map(|row| row.path.as_str()).collect();
    assert_eq!(paths, PREVIEW_ORDER.to_vec(), "the preview's own order");
    for row in &preview.files {
        assert_eq!(row.sha256.chars().count(), 64, "{}", row.path);
        assert!(row.byte_size > 0, "{}", row.path);
        assert!(!row.role.is_empty(), "{}", row.path);
        // Nothing here was attached, so no row may borrow the attachment's word: the new role exists to
        // separate the two classes, and a leak into an attachment-free release would be its own lie.
        if row.path.starts_with("artifacts/") {
            assert_eq!(row.role, "artifact", "{}", row.path);
        }
        assert!(
            !row.path.starts_with('/') && !row.path.contains(':') && !row.path.contains("\\"),
            "{} is not a bundle-relative path",
            row.path
        );
    }

    // §26's last line: the preview is an authorization, not a write.
    assert!(
        !world.release.destination.exists(),
        "prepare created the folder it was not asked to touch"
    );
}

#[test]
fn the_preview_names_no_host_path_in_either_direction() {
    let world = World::new("preview-paths", &release_policy());
    let preview = world.prepare().expect("a PASS run prepares");
    let text = serde_json::to_string(&preview).expect("an IPC payload is JSON");
    if let Some(marker) = host_path_in(&text) {
        panic!("the preview carries {marker}: {text}");
    }
    // The release owner's notes file is named as it will appear inside the bundle, never where it is.
    assert!(text.contains("release-notes.md"), "{text}");
    assert!(!text.contains("RELEASE_NOTES.md"), "{text}");
}

#[test]
fn a_run_that_has_not_passed_produces_no_plan() {
    // §33's rule seen from the desktop: a REVIEW finding nobody accepted is not a release, and the shell says
    // which disposition stopped it.
    let world = World::new("review", &policy_with_growth_review(Some(100)));
    assert_eq!(finding(&world.gate_run(), "diff.growth").state, "REVIEW");

    let error = world
        .prepare()
        .expect_err("an unaccepted review is not a release");
    assert_eq!(error.code, "ERR-BUNDLE-6101");
    assert!(error.message.contains("REVIEW"), "{}", error.message);
    assert!(error.remediation.is_some());
    assert!(!world.release.destination.exists());
}

#[test]
fn an_accepted_review_lets_the_same_run_prepare_and_counts_it() {
    let world = World::new("accepted", &policy_with_growth_review(Some(100)));
    let run = world.gate_run();
    let growth = finding(&run, "diff.growth");
    assert_eq!(growth.state, "REVIEW");
    world
        .session
        .accept_gate_review(
            &AcceptReviewRequestDto {
                run_id: run.run_id.clone(),
                finding_id: growth.id.clone(),
                actor: "release-owner".to_owned(),
                reason: "the modem driver is expected to grow".to_owned(),
            },
            "op-accept",
        )
        .expect("a named acceptance lands");

    let preview = world.prepare().expect("the accepted run prepares");
    assert_eq!(preview.disposition, "PASS");
    assert_eq!(preview.accepted_review_count, 1);
    assert!(
        preview
            .files
            .iter()
            .any(|row| row.path == "diff.json" && row.role == "comparison"),
        "the comparison ships with the review it explains: {:?}",
        preview.files
    );
}

#[test]
fn a_build_this_session_never_analyzed_cannot_be_packaged() {
    let world = World::new("unanalyzed", &release_policy());
    let error = world
        .session
        .prepare_bundle(
            &BundlePlanRequestDto {
                snapshot_id: format!("snap-{}", "0".repeat(64)),
                baseline_snapshot_id: None,
                gate_run_id: world.gate_run().run_id,
            },
            "op-prepare",
        )
        .expect_err("a snapshot id is not a snapshot");
    assert_eq!(error.code, "ERR-BUNDLE-6114");
    // The id is shortened in the message: a person cannot act on a 64-character digest they did not type.
    assert!(!error.message.contains(&world.target), "{}", error.message);
}

#[test]
fn a_run_that_was_never_stored_produces_no_plan() {
    let world = World::new("unknown-run", &release_policy());
    let error = world
        .session
        .prepare_bundle(
            &BundlePlanRequestDto {
                snapshot_id: world.target.clone(),
                baseline_snapshot_id: None,
                gate_run_id: format!("gate-{}", "0".repeat(64)),
            },
            "op-prepare",
        )
        .expect_err("an id nobody stored is not a run");
    assert_eq!(error.code, "ERR-STORAGE-4005");
}

#[test]
fn a_policy_that_was_never_loaded_produces_no_plan() {
    // §26 step 1. The release version, the policy fingerprint and the notes all come from the project's own
    // config, so with none loaded there is no release to name.
    let (_db, session) = bare_session("no-policy");
    let target = store(&session, "target");
    let error = session
        .prepare_bundle(
            &BundlePlanRequestDto {
                snapshot_id: target,
                baseline_snapshot_id: None,
                gate_run_id: "gate-unrecorded".to_owned(),
            },
            "op-prepare",
        )
        .expect_err("no policy is no release");
    assert_eq!(error.code, "ERR-BUNDLE-6112");
}

// --------------------------------------------------------------------------- export (§29, §30, §31)

#[test]
fn an_export_writes_the_canonical_layout_and_verifies_from_its_own_bytes() {
    let world = World::new("export", &release_policy());
    let outcome = world.publish(false);

    assert_eq!(outcome.release_version, "1.2.3");
    assert_eq!(outcome.artifact_count, 2);
    assert_eq!(outcome.file_count, PREVIEW_ORDER.len());
    assert_eq!(outcome.manifest_sha256.chars().count(), 64);
    assert!(!outcome.replaced);
    assert!(outcome.record_written);
    assert!(
        !outcome.folder_display_name.contains('/') && !outcome.folder_display_name.contains('\\'),
        "a display name, not a path: {}",
        outcome.folder_display_name
    );

    let bundle = world.bundle_dir(&outcome);
    let verification = firmwaresight_project::verify_bundle(&bundle)
        .unwrap_or_else(|error| panic!("the exported bundle does not verify: {error}"));
    assert_eq!(verification.release_id, outcome.release_id);
    assert_eq!(verification.file_count, outcome.file_count);
    assert!(verification.carries_comparison);
    assert!(verification.carries_release_notes);

    // The folder a person chose holds exactly one thing: the bundle. No staging sibling, no backup.
    assert_eq!(
        entries(&world.release.destination),
        vec![outcome.folder_display_name.clone()],
        "a published bundle leaves no working files behind"
    );
}

#[test]
fn the_release_record_lands_only_after_the_bytes_do() {
    let world = World::new("record", &release_policy());
    let preview = world.prepare().expect("the release prepares");
    assert!(
        world.record(&preview.release_id).is_none(),
        "a preview records nothing"
    );

    let chosen = world
        .choose(&preview.plan_id)
        .expect("a chosen folder is not the cancel path");
    let refused = world
        .export(&preview.plan_id, "dst-never-issued", false)
        .expect_err("a token this process never issued names no folder");
    assert_eq!(refused.code, "ERR-BUNDLE-6113");
    assert!(world.record(&preview.release_id).is_none());
    assert!(!world.release.destination.exists(), "and writes nothing");

    let outcome = world
        .export(&preview.plan_id, &chosen.destination_token, false)
        .expect("the bundle exports");
    let record = world
        .record(&outcome.release_id)
        .expect("the release is recorded");
    assert_eq!(record.manifest_sha256, outcome.manifest_sha256);
    assert_eq!(record.release_version, outcome.release_version);
    assert_eq!(record.gate_run_id, preview.gate_run_id);
    assert!(
        record.baseline_build_id.is_some(),
        "this release was judged against a baseline"
    );

    // Recording the same release twice is the dedupe §37 asks for, not a second row.
    let mut db = Database::open(&world.db.0).expect("the database reopens");
    let draft = ReleaseRecordDraft {
        release_id: &outcome.release_id,
        build_id: &record.build_id,
        baseline_build_id: record.baseline_build_id.as_deref(),
        gate_run_id: &record.gate_run_id,
        release_version: &outcome.release_version,
        manifest_sha256: &outcome.manifest_sha256,
    };
    assert_eq!(
        db.persist_release_record(&draft)
            .expect("the same facts re-record"),
        ReleaseRecordWrite::AlreadyStored
    );
}

#[test]
fn an_existing_destination_is_refused_until_the_owner_confirms() {
    let world = World::new("existing", &release_policy());
    let first = world.publish(false);
    let bundle = world.bundle_dir(&first);
    let before = std::fs::read(bundle.join("release-manifest.json")).expect("the first manifest");

    // A fresh plan against a folder that now holds that bundle, exported without confirmation.
    let preview = world.prepare().expect("the release prepares again");
    let chosen = world
        .choose(&preview.plan_id)
        .expect("a chosen folder is not the cancel path");
    assert!(chosen.exists, "the folder the preview proposes is occupied");
    assert_eq!(chosen.bundle_folder_name, first.folder_display_name);
    assert!(
        chosen.recognizable_bundle,
        "and what occupies it is a bundle this engine wrote"
    );

    let refused = world
        .export(&preview.plan_id, &chosen.destination_token, false)
        .expect_err("no implicit replacement");
    assert_eq!(refused.code, "ERR-BUNDLE-6106");
    assert!(
        refused.message.contains(&chosen.bundle_folder_name),
        "{}",
        refused.message
    );
    assert_eq!(
        std::fs::read(bundle.join("release-manifest.json")).expect("the untouched manifest"),
        before,
        "a refused export leaves the bundle that was there alone"
    );

    let forced = world
        .export(&preview.plan_id, &chosen.destination_token, true)
        .expect("the confirmed replacement writes");
    assert!(forced.replaced);
    assert!(
        firmwaresight_project::verify_bundle(&bundle).is_ok(),
        "the replacement is a bundle in its own right"
    );
    assert_eq!(
        entries(&world.release.destination),
        vec![forced.folder_display_name.clone()],
        "the backup is discarded only once the new bundle is in place (§31)"
    );
}

#[test]
fn overwrite_never_replaces_a_folder_this_engine_did_not_write() {
    let world = World::new("stranger", &release_policy());
    let preview = world.prepare().expect("the release prepares");
    std::fs::create_dir_all(world.release.destination.join(&preview.bundle_folder_name))
        .expect("somebody else's folder is in the way");
    let kept = world
        .release
        .destination
        .join(&preview.bundle_folder_name)
        .join("field-notes.md");
    std::fs::write(&kept, "not a release\n").expect("a file that is not a bundle");

    let chosen = world
        .choose(&preview.plan_id)
        .expect("a chosen folder is not the cancel path");
    assert!(chosen.exists);
    assert!(
        !chosen.recognizable_bundle,
        "the shell says what it found before the release owner confirms anything"
    );

    let error = world
        .export(&preview.plan_id, &chosen.destination_token, true)
        .expect_err("a stranger's folder is not eligible");
    assert_eq!(error.code, "ERR-BUNDLE-6107");
    assert_eq!(
        std::fs::read_to_string(&kept).expect("the stranger's file"),
        "not a release\n",
        "and the refused run deleted nothing"
    );
}

#[test]
fn a_moved_workspace_invalidates_the_run_before_anything_is_written() {
    // §57's revalidation case, and the reason a plan cannot be carried over a change: the recomputed run id
    // is not the one the preview was built from, so §9 refuses with 6102 and the folder stays empty.
    let world = World::new("moved", &release_policy());
    let preview = world.prepare().expect("the release prepares");
    let chosen = world
        .choose(&preview.plan_id)
        .expect("a chosen folder is not the cancel path");

    world.release.dirty();
    let error = world
        .export(&preview.plan_id, &chosen.destination_token, false)
        .expect_err("the workspace moved after the run");
    assert_eq!(error.code, "ERR-BUNDLE-6102");
    assert!(
        !world.release.destination.exists(),
        "and nothing was written"
    );
}

#[test]
fn rewritten_notes_are_refused_rather_than_shipped_under_the_old_digest() {
    let world = World::new("notes", &release_policy());
    let preview = world.prepare().expect("the release prepares");
    let chosen = world
        .choose(&preview.plan_id)
        .expect("a chosen folder is not the cancel path");

    world.release.rewrite_notes();
    let error = world
        .export(&preview.plan_id, &chosen.destination_token, false)
        .expect_err("the notes the Gate read are gone");
    // A changed notes file changes the run the Gate judged (§9) and the input the plan was built from (§28);
    // either answer is a refusal, and neither one is a silent copy of new bytes under an old digest.
    assert!(
        error.code == "ERR-BUNDLE-6102" || error.code == "ERR-BUNDLE-6104",
        "unexpected refusal: {}",
        error.code
    );
    assert!(!world.release.destination.exists());
}

#[test]
fn a_plan_this_session_dropped_is_refused_by_code() {
    let world = World::new("dropped", &release_policy());
    let error = world
        .export("bundle-1-9999", "dst-1-1", false)
        .expect_err("an id this process never issued names no plan");
    assert_eq!(error.code, "ERR-BUNDLE-6105");
    assert_eq!(error.message, "That bundle plan is no longer available.");
}

#[test]
fn cancelling_the_destination_leaves_the_plan_and_writes_nothing() {
    // §30: "Cancel: normal non-error". The plan survives, the screen keeps its preview, and no folder is
    // created to cancel into.
    let world = World::new("cancel", &release_policy());
    let preview = world.prepare().expect("the release prepares");
    let cancelled = world
        .session
        .choose_bundle_destination(&preview.plan_id, None, "op-cancel")
        .expect("a cancelled dialog is not a failure");
    assert!(cancelled.is_none());
    assert!(!world.release.destination.exists());

    // And the plan is still there to be pointed at a folder later in the same session.
    let chosen = world
        .choose(&preview.plan_id)
        .expect("a later choice still resolves to the same plan");
    let outcome = world
        .export(&preview.plan_id, &chosen.destination_token, false)
        .expect("the bundle exports");
    assert_eq!(outcome.file_count, PREVIEW_ORDER.len());
}

#[test]
fn the_exported_bundle_names_its_producer_and_nothing_else_about_its_origin() {
    let world = World::new("provenance", &release_policy());
    let outcome = world.publish(false);
    let bundle = world.bundle_dir(&outcome);

    let text = std::fs::read_to_string(bundle.join("release-manifest.json"))
        .expect("the manifest is text");
    if let Some(marker) = host_path_in(&text) {
        panic!("the manifest carries {marker}");
    }
    let manifest: Value = serde_json::from_str(&text).expect("one document");
    assert_eq!(manifest["generated_by"]["product"], "FirmwareSight");
    assert_eq!(
        manifest["generated_by"]["version"], FWSIGHT_VERSION,
        "§43: the build that packaged this, kept distinct from the project's release version"
    );
    assert_eq!(manifest["release"]["version"], "1.2.3");
}

#[test]
fn no_desktop_surface_of_a_bundle_carries_the_folder_it_was_written_to() {
    // §29 and §50 together: after a successful export the UI knows the bundle's own directory name and the
    // manifest digest, and nothing that would let it name the parent.
    let world = World::new("no-destination-path", &release_policy());
    let outcome = world.publish(false);
    let text = serde_json::to_string(&outcome).expect("an IPC payload is JSON");
    if let Some(marker) = host_path_in(&text) {
        panic!("the export outcome carries {marker}: {text}");
    }
    assert!(text.contains(&outcome.folder_display_name), "{text}");
    assert!(!text.contains(TEMP_MARKER), "{text}");
}

// ------------------------------------------------------------------ release attachments (C1-U4)

/// Bytes a release owner would have chosen in the dialog, in a folder of their own. They are never written
/// inside the project repository: this world's policy requires a clean workspace, and an uncommitted file
/// would make the Gate answer about Git rather than about attachments.
struct ChosenFiles(PathBuf);

impl ChosenFiles {
    fn new(label: &str) -> Self {
        let dir = temp_name(label, "");
        std::fs::create_dir_all(&dir).expect("a folder for chosen files");
        Self(dir)
    }

    fn write(&self, name: &str, bytes: &[u8]) -> PathBuf {
        let path = self.0.join(name);
        std::fs::write(&path, bytes).expect("a chosen file");
        path
    }
}

impl Drop for ChosenFiles {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn attach(session: &Session, path: &Path) -> String {
    session
        .attach_release_file(Some(path.to_path_buf()), "op-attach")
        .expect("a chosen file is observed or refused with its own code")
        .expect("a dialog that returned a file is not the cancel path")
        .attachment_id
}

#[test]
fn an_attached_bin_and_hex_are_judged_stored_shipped_and_verified_from_their_bytes() {
    let world = World::new("attach-chain", &release_policy());
    let chosen = ChosenFiles::new("attach-chain");
    let bin = chosen.write("app.bin", b"APP-BYTES");
    let hex = chosen.write("boot.hex", b":00000001FF");

    let row = world
        .session
        .attach_release_file(Some(bin.clone()), "op-attach")
        .expect("the BIN is observed")
        .expect("a file was chosen");
    assert_eq!(row.file_name, "app.bin");
    assert_eq!(row.kind, "bin", "the name declared it");
    assert_eq!(row.kind_basis, "declared");
    assert_eq!(row.provenance, "unknown");
    assert_eq!(row.byte_size, b"APP-BYTES".len() as u64);
    assert_eq!(row.sha256.len(), 64, "the full digest, not a shortened one");
    attach(&world.session, &hex);

    let run = world.gate_run();
    let set = world
        .session
        .release_attachments(Some(&run.run_id), "op-read")
        .expect("the set reads back");
    assert_eq!(set.attachments.len(), 2);
    assert_eq!(
        set.matches_gate_run,
        Some(true),
        "the set the page holds is the set this run judged"
    );

    // The two rows this policy requires name where each came from, in the same words the rule used.
    let tables = run.tables.as_ref().expect("a fresh run carries its tables");
    for kind in ["elf", "map"] {
        let row = tables
            .artifacts
            .iter()
            .find(|row| row.kind == kind)
            .unwrap_or_else(|| panic!("the {kind} row is listed"));
        assert_eq!(row.satisfied_by, "analyzed", "{kind}");
    }

    // The preview is where a person authorizes the export, so it has to say which of these bytes were
    // analyzed and which were only hashed. Both ship under `artifacts/`, and one word for both would ask
    // the release owner to guess.
    let preview = world.prepare().expect("a passing run previews");
    let role = |path: &str| {
        preview
            .files
            .iter()
            .find(|row| row.path == path)
            .unwrap_or_else(|| panic!("{path} is listed in the preview"))
            .role
            .clone()
    };
    assert_eq!(role("artifacts/firmware.elf"), "artifact");
    assert_eq!(
        role("artifacts/app.bin"),
        "release-attachment",
        "the attached bytes are named for what they are"
    );
    assert_eq!(role("artifacts/boot.hex"), "release-attachment");

    let outcome = world.publish(false);
    let bundle = world.bundle_dir(&outcome);
    let verification =
        verify_bundle(&bundle).expect("a bundle that shipped attachments still verifies");
    assert_eq!(verification.release_id, outcome.release_id);

    let manifest = document(&bundle, "release-manifest.json");
    let rows = manifest["extensions"]["attachments"]
        .as_array()
        .expect("the manifest discloses what it shipped")
        .clone();
    assert_eq!(rows.len(), 2, "{rows:?}");
    for row in &rows {
        let path = row["path"].as_str().expect("a bundle-relative locator");
        let shipped = bundle.join(path);
        assert!(shipped.is_file(), "{path} is disclosed but not shipped");
        assert_eq!(
            fingerprint::file_sha256(&shipped).expect("the shipped bytes re-hash"),
            row["sha256"].as_str().expect("a digest"),
            "{path} ships bytes that are not the ones the manifest disclosed"
        );
        assert_eq!(row["provenance"], serde_json::json!("unknown"));
    }

    let gate = document(&bundle, "gate-results.json");
    assert_eq!(
        gate["extensions"]["canonical_input_label"],
        serde_json::json!("firmwaresight-gate-input/2"),
        "the shipped document names the grammar its own run id was taken over"
    );
    assert_eq!(
        gate["run_id"], manifest["extensions"]["gate_run_id"],
        "one bundle cannot gate one run and ship another's verdict"
    );

    // §9's line in the report the person reads, with the count this bundle actually carries.
    let report =
        std::fs::read_to_string(bundle.join("release-report.html")).expect("the report is written");
    assert!(
        report.contains("2 release attachment(s) ship as byte evidence"),
        "the limits section has to say what was not analyzed: {report}"
    );
}

#[test]
fn a_required_kind_an_attachment_carries_is_listed_as_carried_and_not_as_missing() {
    // The measured defect, from the installed walk: with `bin` and `hex` required and both attached,
    // the verdict read `PASS ... bin and hex from release attachments` while the table under it read
    // `Missing`, because the table looked only at the analyzed snapshot. Both now ask the one
    // predicate that decides which set carries which kind (`requirement_evidence`), so the table
    // cannot disagree with the finding it is there to explain (`04_TECH/28` §9).
    let world = World::new("attach-table", &policy_requiring(&["elf", "bin", "hex"]));
    let chosen = ChosenFiles::new("attach-table");
    let bin = chosen.write("app.bin", b"APP-BYTES");
    let hex = chosen.write("boot.hex", b":00000001FF");
    attach(&world.session, &bin);
    attach(&world.session, &hex);

    let run = world.gate_run();
    assert_eq!(run.counts.block, 0, "both kinds were attached");
    let tables = run.tables.as_ref().expect("a fresh run carries its tables");
    let row = |kind: &str| {
        tables
            .artifacts
            .iter()
            .find(|row| row.kind == kind)
            .cloned()
            .unwrap_or_else(|| panic!("the {kind} row is listed"))
    };
    for (kind, bytes) in [
        ("bin", b"APP-BYTES".as_slice()),
        ("hex", b":00000001FF".as_slice()),
    ] {
        let attached = row(kind);
        assert!(attached.required && attached.present, "{kind} was attached");
        assert_eq!(attached.satisfied_by, "attachment", "{kind}");
        assert_eq!(
            attached.sha256.as_deref(),
            Some(fingerprint::sha256_hex(bytes).as_str()),
            "{kind} carries the digest of the attached bytes rather than a placeholder"
        );
        assert_eq!(attached.byte_size, Some(bytes.len() as u64), "{kind}");
    }
    assert_eq!(row("elf").satisfied_by, "analyzed");
}

#[test]
fn a_cancelled_dialog_changes_nothing_and_is_not_reported_as_a_failure() {
    let (_db, session) = bare_session("attach-cancel");
    let before = session
        .release_attachments(None, "op-cancel")
        .expect("an empty set reads back");
    assert!(before.attachments.is_empty());
    assert_eq!(
        before.matches_gate_run, None,
        "with no run on screen there is nothing for the set to agree or disagree with, and `true` would \
         claim an agreement that was never made"
    );

    let cancelled = session
        .attach_release_file(None, "op-cancel")
        .expect("cancel is a decision, not an error");
    assert!(cancelled.is_none());
    assert!(
        session
            .release_attachments(None, "op-cancel")
            .expect("the set is still empty")
            .attachments
            .is_empty()
    );
}

#[test]
fn an_attachment_row_carries_the_leaf_it_ships_and_never_the_folder_it_came_from() {
    let (_db, session) = bare_session("attach-path");
    let chosen = ChosenFiles::new("attach-path");
    let bin = chosen.write("motor-controller.bin", b"BYTES");

    let row = session
        .attach_release_file(Some(bin), "op-attach")
        .expect("observed")
        .expect("a file was chosen");
    assert_eq!(row.file_name, "motor-controller.bin");
    let text = serde_json::to_string(&row).expect("a row serializes");
    if let Some(marker) = host_path_in(&text) {
        panic!("the attachment row carries {marker}: {text}");
    }
    assert!(!text.contains(TEMP_MARKER), "{text}");
}

#[test]
fn adding_a_file_after_a_run_makes_that_run_not_current_until_the_gate_is_re_run() {
    let world = World::new("attach-added", &release_policy());
    let chosen = ChosenFiles::new("attach-added");
    let bin = chosen.write("app.bin", b"FIRST");
    let hex = chosen.write("boot.hex", b"SECOND");

    attach(&world.session, &bin);
    let first = world.gate_run();
    assert_eq!(
        world
            .session
            .release_attachments(Some(&first.run_id), "op-read")
            .expect("the set reads back")
            .matches_gate_run,
        Some(true)
    );

    attach(&world.session, &hex);
    assert_eq!(
        world
            .session
            .release_attachments(Some(&first.run_id), "op-read")
            .expect("the set reads back")
            .matches_gate_run,
        Some(false),
        "the run on screen judged one file; the page now holds two, and saying otherwise would be a stale \
         verdict"
    );

    let second = world.gate_run();
    assert_ne!(
        first.run_id, second.run_id,
        "the set is part of the identity"
    );
    assert_eq!(
        world
            .session
            .release_attachments(Some(&second.run_id), "op-read")
            .expect("the set reads back")
            .matches_gate_run,
        Some(true)
    );
}

#[test]
fn withdrawing_an_attachment_leaves_the_run_it_judged_untouched_and_the_page_disagreeing() {
    let world = World::new("attach-withdraw", &release_policy());
    let chosen = ChosenFiles::new("attach-withdraw");
    let bin = chosen.write("app.bin", b"FIRST");
    let hex = chosen.write("boot.hex", b"SECOND");
    let second_id = attach(&world.session, &hex);
    attach(&world.session, &bin);

    let run = world.gate_run();
    let remaining = world
        .session
        .remove_attachment(&second_id, "op-remove")
        .expect("the withdrawal is reported as the set that remains");
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].file_name, "app.bin");

    // The stored run is a record, not a view: it keeps the two rows it judged, and the page's set now
    // differs from it.
    let set = world
        .session
        .release_attachments(Some(&run.run_id), "op-read")
        .expect("the set reads back");
    assert_eq!(set.attachments.len(), 1);
    assert_eq!(set.matches_gate_run, Some(false));
    let stored: GateRunDto = world
        .session
        .gate_run(&run.run_id, "op-read")
        .expect("the run is still readable")
        .expect("the run was stored");
    assert_eq!(stored.run_id, run.run_id, "withdrawing rewrote nothing");
}

#[test]
fn a_renamed_attachment_that_cannot_be_found_refuses_the_export_and_writes_nothing() {
    // §28's re-read covers attached bytes: the plan froze a path, and a file that is gone when the person
    // presses Ship stops the write rather than shipping a bundle that quietly lost a file.
    let world = World::new("attach-moved", &release_policy());
    let chosen = ChosenFiles::new("attach-moved");
    let bin = chosen.write("app.bin", b"APP-BYTES");
    attach(&world.session, &bin);

    let preview = world.prepare().expect("the release prepares");
    std::fs::remove_file(&bin).expect("the chosen file is moved away after the preview");
    let chosen_folder = world.choose(&preview.plan_id).expect("a folder was chosen");
    let refused = world
        .export(&preview.plan_id, &chosen_folder.destination_token, false)
        .expect_err("an attached file that no longer exists cannot be shipped silently");

    assert_eq!(refused.code, "ERR-BUNDLE-6115", "{refused:?}");
    assert!(
        refused.message.contains("app.bin"),
        "the refusal names the leaf the owner chose: {refused:?}"
    );
    let text = serde_json::to_string(&refused).expect("an envelope serializes");
    if let Some(marker) = host_path_in(&text) {
        panic!("the refusal carries {marker}: {text}");
    }
    assert!(
        !std::fs::exists(&world.release.destination).unwrap_or(false)
            || entries(&world.release.destination).is_empty(),
        "a refused export wrote a bundle"
    );
}

#[test]
fn declaring_a_different_kind_moves_the_run_identity_without_touching_the_bytes() {
    // `.img` is not one of the two names the picker rule knows, so it enters as `unknown` — and the person
    // can say what they meant. The digest is the same file either way: a declaration changes what the run
    // binds, not what was measured.
    let world = World::new("attach-declared", &release_policy());
    let chosen = ChosenFiles::new("attach-declared");
    let img = chosen.write("payload.img", b"BYTES");
    let id = attach(&world.session, &img);
    let before = world
        .session
        .release_attachments(None, "op-read")
        .expect("the set reads back")
        .attachments;
    assert_eq!(before[0].kind, "unknown");
    let digest = before[0].sha256.clone();

    let first = world.gate_run();
    let declared = world
        .session
        .set_attachment_kind(&id, "bin", "op-kind")
        .expect("the kind is re-declared");
    assert_eq!(declared.kind, "bin");
    assert_eq!(declared.sha256, digest, "the bytes were not touched");
    assert_eq!(declared.byte_size, b"BYTES".len() as u64);

    let second = world.gate_run();
    assert_ne!(
        first.run_id, second.run_id,
        "a run that bound a declared BIN is not the run that bound an unidentified file"
    );
}

#[test]
fn a_kind_word_outside_the_three_a_release_can_attach_is_refused_not_stored() {
    let world = World::new("attach-word", &release_policy());
    let chosen = ChosenFiles::new("attach-word");
    let bin = chosen.write("app.bin", b"BYTES");
    let id = attach(&world.session, &bin);

    for word in ["elf", "map", "IMAGE", ""] {
        let refused = world
            .session
            .set_attachment_kind(&id, word, "op-kind")
            .expect_err("`{word}` is not a kind a release may declare");
        assert_eq!(refused.code, "ERR-INPUT-0001", "{refused:?}");
    }
    // The refusal changed nothing: the row still says what the name declared.
    assert_eq!(
        world
            .session
            .release_attachments(None, "op-read")
            .expect("the set reads back")
            .attachments[0]
            .kind,
        "bin"
    );
}

#[test]
fn an_attachment_handle_this_session_never_issued_is_refused_rather_than_invented() {
    let world = World::new("attach-handle", &release_policy());
    let refused = world
        .session
        .remove_attachment("att-dead-beef", "op-remove")
        .expect_err("a handle this process never issued is not an attachment");
    assert_eq!(refused.code, "ERR-INPUT-0001", "{refused:?}");
}

/// One JSON document read out of a written bundle.
fn document(bundle: &Path, name: &str) -> Value {
    let bytes = std::fs::read(bundle.join(name))
        .unwrap_or_else(|error| panic!("`{name}` is not in the bundle: {error}"));
    serde_json::from_slice(&bytes).unwrap_or_else(|error| panic!("`{name}` is not JSON: {error}"))
}

/// `NEXT-06` §E and T05 asked for this comparison in bytes rather than in UI assertions: the same release,
/// exported again after a language has been stored, must arrive at the same bundle.
///
/// `SHA256SUMS` is the file that makes one assertion cover the whole contract. Core writes it over every
/// other entry in `PREVIEW_ORDER`, so it names the digest of the analysis JSON, the diff JSON, the
/// gate-results JSON, the Release Notes and the self-contained `release-report.html`; comparing its bytes
/// therefore compares theirs, and a person's interface language reaching any exported document would show
/// up here as a different sums file rather than as a claim in a report.
#[test]
fn a_stored_language_moves_no_bundle_byte_no_manifest_digest_and_no_release_id() {
    let mut world = World::new("lang-identity", &release_policy());
    let prefs_dir = temp_name("lang-identity", "-settings");
    std::fs::create_dir_all(&prefs_dir).expect("a temporary settings folder");
    world
        .session
        .use_preferences(Store::new(Some(prefs_dir.clone())));

    let first = world.publish(false);
    let bundle = world.bundle_dir(&first);
    let sums = std::fs::read(bundle.join("SHA256SUMS")).expect("`SHA256SUMS` is in the bundle");
    let manifest =
        std::fs::read(bundle.join("release-manifest.json")).expect("the manifest is in the bundle");
    let report =
        std::fs::read(bundle.join("release-report.html")).expect("the report is in the bundle");
    let gate_results = std::fs::read(bundle.join("gate-results.json"))
        .expect("the gate results are in the bundle");

    // All four words move together, because a preference that changed a bundle would most plausibly do it
    // through the one this unit is about — and the other three are already covered by the Gate test.
    world
        .session
        .preferences()
        .save(&ApplicationPreferences {
            size_unit: SizeUnitPref::Kib,
            startup_page: StartupPagePref::History,
            theme: ThemePref::Dark,
            language: LanguagePref::ZhCn,
        })
        .expect("the four preferences are stored");
    let stored = std::fs::read_to_string(prefs_dir.join(SETTINGS_FILE_NAME))
        .expect("the document the session wrote");
    assert!(
        stored.contains("\"zh-CN\""),
        "the language never reached the document, so the comparison below proves nothing: {stored}"
    );

    // The same release, re-exported over the bundle this engine wrote a moment ago.
    let second = world.publish(true);
    assert_eq!(
        second.release_id, first.release_id,
        "a stored language moved the release id"
    );
    assert_eq!(
        second.manifest_sha256, first.manifest_sha256,
        "a stored language moved the manifest digest"
    );

    let reborn = world.bundle_dir(&second);
    assert_eq!(
        std::fs::read(reborn.join("SHA256SUMS")).expect("`SHA256SUMS` after the second export"),
        sums,
        "a stored language moved a digest inside the bundle"
    );
    assert_eq!(
        std::fs::read(reborn.join("release-manifest.json"))
            .expect("the manifest after the second export"),
        manifest,
        "a stored language moved the release manifest"
    );
    assert_eq!(
        std::fs::read(reborn.join("release-report.html"))
            .expect("the report after the second export"),
        report,
        "a stored language moved the self-contained report another person receives"
    );
    assert_eq!(
        std::fs::read(reborn.join("gate-results.json"))
            .expect("the gate results after the second export"),
        gate_results,
        "a stored language moved the stored gate results"
    );

    let _ = std::fs::remove_dir_all(&prefs_dir);
}
