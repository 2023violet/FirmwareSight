//! Diagnostics: one bounded, allowlisted payload about *this application*, and one Rust-side save
//! dialog that puts it in a file.
//!
//! Prompt §5 puts this under the Help/About surface rather than on a fifth navigation verb, and §6
//! decides its shape: an explicit allowlist of field names, each filled by Rust from a source that is
//! not the WebView. That is why `DiagnosticsDto` is seven closed structs instead of a map — a map is how
//! "we only send what we meant to send" stops being true the day someone stores one more fact in the
//! session. §7's prohibitions (artifact path, MAP path, project root, database path, bundle
//! destination, home, username, Git remote, firmware bytes, MAP or notes contents, symbol names, an
//! environment dump, credentials, shell history, raw logs, stack traces, SQL, raw SQLite rows) are
//! then enforced by the shape: no field of this payload can hold one of those things, and
//! `tests/diagnostics.rs` proves each of them is *present in the session* before asserting it is
//! *absent from the payload* (prompt §20's "no vacuous privacy test").
//!
//! Three deliberate absences are as load-bearing as the fields:
//!
//! - **No OS version.** The standard library does not carry one, and §26's rule against buying a
//!   platform dependency for a single field applies just as hard here. The field exists and says
//!   `not_reported`, so a reader knows it was asked and refused rather than forgotten.
//! - **No repository provenance from Git.** §25: Diagnostics describes the support environment, not
//!   the project. The probe runs `--version` through the same bounded runner the Gate uses, with a
//!   fixed argument vector and no repository in sight.
//! - **No verbose log.** §24 forbids building a logging subsystem for this surface. What is here is a
//!   fixed-capacity list of *stable codes* — never a message, because a message can quote the path it
//!   failed on.
//!
//! The export is user-triggered only (§18), writes `diagnostics.json` through the native Save dialog
//! the Rust side already owns (`ADR-0025`: the WebView gains no filesystem surface from it), and lands
//! atomically the way the Compare export does: a temporary file beside the target, then a rename.

use std::ffi::OsString;
use std::path::Path;
use std::sync::{Arc, Mutex, PoisonError};

use firmwaresight_project::git::{GitProbe, RunOutcome, run_bounded};
use firmwaresight_report::render;
use firmwaresight_storage::{Database, SCHEMA_VERSION, StoreHealth};
use tauri::{AppHandle, State};

use crate::compare::{ExportFormat, confirm_replacement, save_path, write_export};
use crate::ipc::{
    DiagnosticsCountsDto, DiagnosticsDto, DiagnosticsGitDto, DiagnosticsPolicyDto,
    DiagnosticsProductDto, DiagnosticsRuntimeDto, DiagnosticsStoreDto, DiagnosticsSupportDto,
    ErrorEnvelopeDto, ExportOutcomeDto,
};
use crate::service::display_name;
use crate::support::{ProductFacts, product_facts};
use crate::{Session, next_operation_id, task_join_error};

/// The payload's own contract label, so a support file says what shape it is. This is a Desktop
/// contract, not a portable firmware schema (prompt §19), and it is versioned separately for that
/// reason: nothing outside this shell promises to read it.
const SCHEMA: &str = "firmwaresight-diagnostics-1";

/// What an unobservable fact says. Never a guess, never an empty string a reader has to interpret.
const NOT_REPORTED: &str = "not_reported";

/// The supported input cohort, stated by this build rather than inferred from what happened to parse.
/// The Support Matrix document is the authority on what is claimed to work; this string is the same
/// claim in payload form, and `Help.tsx` points a reader at that document rather than at this line.
const INPUT_COHORT: &str = "ELF firmware artifacts, with an optional GNU ld MAP file beside them";

/// How many codes the ring keeps. Small on purpose: this is a support hint, not a journal (§24).
const RECENT_CODE_CAPACITY: usize = 8;

/// How many snapshot names a payload may carry. The set of transitions this product can perform is
/// itself finite, so the cap is a floor under the claim rather than the mechanism that bounds it.
const MAX_BACKUP_NAMES: usize = 8;

/// The longest version line worth carrying. `git --version` answers in about thirty characters; a
/// longer answer is not a version and will not be pasted into a support file.
const MAX_VERSION_CHARS: usize = 60;

/// The `require_clean_git = false` caveat prompt §1 asks to be made legible wherever the flag is
/// reported. It states what the flag does to identity, and it claims nothing about the state of a
/// workspace: only Git gets to say that, and this payload never asks Git about a repository.
const REQUIRE_CLEAN_GIT_NOTE: &str = "With require_clean_git = false, a working-tree difference in \
line endings can change the Release Notes digest, the Gate run id and the release id without \
git.clean blocking the release. FirmwareSight reports the policy as the project set it.";

/// The file name the Save dialog opens with. Deterministic, because §18 asks an export to be
/// reproducible and a timestamp in a name would make one capture look like two.
const EXPORT_FILE_NAME: &str = "firmwaresight-diagnostics.json";

/// The codes this process produced, newest last.
///
/// A process-wide fixed-capacity list rather than a per-session one, because the paths that fail are
/// spread across the shell and none of them carries a session handle. A poisoned lock falls back to
/// the values it holds: losing the ring because an unrelated thread panicked would be a worse support
/// outcome than a slightly stale list.
static RECENT_CODES: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// Remember one stable code for the next Diagnostics payload.
///
/// Fed by the five converters that turn a typed domain error into an envelope — artifact, storage,
/// project, diff and bundle — because those are the failures whose code is a fact about the product
/// rather than about a session's bookkeeping. The hand-written envelopes are not routed here: a
/// `*_missing` reply states that an id from an earlier view did not resolve, and an
/// `ERR-INTERNAL-9001` lock reply states that a thread panicked somewhere else. Neither helps the
/// person reading the file do anything they could not already do.
pub(crate) fn record_error_code(code: &str) {
    let mut held = RECENT_CODES.lock().unwrap_or_else(PoisonError::into_inner);
    held.push(code.to_owned());
    while held.len() > RECENT_CODE_CAPACITY {
        held.remove(0);
    }
}

fn recent_error_codes() -> Vec<String> {
    RECENT_CODES
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone()
}

impl Session {
    /// The payload, assembled from what this session and this process know.
    ///
    /// `webview_version` is resolved by the commands rather than here: `tauri::webview_version()` is a
    /// process-level runtime call, and a test binary should not have to reach a WebView in order to
    /// assert a field this module only bounds and passes through.
    ///
    /// `pub` for the same reason `Session::analyze_and_store` is: the command body adds nothing but the
    /// thread hop and the two facts only a live process knows, so the test drives this path directly.
    pub fn diagnostics(
        &self,
        product: &ProductFacts,
        webview_version: &str,
        operation_id: &str,
    ) -> Result<DiagnosticsDto, ErrorEnvelopeDto> {
        let (store, generated_at) = self.store_diagnostics(operation_id)?;
        Ok(DiagnosticsDto {
            schema: SCHEMA.to_owned(),
            generated_at,
            product: product_dto(product, &self.store_file_name()),
            runtime: runtime_dto(webview_version),
            store,
            git: git_dto(),
            support: DiagnosticsSupportDto {
                input_cohort: INPUT_COHORT.to_owned(),
                install_channel: install_channel(),
            },
            policy: self.policy_diagnostics(operation_id)?,
            recent_error_codes: recent_error_codes(),
        })
    }

    /// The payload as exported text: the same serializer every other JSON document in this product
    /// goes through, plus the trailing newline a file should end with.
    pub fn diagnostics_json(
        &self,
        product: &ProductFacts,
        webview_version: &str,
        operation_id: &str,
    ) -> Result<String, ErrorEnvelopeDto> {
        let dto = self.diagnostics(product, webview_version, operation_id)?;
        let mut text = render::to_json_pretty(&dto);
        text.push('\n');
        Ok(text)
    }

    /// The store section, read from the file this session opened.
    ///
    /// One lock, one pass: health, counts, journal mode, the schema number the *file* carries, and the
    /// clock the engine reads. The snapshot names come from the directory beside it, because §17 asks
    /// that a backup's existence be explainable, and a name is the part that is safe to explain.
    fn store_diagnostics(
        &self,
        operation_id: &str,
    ) -> Result<(DiagnosticsStoreDto, Option<String>), ErrorEnvelopeDto> {
        let db = self.lock_db(operation_id)?;
        let (health, health_summary, health_error_code) = health_of(&db);
        let counts = db.counts();
        let observed_at = db.observed_at();
        let store = DiagnosticsStoreDto {
            schema_version: db.schema_version(),
            supported_schema_version: SCHEMA_VERSION,
            health,
            health_summary,
            health_error_code,
            journal_mode: db.journal_mode().as_str().to_owned(),
            counts: DiagnosticsCountsDto {
                projects: counts.projects,
                builds: counts.builds,
                gate_runs: counts.gate_runs,
                accepted_reviews: counts.accepted_reviews,
                release_records: counts.release_records,
            },
            backup_files: backup_names(&self.store_path),
        };
        Ok((store, observed_at))
    }

    /// The policy section, or `None` when this session has not opened a project.
    fn policy_diagnostics(
        &self,
        operation_id: &str,
    ) -> Result<Option<DiagnosticsPolicyDto>, ErrorEnvelopeDto> {
        let held = self.lock_project(operation_id)?;
        Ok(held.as_ref().map(|loaded| DiagnosticsPolicyDto {
            config_schema_version: loaded.config.schema_version,
            require_clean_git: loaded.policy.require_clean_git,
            require_clean_git_note: REQUIRE_CLEAN_GIT_NOTE.to_owned(),
            require_release_notes: loaded.policy.require_release_notes,
        }))
    }
}

/// The payload itself, for the Help/About section that shows it.
#[tauri::command]
pub(crate) async fn collect_diagnostics(
    app: AppHandle,
    state: State<'_, Arc<Session>>,
) -> Result<DiagnosticsDto, ErrorEnvelopeDto> {
    let session = Arc::clone(&state);
    let operation_id = next_operation_id();
    tauri::async_runtime::spawn_blocking(move || {
        let product = product_facts(&app);
        session.diagnostics(&product, &webview_version(), &operation_id)
    })
    .await
    .map_err(task_join_error)?
}

/// Open the Save dialog, write the payload, and report what happened. Cancelling the dialog is an
/// outcome rather than an error, which is the contract `ExportOutcomeDto` already established for the
/// Compare export.
#[tauri::command]
pub(crate) async fn export_diagnostics(
    app: AppHandle,
    state: State<'_, Arc<Session>>,
) -> Result<ExportOutcomeDto, ErrorEnvelopeDto> {
    let session = Arc::clone(&state);
    let operation_id = next_operation_id();
    tauri::async_runtime::spawn_blocking(move || {
        let product = product_facts(&app);
        let text = session.diagnostics_json(&product, &webview_version(), &operation_id)?;
        let picked = save_path(
            &app,
            "Export the diagnostics file",
            ExportFormat::Json,
            EXPORT_FILE_NAME,
        );
        write_export(picked, &text, ExportFormat::Json, |path| {
            confirm_replacement(&app, &display_name(path))
        })
    })
    .await
    .map_err(task_join_error)?
}

/// What the WebView runtime reports, or `not_reported`.
///
/// `tauri::webview_version()` is the only runtime-version accessor the stack exposes, and prompt §26
/// says to use one only if it answers safely and deterministically: it is a process-level call whose
/// `Err` arm means "this platform did not say", which is exactly what `NOT_REPORTED` carries here. It
/// is read only from a command, never from a test binary.
fn webview_version() -> String {
    tauri::webview_version()
        .map(|value| bound_version(&value, MAX_VERSION_CHARS))
        .unwrap_or_else(|_| NOT_REPORTED.to_owned())
}

/// Health as three states, because "the store is damaged" and "we could not ask" are different facts
/// and both matter to the person reading this file.
///
/// A damaged store reports the engine's bounded, directory-free summary. A check that could not run
/// reports its stable code and nothing else — the message of an open or read failure can quote the
/// path it failed on (§10, §24).
fn health_of(db: &Database) -> (String, Option<String>, Option<String>) {
    match db.integrity_check() {
        Ok(StoreHealth::Healthy) => ("healthy".to_owned(), None, None),
        Ok(StoreHealth::Unhealthy { summary }) => ("unhealthy".to_owned(), Some(summary), None),
        Err(err) => (
            "unknown".to_owned(),
            None,
            Some(err.stable_code().to_owned()),
        ),
    }
}

/// The snapshot file names kept beside the store, in name order.
///
/// Names only. `store_path` is what makes this possible, and it never leaves the Rust side: prompt §7
/// puts a database path outside the payload, and `tests/diagnostics.rs` asserts that the directory
/// these names were read from does not appear in the serialized result.
fn backup_names(store_path: &Path) -> Vec<String> {
    let stem = store_path
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default();
    let prefix = format!("{stem}.pre-migration-");
    let Some(dir) = store_path.parent() else {
        return Vec::new();
    };
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        // The `.sqlite` test also drops a snapshot's `-wal` and `-shm` partners, which are not
        // something a person restores from and not a fact a support file should imply.
        .filter(|name| name.starts_with(&prefix) && name.ends_with(".sqlite"))
        .collect();
    names.sort();
    names.truncate(MAX_BACKUP_NAMES);
    names
}

/// What the binary knows about itself, with the store's logical name taken from the session that
/// opened it (§8): the name is `firmwaresight-p0.sqlite` and the historical `p0` stays in the file
/// name, while the wording a person reads for it is "local FirmwareSight data store".
fn product_dto(product: &ProductFacts, store_file_name: &str) -> DiagnosticsProductDto {
    DiagnosticsProductDto {
        product_name: product.product_name.clone(),
        app_version: product.app_version.clone(),
        binary_name: product.binary_name.clone(),
        identifier: product.identifier.clone(),
        store_file_name: store_file_name.to_owned(),
    }
}

/// The platform this process is running on, from the same compile-time constants the window title and
/// the packages already rely on.
fn runtime_dto(webview_version: &str) -> DiagnosticsRuntimeDto {
    use std::env::consts::{ARCH, OS};

    DiagnosticsRuntimeDto {
        os_family: OS.to_owned(),
        architecture: ARCH.to_owned(),
        platform: format!("{OS}-{ARCH}"),
        os_version: NOT_REPORTED.to_owned(),
        tauri_version: tauri::VERSION.to_owned(),
        webview_version: bound_version(webview_version, MAX_VERSION_CHARS),
    }
}

/// `git --version`, through the same bounded runner the Gate uses, with a fixed argument vector.
///
/// `available` means the executable answered something: a nonzero exit is an answer, a spawn failure or
/// a timeout is not. The text is shape-checked before it is carried, because a string from a child
/// process is not automatically a fact.
fn git_dto() -> DiagnosticsGitDto {
    let probe = GitProbe::system();
    let argv = [OsString::from("--version")];
    let outcome = run_bounded(&probe.program, &argv, probe.timeout, probe.max_output_bytes);
    match outcome {
        RunOutcome::Success { output } => DiagnosticsGitDto {
            available: true,
            version: match output.lines().next() {
                Some(line) if line.trim().starts_with("git version ") => {
                    bound_version(line.trim(), MAX_VERSION_CHARS)
                }
                _ => NOT_REPORTED.to_owned(),
            },
        },
        RunOutcome::Failed { .. } => DiagnosticsGitDto {
            available: true,
            version: NOT_REPORTED.to_owned(),
        },
        RunOutcome::Unavailable | RunOutcome::TimedOut | RunOutcome::Overlong => {
            DiagnosticsGitDto {
                available: false,
                version: NOT_REPORTED.to_owned(),
            }
        }
    }
}

/// How the binary was delivered, from the marker `tauri-build` patches into it.
///
/// This is the only channel identity a running application can state truthfully, so it is the only one
/// reported: prompt §27 refuses an inference from a path string, and a binary run out of a build
/// directory genuinely has no channel — which is what `unknown` means here rather than a failure.
fn install_channel() -> String {
    tauri::utils::platform::bundle_type()
        .map(|bundle| bundle.to_string())
        .unwrap_or_else(|| "unknown".to_owned())
}

/// Keep a version-looking string short and free of control characters, or say it was not reported.
fn bound_version(text: &str, max_chars: usize) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty()
        || trimmed.chars().count() > max_chars
        || trimmed.chars().any(|c| c.is_control())
    {
        return NOT_REPORTED.to_owned();
    }
    trimmed.to_owned()
}
