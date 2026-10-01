//! The Tauri 2 shell. Thin by design: every fact and every verdict comes from Core through the
//! artifact pipeline, and the command bodies contain no business logic.
//!
//! Commands are use-case oriented. There is no path-taking, statement-taking or shell-executing
//! command (`AGENTS.md` §7): the WebView either names a closed fixture key, names a session-local
//! selection id, or names a snapshot id it already received - and the file behind the second was
//! chosen by a person in a native dialog the Rust side opened (`intake`), while the third only
//! selects a bounded page from history (`details`).
//!
//! P2 adds the same discipline to Compare. The UI names two snapshot ids, receives a bounded summary
//! and a session-local diff handle, then pages the changed rows with that handle. Export takes a diff
//! id and nothing else: the save path comes from a dialog the Rust side opened and is not returned to
//! the WebView (`compare`).
//!
//! P3 adds the Release Gate. The UI names a stored build and an optional baseline, and Rust assembles
//! the rest: the policy from the `firmwaresight.toml` a person chose in a dialog the WebView cannot
//! see, the workspace Git facts read through the project adapter, and the build's own facts hydrated
//! from SQLite. Core decides every state and the aggregate; the run is then stored, and a review
//! acceptance is a separate immutable row beside it (`release`).
//!
//! P4 adds the Release Bundle on the same discipline, one step further along the same page. The UI names a
//! prepared plan id, a destination token and a yes/no on replacing an existing bundle — and nothing else.
//! The folder comes from a directory dialog Rust opened and is never returned; the plan is held in memory
//! because it carries every byte it would write; the build it packages is the one this session analyzed,
//! since §22 has to re-read the source files where they are (`bundle`).

#![forbid(unsafe_code)]

pub mod bundle;
pub mod compare;
pub mod details;
pub mod intake;
pub mod ipc;
pub mod release;
pub mod service;

use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use firmwaresight_artifact::ArtifactError;
use firmwaresight_artifact::pipeline::Analysis;
use firmwaresight_core::domain::build_snapshot::BuildSnapshot;
use firmwaresight_core::domain::diff::DiffResult;
use firmwaresight_project::BundlePlan;
use firmwaresight_project::{LoadedProject, ProjectError};
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

/// A diff the session computed, held so rows can be paged without recomputing the comparison.
///
/// The id is a per-process counter in the same shape as a selection id (`cmp-<pid>-<n>`). It is
/// deliberately nothing more: not persisted, not portable, not part of the diff's content identity,
/// and not stable across runs. After a restart the UI re-runs the comparison from the two snapshot
/// ids, and Core's determinism means it gets the same diff back (prompt §20).
#[derive(Debug, Default)]
struct DiffStore {
    next: u64,
    order: VecDeque<String>,
    results: HashMap<String, DiffResult>,
}

/// How many computed diffs to hold. The screen reads one at a time, so eviction is invisible in
/// practice and it bounds memory that would otherwise grow with every click on Compare.
const MAX_SESSION_DIFFS: usize = 8;

impl DiffStore {
    fn insert(&mut self, result: DiffResult) -> String {
        self.next += 1;
        let id = format!("cmp-{:x}-{:x}", std::process::id(), self.next);
        self.order.push_back(id.clone());
        self.results.insert(id.clone(), result);
        while self.order.len() > MAX_SESSION_DIFFS {
            if let Some(oldest) = self.order.pop_front() {
                self.results.remove(&oldest);
            }
        }
        id
    }

    fn get(&self, id: &str) -> Option<&DiffResult> {
        self.results.get(id)
    }
}

/// A build this session analyzed, kept under the id its own content produced.
///
/// This is what makes a Release Bundle possible without a new persistence surface: the portable documents
/// are composed from a `BuildSnapshot` — sections, symbols, capabilities and evidence locators — and SQLite
/// holds those rows for reading, not for re-sealing a snapshot. The bundle engine therefore assembles from
/// the analysis that ran in this session, exactly as Compare pages a diff this session computed. A build
/// from an earlier session is still reviewable in history; packaging it means analyzing it again, which is
/// also what §22 requires, because the source bytes have to be re-read where they are.
#[derive(Debug, Default)]
struct AnalysisStore {
    order: VecDeque<String>,
    builds: HashMap<String, BuildSnapshot>,
}

impl AnalysisStore {
    /// Remember one analysis. Re-analyzing the same bytes is a no-op: the key is the snapshot id, which is a
    /// digest of those bytes.
    fn hold(&mut self, snapshot: &BuildSnapshot) {
        let id = snapshot.id().as_str().to_owned();
        if self.builds.contains_key(&id) {
            return;
        }
        self.order.push_back(id.clone());
        self.builds.insert(id, snapshot.clone());
        while self.order.len() > MAX_SESSION_ANALYSES {
            if let Some(oldest) = self.order.pop_front() {
                self.builds.remove(&oldest);
            }
        }
    }

    fn get(&self, id: &str) -> Option<&BuildSnapshot> {
        self.builds.get(id)
    }
}

/// How many analyses to hold. Eight is the same bound the diff store uses, and it is a memory bound rather
/// than a workflow one: a release owner analyzes a target and a baseline, and the screen reads one at a time.
const MAX_SESSION_ANALYSES: usize = 8;

/// One prepared bundle this session holds (§26, §28).
///
/// The plan carries every byte it would write, so the preview a person authorized and the bundle that
/// appears in their folder cannot turn out to be two different documents. The three ids beside it are what an
/// export re-gathers the same facts with (§28); the destination is *not* stored here, because one plan may be
/// pointed at several folders in a session and the choice belongs to the token, not to the plan.
#[derive(Debug)]
struct HeldBundle {
    plan: BundlePlan,
    snapshot_id: String,
    baseline_snapshot_id: Option<String>,
    gate_run_id: String,
}

/// The session's bundle store, in the same shape as [`DiffStore`]: a per-process counter, bounded, never
/// persisted, and never portable (§26).
#[derive(Debug, Default)]
struct BundleStore {
    next: u64,
    order: VecDeque<String>,
    plans: HashMap<String, HeldBundle>,
}

impl BundleStore {
    fn insert(&mut self, bundle: HeldBundle) -> String {
        self.next += 1;
        let id = format!("bundle-{:x}-{:x}", std::process::id(), self.next);
        self.order.push_back(id.clone());
        self.plans.insert(id.clone(), bundle);
        while self.order.len() > MAX_SESSION_BUNDLES {
            if let Some(oldest) = self.order.pop_front() {
                self.plans.remove(&oldest);
            }
        }
        id
    }

    fn get(&self, id: &str) -> Option<&HeldBundle> {
        self.plans.get(id)
    }
}

/// How many prepared bundles to hold. A person prepares one release at a time; the bound exists so a
/// deliberate loop over Prepare cannot grow the process without limit.
const MAX_SESSION_BUNDLES: usize = 4;

/// The destinations a release owner chose in the native dialog, named by an opaque token (§29).
///
/// A token rather than the plan id, because one plan can be pointed at several folders in a session — the
/// release owner changes their mind — and the UI must be able to say *which* choice it is confirming without
/// ever learning what any of them is.
#[derive(Debug, Default)]
struct DestinationStore {
    next: u64,
    paths: HashMap<String, PathBuf>,
}

impl DestinationStore {
    fn insert(&mut self, path: PathBuf) -> String {
        self.next += 1;
        let token = format!("dst-{:x}-{:x}", std::process::id(), self.next);
        self.paths.insert(token.clone(), path);
        token
    }

    fn get(&self, token: &str) -> Option<&PathBuf> {
        self.paths.get(token)
    }
}

/// The shell's state: a fixture catalog, one SQLite handle, the selections currently staged, the project
/// whose policy the Release page runs against, and the session-local build, bundle and destination stores
/// that make a Release Bundle possible without giving the WebView a path.
pub struct Session {
    catalog: FixtureCatalog,
    db: Mutex<Database>,
    selections: Mutex<SelectionStore>,
    diffs: Mutex<DiffStore>,
    project: Mutex<Option<LoadedProject>>,
    analyses: Mutex<AnalysisStore>,
    bundles: Mutex<BundleStore>,
    destinations: Mutex<DestinationStore>,
}

impl Session {
    /// Open the catalog and database a run needs. Split from Tauri so the parity tests build a
    /// session against a temporary database with no window, runtime or event loop involved.
    pub fn open(catalog: FixtureCatalog, db_path: impl AsRef<Path>) -> Result<Self, StorageError> {
        Ok(Self {
            catalog,
            db: Mutex::new(Database::open(db_path.as_ref())?),
            selections: Mutex::new(SelectionStore::default()),
            diffs: Mutex::new(DiffStore::default()),
            project: Mutex::new(None),
            analyses: Mutex::new(AnalysisStore::default()),
            bundles: Mutex::new(BundleStore::default()),
            destinations: Mutex::new(DestinationStore::default()),
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

    /// Whether this session holds a project policy.
    ///
    /// Only the yes/no is exposed, because the answer decides which save path a command takes: an
    /// existing file is written back to, and with no file the shell opens a dialog instead of inventing a
    /// location. The path itself stays here (`AGENTS.md` 7).
    #[must_use]
    pub fn has_project(&self) -> bool {
        self.project.lock().is_ok_and(|guard| guard.is_some())
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
    ///
    /// Every analysis the shell runs passes through here, which is also where the session remembers the
    /// sealed snapshot a Release Bundle is composed from (`bundle`).
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
        self.lock_analyses(operation_id)?.hold(&analysis.snapshot);
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

    /// Hold a computed diff for the rest of this session and name it.
    ///
    /// # Errors
    ///
    /// An internal envelope if the diff lock was poisoned by an earlier panic.
    pub(crate) fn remember_diff(
        &self,
        result: DiffResult,
        operation_id: &str,
    ) -> Result<String, ErrorEnvelopeDto> {
        let mut store = self.lock_diffs(operation_id)?;
        Ok(store.insert(result))
    }

    /// Read one page or projection out of a diff this session holds.
    ///
    /// The closure runs while the store is locked, so a page never reflects a diff that was evicted
    /// between the lookup and the read.
    ///
    /// # Errors
    ///
    /// `ERR-DIFF-5002` when the id was never issued or has already been dropped.
    pub(crate) fn with_diff<R>(
        &self,
        diff_id: &str,
        operation_id: &str,
        read: impl FnOnce(&DiffResult) -> R,
    ) -> Result<R, ErrorEnvelopeDto> {
        let store = self.lock_diffs(operation_id)?;
        let result = store
            .get(diff_id)
            .ok_or_else(|| diff_missing(diff_id, operation_id))?;
        Ok(read(result))
    }

    fn lock_diffs(
        &self,
        operation_id: &str,
    ) -> Result<std::sync::MutexGuard<'_, DiffStore>, ErrorEnvelopeDto> {
        self.diffs.lock().map_err(|_| ErrorEnvelopeDto {
            code: "ERR-INTERNAL-9001".to_owned(),
            message: "FirmwareSight hit an internal error.".to_owned(),
            operation_id: operation_id.to_owned(),
            details: Some("the diff lock was poisoned by an earlier panic".to_owned()),
            remediation: Some("Report the operation id; no artifact data is needed.".to_owned()),
        })
    }

    /// The project store. Held as one lock because a Gate run reads it while a dialog may be
    /// replacing it, and a run that half-reads a policy would judge the wrong project.
    fn lock_project(
        &self,
        operation_id: &str,
    ) -> Result<std::sync::MutexGuard<'_, Option<LoadedProject>>, ErrorEnvelopeDto> {
        self.project.lock().map_err(|_| ErrorEnvelopeDto {
            code: "ERR-INTERNAL-9001".to_owned(),
            message: "FirmwareSight hit an internal error.".to_owned(),
            operation_id: operation_id.to_owned(),
            details: Some("the project lock was poisoned by an earlier panic".to_owned()),
            remediation: Some("Report the operation id; no artifact data is needed.".to_owned()),
        })
    }

    fn lock_analyses(
        &self,
        operation_id: &str,
    ) -> Result<std::sync::MutexGuard<'_, AnalysisStore>, ErrorEnvelopeDto> {
        self.analyses.lock().map_err(|_| ErrorEnvelopeDto {
            code: "ERR-INTERNAL-9001".to_owned(),
            message: "FirmwareSight hit an internal error.".to_owned(),
            operation_id: operation_id.to_owned(),
            details: Some("the analysis lock was poisoned by an earlier panic".to_owned()),
            remediation: Some("Report the operation id; no artifact data is needed.".to_owned()),
        })
    }

    fn lock_bundles(
        &self,
        operation_id: &str,
    ) -> Result<std::sync::MutexGuard<'_, BundleStore>, ErrorEnvelopeDto> {
        self.bundles.lock().map_err(|_| ErrorEnvelopeDto {
            code: "ERR-INTERNAL-9001".to_owned(),
            message: "FirmwareSight hit an internal error.".to_owned(),
            operation_id: operation_id.to_owned(),
            details: Some("the bundle lock was poisoned by an earlier panic".to_owned()),
            remediation: Some("Report the operation id; no artifact data is needed.".to_owned()),
        })
    }

    fn lock_destinations(
        &self,
        operation_id: &str,
    ) -> Result<std::sync::MutexGuard<'_, DestinationStore>, ErrorEnvelopeDto> {
        self.destinations.lock().map_err(|_| ErrorEnvelopeDto {
            code: "ERR-INTERNAL-9001".to_owned(),
            message: "FirmwareSight hit an internal error.".to_owned(),
            operation_id: operation_id.to_owned(),
            details: Some("the destination lock was poisoned by an earlier panic".to_owned()),
            remediation: Some("Report the operation id; no artifact data is needed.".to_owned()),
        })
    }

    /// The snapshot this session analyzed for one build, or `None` when it never analyzed that build.
    pub(crate) fn held_snapshot(
        &self,
        snapshot_id: &str,
        operation_id: &str,
    ) -> Result<Option<BuildSnapshot>, ErrorEnvelopeDto> {
        Ok(self.lock_analyses(operation_id)?.get(snapshot_id).cloned())
    }

    /// Read one prepared bundle this session holds.
    ///
    /// The closure runs under the store lock, so a bundle cannot be evicted between the lookup and the read —
    /// the same discipline [`Session::with_diff`] keeps for a computed comparison.
    pub(crate) fn with_bundle<R>(
        &self,
        plan_id: &str,
        operation_id: &str,
        read: impl FnOnce(&HeldBundle) -> R,
    ) -> Result<R, ErrorEnvelopeDto> {
        let store = self.lock_bundles(operation_id)?;
        let held = store
            .get(plan_id)
            .ok_or_else(|| bundle_missing(plan_id, operation_id))?;
        Ok(read(held))
    }

    /// Hold one prepared bundle for the rest of this session and name it.
    pub(crate) fn remember_bundle(
        &self,
        bundle: HeldBundle,
        operation_id: &str,
    ) -> Result<String, ErrorEnvelopeDto> {
        Ok(self.lock_bundles(operation_id)?.insert(bundle))
    }

    /// Remember a folder the native dialog produced and return the token that names it (§29).
    pub(crate) fn remember_destination(
        &self,
        path: PathBuf,
        operation_id: &str,
    ) -> Result<String, ErrorEnvelopeDto> {
        Ok(self.lock_destinations(operation_id)?.insert(path))
    }

    /// The folder one destination token names.
    pub(crate) fn destination_of(
        &self,
        token: &str,
        operation_id: &str,
    ) -> Result<PathBuf, ErrorEnvelopeDto> {
        self.lock_destinations(operation_id)?
            .get(token)
            .cloned()
            .ok_or_else(|| destination_missing(token, operation_id))
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

/// A diff id this session never issued, or has already dropped.
///
/// The id is quoted back here, unlike a selection id, because a stale diff handle is something a
/// person can act on: they re-run the comparison from the two builds they can still see.
fn diff_missing(_diff_id: &str, operation_id: &str) -> ErrorEnvelopeDto {
    ErrorEnvelopeDto {
        code: "ERR-DIFF-5002".to_owned(),
        message: "That comparison is no longer available.".to_owned(),
        operation_id: operation_id.to_owned(),
        details: Some(
            "a computed diff is held for the current run only and is dropped after eight more \
             comparisons"
                .to_owned(),
        ),
        remediation: Some("Run the comparison again from the two builds.".to_owned()),
    }
}

/// A bundle plan this session never prepared, or has already dropped.
///
/// The id is quoted back because a person can act on it: Prepare again, from the run they can still see.
/// The plan is session-local by design (§26), so an id from an earlier session was never going to resolve.
fn bundle_missing(_plan_id: &str, operation_id: &str) -> ErrorEnvelopeDto {
    ErrorEnvelopeDto {
        code: "ERR-BUNDLE-6105".to_owned(),
        message: "That bundle plan is no longer available.".to_owned(),
        operation_id: operation_id.to_owned(),
        details: Some(
            "a prepared bundle is held for the current run only, and is dropped after four more \
             preparations"
                .to_owned(),
        ),
        remediation: Some("Prepare the bundle again.".to_owned()),
    }
}

/// A destination token this session never issued.
///
/// The folder a person chose is held under one token and no path crosses the boundary (§29), so an unknown
/// token is not a path that went missing — it is a choice this process never made, and it is refused rather
/// than guessed at.
fn destination_missing(_token: &str, operation_id: &str) -> ErrorEnvelopeDto {
    ErrorEnvelopeDto {
        code: "ERR-BUNDLE-6113".to_owned(),
        message: "That destination was not chosen in this session.".to_owned(),
        operation_id: operation_id.to_owned(),
        details: Some(
            "an export writes only to a folder a release owner picked in the native dialog this process \
             opened, and the token for that choice is held in memory only"
                .to_owned(),
        ),
        remediation: Some("Choose the destination folder again.".to_owned()),
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

/// A project-policy or provenance failure, put into the shape the CLI prints for the same error.
///
/// `hidden` is the config path and the project root. Both appear inside several `ProjectError`
/// messages, because the CLI runs on the machine that owns those paths and a desktop payload does not
/// go to a terminal — it goes to a WebView that has no business holding either (`AGENTS.md` 7).
fn envelope_from_project(
    err: &ProjectError,
    operation_id: &str,
    hidden: &[&Path],
) -> ErrorEnvelopeDto {
    ErrorEnvelopeDto {
        code: err.code().to_owned(),
        // The redacted text is the whole user-facing statement. `details` stays empty rather than
        // repeating it, because the unreduced form of several of these messages is a host path.
        message: redact(&format!("{err}"), hidden),
        operation_id: operation_id.to_owned(),
        details: None,
        remediation: Some(project_remediation(err.code()).to_owned()),
    }
}

fn project_remediation(code: &str) -> &'static str {
    match code {
        "ERR-CONFIG-7001" | "ERR-CONFIG-7002" => {
            "Fix `firmwaresight.toml` and open it again. FirmwareSight refuses a policy it cannot \
             read fully rather than filling the gap with a default."
        }
        "ERR-CONFIG-7003" => {
            "This build reads schema_version 1. Upgrade the config with a FirmwareSight that \
             understands it."
        }
        "ERR-CONFIG-7004" | "ERR-CONFIG-7006" => {
            "Correct the value in the config. A policy word this build does not understand is never \
             guessed at."
        }
        "ERR-CONFIG-7005" => {
            "Keep configured paths inside the project folder, relative to it. An absolute path or a \
             `..` escape is not read."
        }
        "ERR-CONFIG-7007" => {
            "The save was refused so that the keys this build does not understand stayed in the file. \
             Edit those keys by hand, or teach this build about them."
        }
        "ERR-CONFIG-7008" => {
            "Check that the project folder is writable and not locked, then save again. A failed \
             save leaves the previous config untouched."
        }
        "ERR-GIT-8001" => {
            "Git facts degrade to Unknown; the run still happens. Install Git or point the release \
             owner at why the workspace could not be read."
        }
        _ => "Report the operation id; no artifact data is needed.",
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
            intake::analyze_selection,
            details::query_sections,
            details::query_symbols,
            details::query_evidence,
            compare::list_compare_candidates,
            compare::compare_snapshots,
            compare::query_section_changes,
            compare::query_symbol_changes,
            compare::export_compare_json,
            compare::export_compare_html,
            release::open_project_config,
            release::save_project_policy,
            release::run_release_gate,
            release::accept_review,
            release::get_gate_run,
            bundle::prepare_release_bundle,
            bundle::choose_bundle_destination,
            bundle::export_release_bundle
        ])
        .run(tauri::generate_context!())
        .expect("error while running the FirmwareSight desktop application");
}
