//! P5 Commit D: what a Diagnostics payload must state, and what it must never carry.
//!
//! These tests are the reason the payload's shape is a claim rather than a hope. Prompt §18 sets the
//! standard: plant each prohibited value in the state the session actually holds, **prove it is there**,
//! and only then assert it is absent from what the product emits. A privacy test that exports an empty
//! object passes for no reason, so every absence here is paired with a presence — through the product's
//! own read path where one exists (`load_project_from`, `query_symbols`), and through a direct read of
//! the store where the product deliberately has none, because `artifacts.path` crossing the boundary is
//! the thing under test and not a convenience.
//!
//! §19 adds the structural half: no field may be a path, a raw row or a free-form map. That is asserted
//! by naming the allowlist of keys out loud, so the day someone adds `dbPath` the suite fails instead of
//! shipping a leak.
//!
//! Nothing here touches an application-data directory (§9's other rule): every database is a file in a
//! throwaway temp directory, and the damaged-store test corrupts only that file.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use firmwaresight_desktop::compare::{ExportFormat, write_export};
use firmwaresight_desktop::ipc::{DiagnosticsDto, GateRunRequestDto, SymbolRequestDto};
use firmwaresight_desktop::service::FixtureCatalog;
use firmwaresight_desktop::support::ProductFacts;
use firmwaresight_desktop::{FixtureKey, Session};
use firmwaresight_storage::{Database, SCHEMA_VERSION};
use serde_json::Value;

// ---- the controls prompt §18 names ---------------------------------------------------------------

/// An artifact directory whose absolute path is stored in `artifacts.path` by the product itself.
const SECRET_ARTIFACT_DIR: &str = "SecretProject";
/// The Release Notes body. The Gate hashes it; it must never be quoted.
const SECRET_NOTES_TEXT: &str = "SECRET_RELEASE_TEXT_SHOULD_NOT_LEAK";
/// A symbol name written into the store, then read back through `query_symbols`.
const SECRET_SYMBOL: &str = "top_secret_symbol_name";
/// A project name held by the loaded `LoadedProject`, and shown by `ProjectContextDto`.
const SECRET_PROJECT_NAME: &str = "SECRET_PROJECT_NAME_SHOULD_NOT_TRAVEL";
/// A remote URL held by the repository the loaded project sits in.
const SECRET_REMOTE: &str = "git@example.com:secret/repo.git";

/// The complete set of object keys a Diagnostics payload may contain, at any depth.
///
/// This is prompt §16's allowlist written down as a test. An extra field — a database path, a project
/// root, an environment map — fails here before it can fail in a support file.
const ALLOWED_KEYS: [&str; 41] = [
    "acceptedReviews",
    "appVersion",
    "architecture",
    "available",
    "backupFiles",
    "binaryName",
    "builds",
    "configSchemaVersion",
    "counts",
    "generatedAt",
    "git",
    "gateRuns",
    "health",
    "healthErrorCode",
    "healthSummary",
    "identifier",
    "inputCohort",
    "installChannel",
    "journalMode",
    "osFamily",
    "osVersion",
    "platform",
    "policy",
    "product",
    "productName",
    "projects",
    "recentErrorCodes",
    "releaseRecords",
    "requireCleanGit",
    "requireCleanGitNote",
    "requireReleaseNotes",
    "runtime",
    "schema",
    "schemaVersion",
    "store",
    "storeFileName",
    "supportedSchemaVersion",
    "support",
    "tauriVersion",
    "version",
    "webviewVersion",
];

// ---- harness -------------------------------------------------------------------------------------

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .expect("the desktop shell lives at <root>/apps/desktop/src-tauri")
        .to_path_buf()
}

/// A throwaway directory holding everything one test touches: the store, the artifact, the project.
///
/// A directory rather than a bare file because the store writes a pre-migration snapshot and WAL
/// partners beside itself, and because the paths inside it are the secrets the tests measure. The store
/// folder is deliberately called `appdata`, so a leaked store location is detectable as a word.
struct Sandbox(PathBuf);

impl Sandbox {
    fn new(label: &str) -> Self {
        let unique = format!(
            "fwsight-diagnostics-{label}-{}-{:?}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|elapsed| elapsed.as_nanos())
                .unwrap_or(0)
        );
        let dir = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&dir).expect("a temporary sandbox");
        Self(dir)
    }

    fn dir(&self) -> &Path {
        &self.0
    }

    /// The store, at the logical name §20 requires, inside a directory of its own.
    fn store_path(&self) -> PathBuf {
        self.directory(&["appdata"]).join("firmwaresight-p0.sqlite")
    }

    /// A directory, created.
    fn directory(&self, segments: &[&str]) -> PathBuf {
        let mut path = self.0.join(segments[0]);
        for segment in &segments[1..] {
            path.push(segment);
        }
        std::fs::create_dir_all(&path).expect("a sandbox directory");
        path
    }

    /// A path for a file, with every parent created.
    fn file(&self, segments: &[&str]) -> PathBuf {
        self.directory(&segments[..segments.len() - 1])
            .join(segments[segments.len() - 1])
    }

    /// A real ELF at a path nobody would name in a support file.
    fn secret_artifact(&self) -> PathBuf {
        let target = self.file(&[SECRET_ARTIFACT_DIR, "firmware.elf"]);
        std::fs::copy(
            repo_root().join("fixtures/elf/p0-basic/firmware.elf"),
            &target,
        )
        .expect("the committed fixture ELF is copied to the secret-looking path");
        target
    }

    fn session(&self) -> Session {
        Session::open(FixtureCatalog::from_environment(), self.store_path())
            .expect("a session opens on a throwaway store")
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// What the running binary would report, written out here because these four facts come from a bundle
/// and no test has one. The payload's contents are what these tests are about; what the real bundle
/// answers is prompt §27's installed-app check.
fn facts() -> ProductFacts {
    ProductFacts {
        product_name: "FirmwareSight".to_owned(),
        binary_name: "firmwaresight-desktop".to_owned(),
        app_version: "0.6.0".to_owned(),
        identifier: "com.example.firmwaresight".to_owned(),
    }
}

/// The WebView version a test injects. `tauri::webview_version()` is reached only from a command: a
/// headless test binary must not depend on a WebView runtime being present.
const INJECTED_WEBVIEW: &str = "not_reported";

fn payload(session: &Session) -> DiagnosticsDto {
    session
        .diagnostics(&facts(), INJECTED_WEBVIEW, "op-diagnostics")
        .expect("a session answers for itself")
}

fn text(payload: &DiagnosticsDto) -> String {
    serde_json::to_string_pretty(payload).expect("the payload is JSON by construction")
}

/// A `firmwaresight.toml` that requires Release Notes, permits a dirty worktree, and names the project
/// with the control string.
fn config_text() -> String {
    format!(
        "schema_version = 1\n\n[project]\nname = \"{SECRET_PROJECT_NAME}\"\n\n[artifacts]\nrequired = \
         [\"elf\"]\n\n[release]\nrequire_clean_git = false\nrequire_release_notes = true\n\
         release_notes_path = \"docs/RELEASE_NOTES.md\"\n"
    )
}

fn git(args: &[&str], dir: &Path) {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("a Git remote is planted against a real repository, so git must be installed");
    assert!(
        output.status.success(),
        "`git {}` failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Read one string out of the store with no product API in the way.
///
/// This is the only place in this file that speaks SQL, and it speaks it *to the file*, from outside the
/// application: the product's refusal to expose `artifacts.path` is what is being tested, so the test
/// cannot ask the product to confirm the value it is about to prove absent.
fn stored_string(db: &Database, sql: &str) -> String {
    db.connection()
        .query_row(sql, [], |row| row.get::<_, String>(0))
        .expect("the store reads back")
}

/// Stage and analyze the secret-pathed artifact, returning the snapshot id the UI would hold.
fn analyze_secret(session: &Session, artifact: &Path) -> String {
    let selection = session
        .stage_artifact(artifact, "op-stage")
        .expect("the artifact stages");
    session
        .analyze_selection(&selection.selection_id, "op-analyze")
        .expect("the artifact analyzes and stores")
        .identity
        .snapshot_id
}

/// Every object key at every depth, as a set.
fn keys(value: &Value, found: &mut BTreeSet<String>) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                found.insert(key.clone());
                keys(child, found);
            }
        }
        Value::Array(items) => {
            for item in items {
                keys(item, found);
            }
        }
        _ => {}
    }
}

/// Every string in the payload, paired with the key that holds it.
fn leaf_strings(value: &Value) -> Vec<(String, String)> {
    fn walk(value: &Value, key: &str, out: &mut Vec<(String, String)>) {
        match value {
            Value::Object(map) => {
                for (child_key, child) in map {
                    if let Value::String(text) = child {
                        out.push((child_key.clone(), text.clone()));
                    }
                    walk(child, child_key, out);
                }
            }
            Value::Array(items) => {
                for item in items {
                    walk(item, key, out);
                }
            }
            Value::String(text) => out.push((key.to_owned(), text.clone())),
            _ => {}
        }
    }
    let mut out = Vec::new();
    walk(value, "<root>", &mut out);
    out
}

/// Take the two fields that are readings rather than facts out of a payload, and hand them back.
///
/// `generatedAt` is the engine's clock at the instant the payload was assembled, and `recentErrorCodes` is
/// a process-wide ring that tests in this same binary write concurrently. A payload collected twice is
/// therefore not required to match in either, and pretending otherwise makes a correct product fail a test
/// for the order its threads ran in — which the local gate did, once. Everything else in the payload
/// describes state that does not move between the two calls, and is compared whole.
fn readings(value: &mut Value) -> (Option<String>, Vec<String>) {
    let object = value
        .as_object_mut()
        .expect("the diagnostics payload is an object");
    let stamp = object
        .remove("generatedAt")
        .and_then(|value| value.as_str().map(str::to_owned));
    let codes = object
        .remove("recentErrorCodes")
        .and_then(|codes| codes.as_array().cloned())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    (stamp, codes)
}

// ---- 1. §16: the payload states facts a support conversation starts with ------------------------

#[test]
fn the_payload_states_every_safe_fact_a_support_conversation_starts_with() {
    let sandbox = Sandbox::new("facts");
    let session = sandbox.session();
    let artifact = sandbox.secret_artifact();
    analyze_secret(&session, &artifact);

    let payload = payload(&session);
    let raw = text(&payload);

    assert_eq!(payload.schema, "firmwaresight-diagnostics-1");
    assert_eq!(payload.product.app_version, "0.6.0");
    assert_eq!(payload.product.product_name, "FirmwareSight");
    // §8's logical store identity: the name this session opened, and only the name.
    assert_eq!(payload.product.store_file_name, "firmwaresight-p0.sqlite");
    assert_eq!(payload.runtime.os_family, std::env::consts::OS);
    assert_eq!(payload.runtime.architecture, std::env::consts::ARCH);
    assert_eq!(
        payload.runtime.platform,
        format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
    );
    assert_eq!(payload.runtime.tauri_version, tauri::VERSION);
    // An unobservable fact says so instead of being invented (§21).
    assert_eq!(payload.runtime.os_version, "not_reported");
    assert_eq!(payload.runtime.webview_version, INJECTED_WEBVIEW);

    // Storage: the file's own version, its health, how it journals, one number per class.
    assert_eq!(payload.store.supported_schema_version, SCHEMA_VERSION);
    assert_eq!(payload.store.schema_version, Some(SCHEMA_VERSION));
    assert_eq!(payload.store.health, "healthy", "{raw}");
    assert!(
        ["wal", "delete", "memory", "truncate"].contains(&payload.store.journal_mode.as_str()),
        "an unexpected journal mode means the open path changed: {}",
        payload.store.journal_mode
    );
    assert_eq!(payload.store.counts.projects, Some(1));
    assert_eq!(payload.store.counts.builds, Some(1));
    assert_eq!(
        payload.store.counts.gate_runs,
        Some(0),
        "no Gate has run here"
    );
    assert_eq!(payload.store.counts.accepted_reviews, Some(0));
    assert_eq!(payload.store.counts.release_records, Some(0));
    assert!(
        payload.store.backup_files.is_empty(),
        "nothing has been upgraded"
    );

    // Git is a yes/no plus a version that was shape-checked, and nothing about a repository.
    assert!(
        payload.git.version == "not_reported" || payload.git.version.starts_with("git version "),
        "a Git version is a version: {}",
        payload.git.version
    );

    // Support: the cohort is stated, and the channel is the marker the binary carries or `unknown`.
    assert!(
        payload.support.input_cohort.contains("ELF"),
        "the cohort has to say what this build reads: {}",
        payload.support.input_cohort
    );
    assert!(
        [
            "msi", "nsis", "deb", "rpm", "appimage", "app", "dmg", "unknown"
        ]
        .contains(&payload.support.install_channel.as_str()),
        "the install channel is a bundle name or unknown: {}",
        payload.support.install_channel
    );

    // No project has been opened, so the policy section states that rather than inventing one.
    assert!(payload.policy.is_none());

    // §18: the clock is observation metadata, in the shape every stored timestamp already uses.
    let stamp = payload
        .generated_at
        .clone()
        .expect("the engine reads a clock");
    assert_eq!(stamp.len(), 20, "YYYY-MM-DDTHH:MM:SSZ, got {stamp}");
    assert!(stamp.ends_with('Z'));
    assert!(
        !raw.contains("\"product\": {}") && raw.contains(payload.product.app_version.as_str()),
        "an empty section proves nothing: {raw}"
    );
}

// ---- 2. §18: every secret the session holds stays inside the session ----------------------------

#[test]
fn every_secret_the_session_holds_stays_inside_the_session() {
    let sandbox = Sandbox::new("privacy");
    let store = sandbox.store_path();
    let session = sandbox.session();

    // A build stored from a path nobody would name in a support file.
    let artifact = sandbox.secret_artifact();
    let snapshot_id = analyze_secret(&session, &artifact);

    // A loaded project: its name, its root, its config file, and a repository with a remote.
    let project = sandbox.directory(&["project"]);
    let notes = sandbox.file(&["project", "docs", "RELEASE_NOTES.md"]);
    std::fs::write(project.join("firmwaresight.toml"), config_text()).expect("a config");
    std::fs::write(&notes, format!("# Notes\n\n{SECRET_NOTES_TEXT}\n")).expect("a notes file");
    git(&["init", "-q", "-b", "main"], &project);
    git(&["remote", "add", "origin", SECRET_REMOTE], &project);
    let context = session
        .load_project_from(Some(project.join("firmwaresight.toml")), "op-open")
        .expect("the config opens")
        .expect("a chosen path is not the cancel path");

    // A Gate run, so the counts are not all zero and a notes digest exists somewhere.
    session
        .run_release_gate(
            &GateRunRequestDto {
                snapshot_id: snapshot_id.clone(),
                baseline_snapshot_id: None,
            },
            "op-gate",
        )
        .expect("the Gate runs");

    // A symbol name planted in the store.
    {
        let probe = Database::open(&store).expect("the store opens for the fixture write");
        probe
            .connection()
            .execute(
                &format!("UPDATE symbols SET name = '{SECRET_SYMBOL}' WHERE ordinal = 1"),
                [],
            )
            .expect("the secret symbol name is written");
        drop(probe);
    }

    // ---- prove each control is really in the state this session holds ----
    let probe = Database::open(&store).expect("the store opens for the independent read");
    let stored_path = stored_string(&probe, "SELECT path FROM artifacts LIMIT 1");
    assert!(
        stored_path.contains(SECRET_ARTIFACT_DIR) && Path::new(&stored_path).is_absolute(),
        "the store must hold the absolute artifact path or its absence proves nothing: {stored_path}"
    );
    assert!(
        store.exists(),
        "the store exists at a path nobody may see in the payload"
    );
    assert_eq!(
        context.project_name, SECRET_PROJECT_NAME,
        "the loaded project's name is the control string"
    );
    let git_config = std::fs::read_to_string(project.join(".git").join("config"))
        .expect("the repository config");
    assert!(
        git_config.contains(SECRET_REMOTE),
        "the session's repository holds the remote: {git_config}"
    );
    assert!(
        std::fs::read_to_string(&notes)
            .expect("the notes file reads back")
            .contains(SECRET_NOTES_TEXT)
    );
    let page = session
        .query_symbols(
            &SymbolRequestDto {
                snapshot_id,
                limit: Some(500),
                ..Default::default()
            },
            "op-symbols",
        )
        .expect("the symbols page reads");
    assert!(
        page.rows
            .iter()
            .any(|row| row.name.as_deref() == Some(SECRET_SYMBOL)),
        "the product itself will name the planted symbol, so the store really holds it"
    );

    // ---- and none of it leaves ----
    let payload = payload(&session);
    let raw = text(&payload);
    for secret in [
        SECRET_ARTIFACT_DIR,
        SECRET_NOTES_TEXT,
        SECRET_SYMBOL,
        SECRET_PROJECT_NAME,
        SECRET_REMOTE,
        "example.com",
        "secret/repo",
        "RELEASE_NOTES.md",
        "firmwaresight.toml",
    ] {
        assert!(
            !raw.contains(secret),
            "a Diagnostics payload must not carry {secret}, and this one did:\n{raw}"
        );
    }
    for location in [
        sandbox.dir(),
        store.parent().expect("the store has a directory"),
        artifact.parent().expect("the artifact has a directory"),
        &project,
    ] {
        let shown = location.display().to_string();
        assert!(
            !raw.contains(&shown),
            "no directory the session knows may travel: {shown}"
        );
    }

    // The payload is not emptied to achieve that: the safe facts are still there, and the policy caveat
    // §1 requires travels with the flag.
    assert_eq!(payload.store.health, "healthy");
    assert_eq!(payload.store.counts.projects, Some(1));
    assert_eq!(payload.store.counts.builds, Some(1));
    assert_eq!(
        payload.store.counts.gate_runs,
        Some(1),
        "the run this session made"
    );
    let policy = payload.policy.expect("a project is loaded");
    assert_eq!(policy.config_schema_version, 1);
    assert!(!policy.require_clean_git);
    assert!(
        policy.require_clean_git_note.contains("line endings"),
        "the false flag has to arrive with its meaning: {}",
        policy.require_clean_git_note
    );
    assert!(policy.require_release_notes);
    assert_eq!(payload.product.store_file_name, "firmwaresight-p0.sqlite");
}

// ---- 3. §19: no field of the payload can hold a path -------------------------------------------

#[test]
fn no_field_of_the_payload_can_hold_a_path() {
    let sandbox = Sandbox::new("shape");
    let session = sandbox.session();
    // A loaded project, so the policy section is populated and the key set compared is the whole one.
    let project = sandbox.directory(&["project"]);
    std::fs::write(project.join("firmwaresight.toml"), config_text()).expect("a config");
    session
        .load_project_from(Some(project.join("firmwaresight.toml")), "op-open")
        .expect("the config opens")
        .expect("a chosen path is not the cancel path");

    let payload = payload(&session);
    let value = serde_json::to_value(&payload).expect("the payload is JSON by construction");
    let raw = serde_json::to_string(&value).expect("the payload is JSON by construction");

    let mut found = BTreeSet::new();
    keys(&value, &mut found);
    let expected: BTreeSet<String> = ALLOWED_KEYS.iter().map(|key| (*key).to_owned()).collect();
    assert_eq!(
        found, expected,
        "the allowlist moved without this test moving with it"
    );

    // No string in the payload is a path — with the two exceptions that can legitimately carry a slash:
    // a WebView runtime name (`Edg/120.0.2210.91`) and a Git version string. Those are the only fields
    // whose text comes from outside this repository, so they are named here rather than silently exempt.
    for (key, item) in leaf_strings(&value) {
        if key == "webviewVersion" || key == "version" {
            continue;
        }
        assert!(
            !item.contains('/') && !item.contains('\\'),
            "{key} carries a separator, which is how a path arrives: {item}"
        );
    }
    assert!(
        !raw.contains("appdata"),
        "the sandbox's own store directory leaked: {raw}"
    );
}

// ---- 4. §20: the exported file parses on its own and is the same payload -----------------------

#[test]
fn the_exported_file_is_the_payload_and_parses_on_its_own() {
    let sandbox = Sandbox::new("export");
    let session = sandbox.session();
    let payload = payload(&session);

    let written = session
        .diagnostics_json(&facts(), INJECTED_WEBVIEW, "op-json")
        .expect("the payload renders");
    assert!(written.ends_with('\n'), "a file should end with a newline");

    let target = sandbox.file(&["out", "firmwaresight-diagnostics.json"]);
    let outcome = write_export(Some(target.clone()), &written, ExportFormat::Json, |_| true)
        .expect("the export writes");
    assert_eq!(outcome.status, "written");
    assert_eq!(
        outcome.file_name.as_deref(),
        Some("firmwaresight-diagnostics.json"),
        "the export names the file and never the folder it landed in"
    );
    assert_eq!(outcome.format, "json");

    let from_disk = std::fs::read_to_string(&target).expect("the file reads back");
    assert_eq!(from_disk, written);
    let mut parsed: Value =
        serde_json::from_str(&from_disk).expect("diagnostics.json parses alone");
    let mut collected = serde_json::to_value(&payload).expect("the payload is JSON");
    // Everything the file says about this application is the payload's own, so it is compared field by
    // field. Two fields are excluded and asserted separately, on purpose: `generatedAt` is a clock reading
    // and `recentErrorCodes` is a process-wide ring that other tests in this same binary write while this
    // one runs. Demanding they match across two captures of one payload would be demanding that a reading
    // be a fact — and it did fail, in the local gate's own run order, for exactly that reason.
    let written_reading = readings(&mut parsed);
    let collected_reading = readings(&mut collected);
    assert_eq!(
        parsed, collected,
        "what was written is what was collected, apart from the two readings"
    );
    for (stamp, codes) in [written_reading.clone(), collected_reading.clone()] {
        let stamp = stamp.expect("a store that answers the health check answers the clock too");
        assert_eq!(
            stamp.len(),
            20,
            "UTC to the second, the shape every stored timestamp uses: {stamp}"
        );
        assert!(stamp.ends_with('Z'), "{stamp}");
        assert!(
            codes.len() <= 8,
            "the ring is bounded at 8 and held {}",
            codes.len()
        );
        for code in codes {
            assert!(
                code.starts_with("ERR-") && !code.contains('/') && !code.contains('\\'),
                "the ring holds registry codes: {code}"
            );
        }
    }
    // The two readings differ only by what happened between the two calls, which is the honest statement of
    // what this test can know: the codes each side holds are all codes, and neither holds prose.
    assert!(
        written_reading
            .1
            .iter()
            .chain(collected_reading.1.iter())
            .all(|code| code.starts_with("ERR-")),
        "a reading carried something that is not a code"
    );

    // A second export the person refuses leaves the first file exactly as it was.
    let kept = write_export(
        Some(target.clone()),
        "{\"different\":true}\n",
        ExportFormat::Json,
        |_| false,
    )
    .expect("keeping an existing file is an outcome, not an error");
    assert_eq!(kept.status, "kept-existing");
    assert_eq!(
        std::fs::read_to_string(&target).expect("the file still reads"),
        from_disk
    );

    // One file in the folder, and no scratch left behind.
    let mut leftovers: Vec<String> =
        std::fs::read_dir(target.parent().expect("the export directory"))
            .expect("the directory reads")
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect();
    leftovers.sort();
    assert_eq!(leftovers, vec!["firmwaresight-diagnostics.json".to_owned()]);
}

// ---- 4b. §21: a code recorded in between moves the ring, and nothing else ----------------------

#[test]
fn a_failure_between_two_captures_changes_the_ring_and_nothing_else() {
    // The deterministic shape of what the local gate found: two captures of "the same" payload differed, in
    // `recentErrorCodes` alone, because a command failed in between. Reproduced here on one thread, on
    // purpose, so the two fields the export test excludes are a characterized boundary rather than a
    // tolerated flake.
    let sandbox = Sandbox::new("ring");
    let missing = sandbox.directory(&["no-catalog"]);
    let session = Session::open(
        FixtureCatalog::new(missing.join("no-such-fixtures")),
        sandbox.store_path(),
    )
    .expect("a session opens even when the catalog does not exist yet");

    let mut before = serde_json::to_value(payload(&session)).expect("the payload is JSON");
    let failure = session
        .analyze_and_store(FixtureKey::P0Basic, "op-missing")
        .expect_err("a fixture that is not there cannot analyze");
    let mut after = serde_json::to_value(payload(&session)).expect("the payload is JSON");

    let (before_stamp, before_codes) = readings(&mut before);
    let (after_stamp, after_codes) = readings(&mut after);
    // The other seven sections describe this session, and a refused analysis changed none of them.
    assert_eq!(
        before, after,
        "the failing command moved the ring, not the store, the runtime, the policy or the Git probe"
    );
    assert!(
        after_codes.contains(&failure.code),
        "the code the user was handed is in the second reading: {after_codes:?}"
    );
    assert!(
        before_codes.len() <= after_codes.len() || before_codes.len() == 8,
        "the ring fills, and at its bound of 8 it replaces: {before_codes:?} -> {after_codes:?}"
    );
    for code in &after_codes {
        assert!(
            code.starts_with("ERR-") && !code.contains('/') && !code.contains('\\'),
            "a reading is a code, never prose that could carry a path: {code}"
        );
    }
    // The clock is a reading too. Both sides state one; neither is required to state the same second.
    assert!(
        before_stamp.is_some() && after_stamp.is_some(),
        "a store that answers the health check answers the clock: {before_stamp:?} {after_stamp:?}"
    );
}

// ---- 5. §9 and §22: a damaged store is described, not repaired ---------------------------------

#[test]
fn a_damaged_store_is_described_and_left_where_it_was() {
    let sandbox = Sandbox::new("damaged");
    let store = sandbox.store_path();

    // Build a store with real rows and enough of them that the back half is data pages.
    {
        let session = sandbox.session();
        let artifact = sandbox.secret_artifact();
        analyze_secret(&session, &artifact);
        drop(session);
        let probe = Database::open(&store).expect("the store opens for the filler write");
        for start in (1_000..9_000usize).step_by(500) {
            let end = start + 499;
            let mut sql = String::from(
                "INSERT INTO symbols (build_id, ordinal, kind, binding, section_ref, name, address, \
                 size) VALUES ",
            );
            for ordinal in start..=end {
                sql.push_str(&format!(
                    "((SELECT id FROM builds LIMIT 1), {ordinal}, 'OBJECT', 'GLOBAL', 1, \
                     'filler_symbol_{ordinal}', {ordinal}, 8), "
                ));
            }
            sql.truncate(sql.len() - 2);
            sql.push(';');
            probe
                .connection()
                .execute_batch(&sql)
                .expect("filler symbols");
        }
        drop(probe);
    }

    let before = std::fs::read(&store).expect("the store reads");
    let blasted = damage_deep_leaf_pages(&store);
    assert!(
        blasted >= 5,
        "the fixture damaged {blasted} pages; a store this size should have more to damage"
    );

    let session = Session::open(FixtureCatalog::from_environment(), &store).expect(
        "a store with damaged data pages still opens, which is what makes this the health path",
    );
    let payload = payload(&session);
    let raw = text(&payload);

    assert_ne!(
        payload.store.health, "healthy",
        "a damaged store cannot be reported as well: {raw}"
    );
    if let Some(summary) = &payload.store.health_summary {
        assert_eq!(payload.store.health, "unhealthy");
        assert!(
            !summary.contains('/') && !summary.contains('\\'),
            "the problems are named without naming a directory: {summary}"
        );
        assert!(
            summary.chars().count() <= 600,
            "the summary is bounded: {} chars",
            summary.chars().count()
        );
    } else {
        assert_eq!(payload.store.health, "unknown");
        assert!(
            payload
                .store
                .health_error_code
                .as_deref()
                .unwrap_or_default()
                .starts_with("ERR-"),
            "a check that could not be run still says which code it was"
        );
    }

    // Nothing was repaired, replaced or truncated, and no empty database took its place.
    let after = std::fs::read(&store).expect("the store still exists");
    assert!(
        after.len() >= before.len(),
        "a health check that shrank the file would be a repair: {} bytes became {}",
        before.len(),
        after.len()
    );
    let probe = Database::open(&store).expect("the damaged store opens again for the read");
    let artifacts: i64 = probe
        .connection()
        .query_row("SELECT COUNT(*) FROM artifacts", [], |row| row.get(0))
        .unwrap_or(-1);
    assert_eq!(
        artifacts, 1,
        "the build this session stored is still the build the store holds"
    );
}

/// Blast the cell count of every b-tree leaf page past the halfway mark — the measured shape that leaves
/// the file openable. Page types 0x0d and 0x0a are table and index leaves; page 1, the schema pages, the
/// table and index root pages and the freelist all sit earlier in the file, which is why the damage stops
/// at the halfway point rather than at page 2.
fn damage_deep_leaf_pages(path: &Path) -> usize {
    let page = 4096usize;
    let mut bytes = std::fs::read(path).expect("the store reads");
    let pages = bytes.len() / page;
    let mut blasted = 0;
    for index in pages / 2..pages {
        let header = page * index;
        if matches!(bytes[header], 0x0d | 0x0a) {
            bytes[header + 3] = 0xFF;
            bytes[header + 4] = 0xFF;
            blasted += 1;
        }
    }
    std::fs::write(path, &bytes).expect("the damage is written");
    blasted
}

// ---- 6. §21: recent error codes are codes, and nothing else ------------------------------------

#[test]
fn a_failing_command_records_its_code_and_never_its_message() {
    let sandbox = Sandbox::new("codes");
    // A catalog that points at nothing: the same failure a packaged run makes when its resources are not
    // where the binary expects them, which is exactly what a support file should be able to name.
    let missing = sandbox.directory(&["empty-catalog"]);
    let session = Session::open(
        FixtureCatalog::new(missing.join("no-such-fixtures")),
        sandbox.store_path(),
    )
    .expect("the session opens even when the catalog does not exist yet");

    let failure = session
        .analyze_and_store(FixtureKey::P0Basic, "op-missing")
        .expect_err("a fixture that is not there cannot analyze");

    let payload = payload(&session);
    assert!(
        payload
            .recent_error_codes
            .iter()
            .any(|code| code == &failure.code),
        "the code the user was given is in the ring, and the ring held {:?}",
        payload.recent_error_codes
    );
    for code in &payload.recent_error_codes {
        assert!(
            code.starts_with("ERR-") && !code.contains('/') && !code.contains('\\'),
            "the ring holds registry codes, never text that could carry a path: {code}"
        );
    }
    assert!(
        payload.recent_error_codes.len() <= 8,
        "the ring is bounded at 8 and held {}",
        payload.recent_error_codes.len()
    );
    // The code travels without the message: nothing in the payload is the failure's prose.
    assert!(
        !text(&payload).contains("No artifact was found"),
        "a message is not a code, and only the code may travel"
    );
}

// ---- 7. §17: a snapshot's existence is explainable, and its location is not --------------------

#[test]
fn a_snapshot_this_store_forced_is_named_and_never_located() {
    let sandbox = Sandbox::new("snapshot");
    let store = sandbox.store_path();

    // A store a version-4 build would have left behind, made by removing exactly what the later stages
    // added, and holding one build so the upgrade is of a file with history in it rather than of an
    // empty shell.
    {
        let db = Database::open(&store).expect("the store is created");
        db.connection()
            .execute_batch(
                "DROP INDEX idx_builds_created;
                 ALTER TABLE sections DROP COLUMN file_offset_unknown;
                 ALTER TABLE symbols DROP COLUMN address_unknown;
                 DROP TABLE gate_run_attachments;
                 DELETE FROM schema_migrations WHERE version >= 5;
                 INSERT INTO projects (id, name) VALUES ('proj-s', 'Snapshot project');
                 INSERT INTO builds (id, project_id, snapshot_id, normalization_version,
                                     created_by_fwsight, state)
                     VALUES ('build-s', 'proj-s', 'snap-s', 'p0-normalize-1', '0.6.0', 'COMPLETE');",
            )
            .expect("the store is stepped down to schema 4 with a build in it");
    }

    // Opening it with this build upgrades it, and the upgrade is preceded by the copy §10 requires.
    let session = sandbox.session();
    let payload = payload(&session);
    let raw = text(&payload);

    assert_eq!(
        payload.store.backup_files,
        vec!["firmwaresight-p0.pre-migration-v4-to-v6.sqlite".to_owned()],
        "the snapshot this transition owed is reported by name: {:?}",
        payload.store.backup_files
    );
    assert!(
        store
            .parent()
            .expect("the store has a directory")
            .join("firmwaresight-p0.pre-migration-v4-to-v6.sqlite")
            .exists(),
        "the file the payload names is the file that exists"
    );
    assert!(
        !raw.contains("appdata") && !raw.contains(&sandbox.dir().display().to_string()),
        "a snapshot's name travels; the folder it lives in does not: {raw}"
    );
    assert_eq!(
        payload.store.schema_version,
        Some(SCHEMA_VERSION),
        "the live store reached this build's schema"
    );
}
