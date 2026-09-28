//! The Tauri 2 shell. Thin by design: every fact and every verdict comes from Core through the
//! artifact pipeline, and the command bodies contain no business logic.
//!
//! Commands are use-case oriented. There is no path-taking, SQL-taking or shell-executing command
//! (`AGENTS.md` §7): the WebView either names a closed fixture key or names a session-local
//! selection id, and the file behind that id was chosen by a person in a native dialog the Rust
//! side opened (`intake`).

#![forbid(unsafe_code)]

pub mod intake;
pub mod ipc;
pub mod service;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use firmwaresight_artifact::ArtifactError;
use firmwaresight_artifact::pipeline::Analysis;
use firmwaresight_storage::{Database, StorageError};
use ipc::{AnalysisSummaryDto, ErrorEnvelopeDto, FixtureOptionDto, SelectionDto};
use service::{FixtureCatalog, display_name};

pub use ipc::FixtureKey;

/// The project label P0 history is stored under.
const P0_PROJECT_ID: &str = "p0-desktop";
const P0_PROJECT_NAME: &str = "FirmwareSight P0";

/// The project label a user-selected artifact is stored under.
///
/// P0 needed a demo identity because the only files it could analyze were its own fixtures. P1-A0
/// analyzes files that belong to nobody but the person who chose them, and filing those under a
/// label that says "P0" would be a claim about the data that is not true. No schema change: the
/// same `projects.id`/`projects.name` columns, a different value.
pub const LOCAL_PROJECT_ID: &str = "local-desktop";
pub const LOCAL_PROJECT_NAME: &str = "Local analyses";

/// A file the user chose, plus the optional MAP attached to it. Paths live here and nowhere else:
/// the WebView sees the [`SelectionDto`] projection, which carries names only.
#[derive(Debug)]
struct StagedSelection {
    artifact: PathBuf,
    map: Option<PathBuf>,
}

/// The session's selection store.
///
/// The id is a per-process counter, not a secret: it only has to be unambiguous within this run,
/// and the value of a selection is that the UI can refer to a file it is not allowed to name.
#[derive(Debug, Default)]
struct SelectionStore {
    next: u64,
    staged: HashMap<String, StagedSelection>,
}

impl SelectionStore {
    fn insert(&mut self, artifact: PathBuf, map: Option<PathBuf>) -> String {
        self.next += 1;
        let id = format!("sel-{:x}-{:x}", std::process::id(), self.next);
        self.staged
            .insert(id.clone(), StagedSelection { artifact, map });
        id
    }

    fn get(&self, id: &str) -> Option<&StagedSelection> {
        self.staged.get(id)
    }

    fn staged_paths(&self, id: &str) -> Option<(PathBuf, Option<PathBuf>)> {
        self.staged
            .get(id)
            .map(|s| (s.artifact.clone(), s.map.clone()))
    }
}

/// The shell's state: a fixture catalog, one SQLite handle, and the selections currently staged.
pub struct Session {
    catalog: FixtureCatalog,
    db: Mutex<Database>,
    selections: Mutex<SelectionStore>,
}

impl Session {
    /// Open the catalog and database a run needs. Split from Tauri so the parity tests build a
    /// session against a temporary database with no window, runtime or event loop involved.
    pub fn open(catalog: FixtureCatalog, db_path: impl AsRef<Path>) -> Result<Self, StorageError> {
        Ok(Self {
            catalog,
            db: Mutex::new(Database::open(db_path.as_ref())?),
            selections: Mutex::new(SelectionStore::default()),
        })
    }

    #[must_use]
    pub fn catalog(&self) -> &FixtureCatalog {
        &self.catalog
    }

    /// The closed set the UI may offer. Labels live in Rust so the generated union and the
    /// rendered text cannot disagree.
    #[must_use]
    pub fn fixtures() -> Vec<FixtureOptionDto> {
        [FixtureKey::P0Basic, FixtureKey::P0DualRegion]
            .into_iter()
            .map(|key| FixtureOptionDto {
                key: key.as_key_str().to_owned(),
                label: key.label().to_owned(),
            })
            .collect()
    }

    /// Analyze a committed fixture, project, then persist. Blocking by nature, so it only ever
    /// runs on a worker thread and never on the WebView event loop.
    ///
    /// Public so the parity tests drive the same path the command does; the command adds nothing
    /// but the thread hop.
    pub fn analyze_and_store(
        &self,
        fixture: FixtureKey,
        operation_id: &str,
    ) -> Result<AnalysisSummaryDto, ErrorEnvelopeDto> {
        let artifact_path = self.catalog.artifact_path(fixture);
        let analysis = self
            .catalog
            .analyze(fixture)
            .map_err(|err| envelope_from_artifact(&err, operation_id, &[&artifact_path]))?;
        let summary = service::project(&self.catalog, fixture, &analysis);

        self.store(&analysis, P0_PROJECT_ID, P0_PROJECT_NAME, operation_id)?;
        Ok(summary)
    }

    /// Stage a file the Rust side obtained from the native dialog.
    ///
    /// Nothing is read here: the file is only remembered, so the user can attach a MAP or change
    /// their mind before any parse runs. A path that does not exist is refused now rather than
    /// after a dialog round trip.
    ///
    /// # Errors
    ///
    /// `ERR-INPUT-0001` if the chosen path is not a readable file.
    pub fn stage_artifact(
        &self,
        artifact: impl AsRef<Path>,
        operation_id: &str,
    ) -> Result<SelectionDto, ErrorEnvelopeDto> {
        let artifact = artifact.as_ref();
        require_readable(artifact, operation_id)?;
        let mut store = self.lock_selections(operation_id)?;
        let id = store.insert(artifact.to_path_buf(), None);
        Ok(selection_dto(
            &id,
            store.get(&id).expect("the id was just issued"),
        ))
    }

    /// Attach a MAP to an existing selection.
    ///
    /// # Errors
    ///
    /// `ERR-INPUT-0001` if the selection is unknown or the MAP is not a readable file.
    pub fn stage_map(
        &self,
        selection_id: &str,
        map: impl AsRef<Path>,
        operation_id: &str,
    ) -> Result<SelectionDto, ErrorEnvelopeDto> {
        let map = map.as_ref();
        require_readable(map, operation_id)?;
        self.with_selection(selection_id, operation_id, |staged| {
            staged.map = Some(map.to_path_buf())
        })
    }

    /// Detach the MAP. The artifact selection stays staged.
    ///
    /// # Errors
    ///
    /// `ERR-INPUT-0001` if the selection is unknown.
    pub fn clear_map(
        &self,
        selection_id: &str,
        operation_id: &str,
    ) -> Result<SelectionDto, ErrorEnvelopeDto> {
        self.with_selection(selection_id, operation_id, |staged| staged.map = None)
    }

    /// Mutate one selection under a single lock and return its new projection.
    ///
    /// The check and the write share the guard on purpose: a lookup-then-mutate would let a second
    /// dialog result land between the two, and the `expect` that follows is exactly the panic this
    /// shell is supposed to turn into a typed error.
    fn with_selection(
        &self,
        selection_id: &str,
        operation_id: &str,
        change: impl FnOnce(&mut StagedSelection),
    ) -> Result<SelectionDto, ErrorEnvelopeDto> {
        let mut store = self.lock_selections(operation_id)?;
        let staged = store
            .staged
            .get_mut(selection_id)
            .ok_or_else(|| selection_missing(selection_id, operation_id))?;
        change(staged);
        Ok(selection_dto(selection_id, staged))
    }

    /// What a selection currently holds, as names only.
    #[must_use]
    pub fn selection(&self, selection_id: &str) -> Option<SelectionDto> {
        let store = self.selections.lock().ok()?;
        let staged = store.get(selection_id)?;
        Some(selection_dto(selection_id, staged))
    }

    /// Analyze a staged selection and persist it under the local project.
    ///
    /// The pipeline is the same one the CLI and the fixture path call, so a file the user chose is
    /// reported with the same facts and the same evidence grading as a fixture - including the
    /// degradation when no MAP is attached, and the failure when the bytes are not ELF.
    ///
    /// # Errors
    ///
    /// The typed envelope for an unknown selection, a missing file, unsupported or malformed
    /// bytes, an unusable MAP, or a storage failure.
    pub fn analyze_selection(
        &self,
        selection_id: &str,
        operation_id: &str,
    ) -> Result<AnalysisSummaryDto, ErrorEnvelopeDto> {
        let (artifact, map) = self
            .lock_selections(operation_id)?
            .staged_paths(selection_id)
            .ok_or_else(|| selection_missing(selection_id, operation_id))?;

        let analysis = service::analyze_paths(&artifact, map.as_deref()).map_err(|err| {
            let mut hidden = vec![artifact.as_path()];
            if let Some(path) = &map {
                hidden.push(path.as_path());
            }
            envelope_from_artifact(&err, operation_id, &hidden)
        })?;
        let summary = service::project_artifact(&display_name(&artifact), &analysis);

        self.store(
            &analysis,
            LOCAL_PROJECT_ID,
            LOCAL_PROJECT_NAME,
            operation_id,
        )?;
        Ok(summary)
    }

    /// Import a snapshot unless its content is already in history. Shared by both source paths,
    /// because dedupe must not depend on who chose the file.
    fn store(
        &self,
        analysis: &Analysis,
        project_id: &str,
        project_name: &str,
        operation_id: &str,
    ) -> Result<(), ErrorEnvelopeDto> {
        let db = self.lock_db(operation_id)?;
        if db
            .build_id_for_snapshot(analysis.snapshot.id().as_str())
            .map_err(|err| envelope_from_storage(&err, operation_id))?
            .is_none()
        {
            let mut db = db;
            db.import_snapshot(project_id, project_name, &analysis.snapshot)
                .map_err(|err| envelope_from_storage(&err, operation_id))?;
        }
        Ok(())
    }

    fn lock_db(
        &self,
        operation_id: &str,
    ) -> Result<std::sync::MutexGuard<'_, Database>, ErrorEnvelopeDto> {
        self.db.lock().map_err(|_| ErrorEnvelopeDto {
            code: "ERR-INTERNAL-9001".to_owned(),
            message: "FirmwareSight hit an internal error.".to_owned(),
            operation_id: operation_id.to_owned(),
            details: Some("the storage lock was poisoned by an earlier panic".to_owned()),
            remediation: Some("Report the operation id; no artifact data is needed.".to_owned()),
        })
    }

    fn lock_selections(
        &self,
        operation_id: &str,
    ) -> Result<std::sync::MutexGuard<'_, SelectionStore>, ErrorEnvelopeDto> {
        self.selections.lock().map_err(|_| ErrorEnvelopeDto {
            code: "ERR-INTERNAL-9001".to_owned(),
            message: "FirmwareSight hit an internal error.".to_owned(),
            operation_id: operation_id.to_owned(),
            details: Some("the selection lock was poisoned by an earlier panic".to_owned()),
            remediation: Some("Report the operation id; no artifact data is needed.".to_owned()),
        })
    }
}

/// The names a selection exposes. Paths stay in [`StagedSelection`].
fn selection_dto(selection_id: &str, staged: &StagedSelection) -> SelectionDto {
    SelectionDto {
        selection_id: selection_id.to_owned(),
        file_name: display_name(&staged.artifact),
        map_file_name: staged.map.as_deref().map(display_name),
        map_attached: staged.map.is_some(),
    }
}

/// A handle the session never issued, or no longer holds. The id itself is not repeated back: a
/// user cannot act on it, and quoting it would put an internal value in front of a person.
fn selection_missing(_selection_id: &str, operation_id: &str) -> ErrorEnvelopeDto {
    ErrorEnvelopeDto {
        code: "ERR-INPUT-0001".to_owned(),
        message: "That selection is no longer available.".to_owned(),
        operation_id: operation_id.to_owned(),
        details: Some("the shell holds paths for the current run only; this session never issued, or no longer holds, the handle given".to_owned()),
        remediation: Some("Choose the artifact in the file dialog again.".to_owned()),
    }
}

/// Refuse a path that is not there before a dialog round trip is mistaken for a stored fact.
fn require_readable(path: &Path, operation_id: &str) -> Result<(), ErrorEnvelopeDto> {
    let found = std::fs::metadata(path).is_ok();
    if found {
        return Ok(());
    }
    Err(ErrorEnvelopeDto {
        code: "ERR-INPUT-0001".to_owned(),
        message: "No artifact was found at that path.".to_owned(),
        operation_id: operation_id.to_owned(),
        // The name only: the directory the file came from is not the user's problem to read back.
        details: Some(format!("{} could not be opened", display_name(path))),
        remediation: Some("Check the path, or select the artifact in a file dialog.".to_owned()),
    })
}

/// Codes are `&'static str` from the two registries; the envelope owns its text.
///
/// `hidden` is every path this run knows about. The artifact errors quote the path they failed on,
/// which is right for a CLI on a developer machine and wrong for a payload handed to a WebView:
/// the file name carries the information, the directory does not.
fn envelope_from_artifact(
    err: &ArtifactError,
    operation_id: &str,
    hidden: &[&Path],
) -> ErrorEnvelopeDto {
    ErrorEnvelopeDto {
        code: err.stable_code().to_owned(),
        message: err.user_message(),
        operation_id: operation_id.to_owned(),
        details: Some(redact(&format!("{err}"), hidden)),
        remediation: Some(err.remediation().to_owned()),
    }
}

/// Replace every occurrence of a located path in a diagnostic string with its file name.
fn redact(text: &str, hidden: &[&Path]) -> String {
    let mut out = text.to_owned();
    for path in hidden {
        let located = display_path_of(path);
        if located.is_empty() || !out.contains(&located) {
            continue;
        }
        out = out.replace(&located, &display_name(path));
    }
    out
}

/// The exact string a path is rendered as inside an [`ArtifactError`].
///
/// The artifact crate formats paths through `display_path`, which is `path.display()`, so this has
/// to match it or the redaction silently misses.
fn display_path_of(path: &Path) -> String {
    path.display().to_string()
}

fn envelope_from_storage(err: &StorageError, operation_id: &str) -> ErrorEnvelopeDto {
    ErrorEnvelopeDto {
        code: err.stable_code().to_owned(),
        message: format!("{err}"),
        operation_id: operation_id.to_owned(),
        details: None,
        remediation: Some(err.remediation().to_owned()),
    }
}

/// A per-run identifier for diagnostics only. It never enters a deterministic payload, which is
/// the same rule the CLI follows.
pub(crate) fn next_operation_id() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("op-{:x}-{:x}", std::process::id(), nanos)
}

/// A blocking task that panicked or was cancelled. The join never carries artifact data, so this
/// is reported as an internal error with the operation id attached.
pub(crate) fn task_join_error(join: impl std::fmt::Display) -> ErrorEnvelopeDto {
    ErrorEnvelopeDto {
        code: "ERR-INTERNAL-9001".to_owned(),
        message: "FirmwareSight hit an internal error.".to_owned(),
        operation_id: next_operation_id(),
        details: Some(format!("blocking task panicked or was cancelled: {join}")),
        remediation: Some("Report the operation id; no artifact data is needed.".to_owned()),
    }
}

#[tauri::command]
fn list_fixtures() -> Vec<FixtureOptionDto> {
    Session::fixtures()
}

/// The P0 fixture surface. The argument is a closed key, and the heavy work happens on a blocking
/// thread so the WebView keeps painting.
#[tauri::command]
async fn get_analysis_summary(
    fixture: FixtureKey,
    state: tauri::State<'_, Arc<Session>>,
) -> Result<AnalysisSummaryDto, ErrorEnvelopeDto> {
    let session = Arc::clone(&state);
    let operation_id = next_operation_id();

    tauri::async_runtime::spawn_blocking(move || session.analyze_and_store(fixture, &operation_id))
        .await
        .map_err(task_join_error)?
}

/// Where the P0 desktop reads its fixtures from.
///
/// `FIRMWARESIGHT_FIXTURE_DIR` wins when set; otherwise the repository layout is used. Packaging
/// the fixtures as bundled resources arrives with an installer decision, not before it. The
/// value never comes from the WebView.
fn fixture_catalog() -> FixtureCatalog {
    FixtureCatalog::from_environment()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Registered for the Rust-side dialog only. No capability entry grants the WebView any
        // dialog, filesystem or shell permission (`ADR-0025`).
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            use tauri::Manager;

            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let session =
                Session::open(fixture_catalog(), data_dir.join("firmwaresight-p0.sqlite"))
                    .map_err(|err| format!("{} (code {})", err, err.stable_code()))?;
            app.manage(Arc::new(session));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_fixtures,
            get_analysis_summary,
            intake::select_artifact,
            intake::attach_map,
            intake::clear_map,
            intake::analyze_selection
        ])
        .run(tauri::generate_context!())
        .expect("error while running the FirmwareSight desktop application");
}
