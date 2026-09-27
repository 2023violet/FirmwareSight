//! The Tauri 2 shell. Thin by design: every fact and every verdict comes from Core through the
//! artifact pipeline, and the command bodies contain no business logic.
//!
//! Two commands exist, both use-case oriented. There is no path-taking, SQL-taking or
//! shell-executing command (`AGENTS.md` §7), and the WebView cannot reach the filesystem except
//! through the closed fixture set.

#![forbid(unsafe_code)]

pub mod ipc;
pub mod service;

use std::path::Path;
use std::sync::{Arc, Mutex};

use firmwaresight_artifact::ArtifactError;
use firmwaresight_storage::{Database, StorageError};
use ipc::{AnalysisSummaryDto, ErrorEnvelopeDto, FixtureOptionDto};
use service::FixtureCatalog;

pub use ipc::FixtureKey;

/// The project label P0 history is stored under.
const P0_PROJECT_ID: &str = "p0-desktop";
const P0_PROJECT_NAME: &str = "FirmwareSight P0";

/// The shell's only capability: a fixture catalog and one SQLite handle.
pub struct Session {
    catalog: FixtureCatalog,
    db: Mutex<Database>,
}

impl Session {
    /// Open the catalog and database a run needs. Split from Tauri so the parity tests build a
    /// session against a temporary database with no window, runtime or event loop involved.
    pub fn open(catalog: FixtureCatalog, db_path: impl AsRef<Path>) -> Result<Self, StorageError> {
        Ok(Self {
            catalog,
            db: Mutex::new(Database::open(db_path.as_ref())?),
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

    /// Analyze, project, then persist. Blocking by nature, so it only ever runs on a worker
    /// thread and never on the WebView event loop.
    ///
    /// Public so the parity tests drive the same path the command does; the command adds nothing
    /// but the thread hop.
    pub fn analyze_and_store(
        &self,
        fixture: FixtureKey,
        operation_id: &str,
    ) -> Result<AnalysisSummaryDto, ErrorEnvelopeDto> {
        let analysis = self
            .catalog
            .analyze(fixture)
            .map_err(|err| envelope_from_artifact(&err, operation_id))?;
        let summary = service::project(&self.catalog, fixture, &analysis);

        let db = self.lock_db(operation_id)?;
        if db
            .build_id_for_snapshot(analysis.snapshot.id().as_str())
            .map_err(|err| envelope_from_storage(&err, operation_id))?
            .is_none()
        {
            let mut db = db;
            db.import_snapshot(P0_PROJECT_ID, P0_PROJECT_NAME, &analysis.snapshot)
                .map_err(|err| envelope_from_storage(&err, operation_id))?;
        }

        Ok(summary)
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
}

/// Codes are `&'static str` from the two registries; the envelope owns its text.
fn envelope_from_artifact(err: &ArtifactError, operation_id: &str) -> ErrorEnvelopeDto {
    ErrorEnvelopeDto {
        code: err.stable_code().to_owned(),
        message: err.user_message(),
        operation_id: operation_id.to_owned(),
        details: Some(format!("{err}")),
        remediation: Some(err.remediation().to_owned()),
    }
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
fn next_operation_id() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("op-{:x}-{:x}", std::process::id(), nanos)
}

#[tauri::command]
fn list_fixtures() -> Vec<FixtureOptionDto> {
    Session::fixtures()
}

/// The single analysis surface. The argument is a closed key, and the heavy work happens on a
/// blocking thread so the WebView keeps painting.
#[tauri::command]
async fn get_analysis_summary(
    fixture: FixtureKey,
    state: tauri::State<'_, Arc<Session>>,
) -> Result<AnalysisSummaryDto, ErrorEnvelopeDto> {
    let session = Arc::clone(&state);
    let operation_id = next_operation_id();

    tauri::async_runtime::spawn_blocking(move || session.analyze_and_store(fixture, &operation_id))
        .await
        .map_err(|join| ErrorEnvelopeDto {
            code: "ERR-INTERNAL-9001".to_owned(),
            message: "FirmwareSight hit an internal error.".to_owned(),
            operation_id: next_operation_id(),
            details: Some(format!("blocking task panicked or was cancelled: {join}")),
            remediation: Some("Report the operation id; no artifact data is needed.".to_owned()),
        })?
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
            get_analysis_summary
        ])
        .run(tauri::generate_context!())
        .expect("error while running the FirmwareSight desktop application");
}
