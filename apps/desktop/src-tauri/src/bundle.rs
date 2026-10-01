//! The P4 Release Bundle surface: prepare a plan, choose a destination, export, and record the release.
//!
//! Four rules shape this module, and each one is a boundary the rest of the shell already keeps.
//!
//! 1. **The engine is the CLI's engine.** Nothing here stages a file, hashes a byte or decides whether a
//!    release may be packaged: `firmwaresight_project::bundle` does, because §35 forbids a second bundle
//!    semantics living beside the first. This module gathers facts, calls `prepare`, and calls `publish`.
//! 2. **No path crosses the boundary** (`AGENTS.md` 7). The build is named by a snapshot id, the plan by a
//!    session-local id, and the folder by a token the WebView cannot resolve into anything. The directory a
//!    person chooses in the native dialog is held on this side and never returned — not after a successful
//!    write either: §50 asks for the bundle's *own* folder name, which is a display name rather than a
//!    location.
//! 3. **Cancel is not an error.** A dialog that returns nothing is a decision, and it travels as `Ok(None)`.
//!    So is a refused overwrite: the export reports `ERR-BUNDLE-6106`, and the bundle that was there is
//!    untouched.
//! 4. **A record is written after the bytes, never instead of them.** The release record is the last step of
//!    a successful export; if *that* fails, the error says so plainly rather than pretending the release did
//!    not happen (§57).
//!
//! Codes this module adds to §51's family, all of them about session state rather than about the release:
//! `6111` a release record could not be stored after a bundle was written, `6112` no project policy is
//! loaded, `6113` a destination token this process never issued, `6114` the build was not analyzed in this
//! session. Everything else — `6101`..`6110` — arrives from the engine unchanged, with its own remediation.

use std::path::PathBuf;
use std::sync::Arc;

use firmwaresight_artifact::pipeline::FWSIGHT_VERSION;
use firmwaresight_core::domain::build_snapshot::BuildSnapshot;
use firmwaresight_core::domain::diff::{DiffResult, DiffSnapshotInput, compare};
use firmwaresight_project::{
    BundleAcceptance, BundleError, BundleOutcome, BundleRequest, GitObservation, GitProbe,
    LoadedProject, is_recognizable_bundle, prepare,
};
use firmwaresight_storage::{AcceptedReview, ReleaseRecordDraft, ReleaseRecordWrite};
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use crate::compare::envelope_from_diff;
use crate::ipc::{
    BundleDestinationDto, BundleExportDto, BundleFileRowDto, BundlePlanRequestDto,
    BundlePreviewDto, ErrorEnvelopeDto,
};
use crate::{
    HeldBundle, Session, envelope_from_storage, next_operation_id, release, task_join_error,
};

/// A prepared plan's inputs, gathered as the world stood at one moment.
///
/// Held apart from the plan because an export must re-gather them (§28): the plan carries the bytes it would
/// write, and this carries the identifiers needed to ask the same questions again — the project policy, the
/// workspace Git, the two sealed builds, the run and its acceptances. `build_id` and `baseline_build_id` come
/// from the stored run rather than from the builds, because the record a release writes is a statement about
/// the judgement, not about the folder the bytes were read from.
#[derive(Debug)]
struct BundleInputs {
    project: LoadedProject,
    git: GitObservation,
    snapshot: BuildSnapshot,
    /// The baseline enters only through `comparison`: the engine ships the current artifacts and the diff,
    /// and never the baseline's own bytes (§23).
    comparison: Option<DiffResult>,
    acceptances: Vec<BundleAcceptance>,
    run_id: String,
    build_id: String,
    baseline_build_id: Option<String>,
}

impl BundleInputs {
    /// Borrow the gathered facts as the engine's request.
    ///
    /// `selected_run_id` is always `Some`: a desktop bundle is prepared against a run the release owner can
    /// see on screen, which is exactly the case §9 guards. If the workspace, the config or the Release Notes
    /// moved after that run was recorded, the recomputed id differs and the engine refuses with `6102`.
    fn request(&self) -> BundleRequest<'_> {
        BundleRequest {
            project: &self.project,
            git: &self.git,
            snapshot: &self.snapshot,
            comparison: self.comparison.as_ref(),
            selected_run_id: Some(&self.run_id),
            acceptances: &self.acceptances,
            fwsight_version: FWSIGHT_VERSION,
        }
    }
}

impl Session {
    /// §26 in order: validate the context, recompute the run, load the acceptances, hand the whole set of
    /// facts to the engine. Nothing is written here, and no destination is asked for.
    ///
    /// # Errors
    ///
    /// `ERR-BUNDLE-6112` with no project policy loaded, `ERR-BUNDLE-6114` when this session never analyzed the
    /// build or the baseline, `ERR-STORAGE-4005` for a run that was never stored, and the engine's own
    /// `ERR-BUNDLE-6101`..`6110` for everything a release owner can act on.
    pub fn prepare_bundle(
        &self,
        request: &BundlePlanRequestDto,
        operation_id: &str,
    ) -> Result<BundlePreviewDto, ErrorEnvelopeDto> {
        let inputs = self.gather_bundle(request, operation_id)?;
        let plan =
            prepare(&inputs.request()).map_err(|err| envelope_from_bundle(&err, operation_id))?;

        let accepted = inputs.acceptances.len();
        let preview = plan.preview.clone();
        let folder = plan.proposed_directory_name().to_owned();
        let plan_id = self.remember_bundle(
            HeldBundle {
                plan,
                snapshot_id: request.snapshot_id.clone(),
                baseline_snapshot_id: request.baseline_snapshot_id.clone(),
                gate_run_id: request.gate_run_id.clone(),
            },
            operation_id,
        )?;

        Ok(BundlePreviewDto {
            plan_id,
            release_id: preview.release_id,
            release_version: preview.release_version,
            snapshot_id: preview.snapshot_id,
            baseline_snapshot_id: preview.baseline_snapshot_id,
            gate_run_id: preview.gate_run_id,
            disposition: preview.disposition.to_owned(),
            accepted_review_count: accepted,
            files: preview
                .files
                .iter()
                .map(|file| BundleFileRowDto {
                    path: file.path.clone(),
                    role: file.role.as_str().to_owned(),
                    byte_size: file.byte_size,
                    sha256: file.sha256.clone(),
                })
                .collect(),
            warnings: preview.warnings,
            bundle_folder_name: folder,
        })
    }

    /// Attach the folder a native dialog produced to one plan, and report what the UI may show about it.
    ///
    /// `picked` is `None` on the Cancel path, which is a decision rather than a failure: the plan keeps no
    /// destination, the screen keeps the preview, and nothing was written.
    ///
    /// # Errors
    ///
    /// `ERR-BUNDLE-6105` for a plan this session dropped, and the engine's verification code if the folder
    /// cannot be read as anything at all.
    pub fn choose_bundle_destination(
        &self,
        plan_id: &str,
        picked: Option<PathBuf>,
        operation_id: &str,
    ) -> Result<Option<BundleDestinationDto>, ErrorEnvelopeDto> {
        let Some(parent) = picked else {
            return Ok(None);
        };
        let folder = self.with_bundle(plan_id, operation_id, |held| {
            held.plan.proposed_directory_name().to_owned()
        })?;
        let token = self.remember_destination(parent.clone(), operation_id)?;
        let destination = parent.join(&folder);
        let exists = destination.is_dir();
        Ok(Some(BundleDestinationDto {
            destination_token: token,
            bundle_folder_name: folder,
            exists,
            // §32's question, answered by the same function the engine asks before it replaces anything, so
            // the confirmation dialog and the write cannot disagree about what counts as a bundle.
            recognizable_bundle: exists
                && is_recognizable_bundle(&destination)
                    .map_err(|err| envelope_from_bundle(&err, operation_id))?,
        }))
    }

    /// Write the bundle, then record the release.
    ///
    /// `overwrite` is the release owner's explicit confirmation and nothing more: the engine still refuses a
    /// directory that is not a bundle it wrote (§32), and that refusal arrives as `ERR-BUNDLE-6107`.
    ///
    /// # Errors
    ///
    /// `ERR-BUNDLE-6113` for an unknown destination token, the engine's codes for every refusal that stops a
    /// write, and `ERR-BUNDLE-6111` when the bundle was written and the release record was not.
    pub fn export_bundle(
        &self,
        plan_id: &str,
        destination_token: &str,
        overwrite: bool,
        operation_id: &str,
    ) -> Result<BundleExportDto, ErrorEnvelopeDto> {
        let (plan, request) = self.with_bundle(plan_id, operation_id, |held| {
            (
                held.plan.clone(),
                BundlePlanRequestDto {
                    snapshot_id: held.snapshot_id.clone(),
                    baseline_snapshot_id: held.baseline_snapshot_id.clone(),
                    gate_run_id: held.gate_run_id.clone(),
                },
            )
        })?;
        let parent = self.destination_of(destination_token, operation_id)?;

        // §28: the world is re-read rather than trusted. Anything that moved since the preview — a source
        // file, the notes, the policy, the workspace — stops the write with its own code.
        let inputs = self.gather_bundle(&request, operation_id)?;
        let outcome = plan
            .publish(&inputs.request(), &parent, overwrite)
            .map_err(|err| envelope_from_bundle(&err, operation_id))?;

        let record_written = self.record_release(&outcome, &inputs, operation_id)?;

        Ok(BundleExportDto {
            release_id: outcome.release_id,
            release_version: outcome.release_version,
            folder_display_name: outcome.directory_name,
            manifest_sha256: outcome.manifest_sha256,
            file_count: outcome.verification.file_count,
            total_bytes: outcome.verification.total_bytes,
            artifact_count: outcome.verification.artifact_count,
            replaced: outcome.replaced,
            record_written,
        })
    }

    /// Every fact the engine needs, read as it stands right now.
    fn gather_bundle(
        &self,
        request: &BundlePlanRequestDto,
        operation_id: &str,
    ) -> Result<BundleInputs, ErrorEnvelopeDto> {
        let project = self
            .lock_project(operation_id)?
            .clone()
            .ok_or_else(|| no_project_context(operation_id))?;
        let snapshot = self
            .held_snapshot(&request.snapshot_id, operation_id)?
            .ok_or_else(|| analysis_unavailable(&request.snapshot_id, operation_id))?;
        let baseline = match &request.baseline_snapshot_id {
            None => None,
            Some(id) => Some(
                self.held_snapshot(id, operation_id)?
                    .ok_or_else(|| analysis_unavailable(id, operation_id))?,
            ),
        };
        // One comparison, recomputed from the two sealed builds with Core's own diff: the bytes a reader
        // counts in the bundle's `diff.json` are the bytes the Gate was judged from (§23).
        let comparison = baseline.as_ref().map(|base| {
            compare(
                &DiffSnapshotInput::from_snapshot(base),
                &DiffSnapshotInput::from_snapshot(&snapshot),
            )
            .map_err(|err| envelope_from_diff(&err, operation_id))
        });
        let comparison = match comparison {
            None => None,
            Some(result) => Some(result?),
        };

        let run = {
            let db = self.lock_db(operation_id)?;
            db.gate_run_by_id(&request.gate_run_id)
                .map_err(|err| envelope_from_storage(&err, operation_id))?
        }
        .ok_or_else(|| release::stored_run_missing(&request.gate_run_id, operation_id))?;
        let acceptances = {
            let db = self.lock_db(operation_id)?;
            db.accepted_reviews_for_run(&request.gate_run_id)
                .map_err(|err| envelope_from_storage(&err, operation_id))?
        };

        // Git is read against the folder the config came from — the same observation the Gate judged with,
        // and the reason §9 can tell a moved workspace from an unchanged one.
        let git = GitProbe::system().observe(&project.root);

        Ok(BundleInputs {
            project,
            git,
            snapshot,
            comparison,
            acceptances: acceptances.into_iter().map(acceptance_of).collect(),
            run_id: run.run_id,
            build_id: run.build_id,
            baseline_build_id: run.baseline_build_id,
        })
    }

    /// Persist the release record a successful export produced.
    ///
    /// Returns `false` only when the same release id is already recorded with exactly these facts, which is
    /// the dedupe §37 asks for rather than a failure. A real write error is an envelope whose message states
    /// that the bundle is in the folder — a person reading it must not conclude the release did not happen.
    fn record_release(
        &self,
        outcome: &BundleOutcome,
        inputs: &BundleInputs,
        operation_id: &str,
    ) -> Result<bool, ErrorEnvelopeDto> {
        let mut db = self.lock_db(operation_id)?;
        let draft = ReleaseRecordDraft {
            release_id: &outcome.release_id,
            build_id: &inputs.build_id,
            baseline_build_id: inputs.baseline_build_id.as_deref(),
            gate_run_id: &inputs.run_id,
            release_version: &outcome.release_version,
            manifest_sha256: &outcome.manifest_sha256,
        };
        match db.persist_release_record(&draft) {
            Ok(ReleaseRecordWrite::Inserted) => Ok(true),
            Ok(ReleaseRecordWrite::AlreadyStored) => Ok(false),
            Err(err) => {
                let stored = envelope_from_storage(&err, operation_id);
                Err(ErrorEnvelopeDto {
                    code: "ERR-BUNDLE-6111".to_owned(),
                    message: "The release bundle was written, but its record could not be stored in the \
                              validation database."
                        .to_owned(),
                    operation_id: operation_id.to_owned(),
                    details: Some(format!(
                        "{} — the bundle `{}` and the manifest digest {} are on disk either way",
                        stored.message, outcome.directory_name, outcome.manifest_sha256
                    )),
                    remediation: Some(
                        "Keep the bundle. Export the same release again to record it, or report this \
                         operation id with the release id."
                            .to_owned(),
                    ),
                })
            }
        }
    }
}

/// One accepted review, as the engine wants it rather than as the audit row stores it.
fn acceptance_of(accepted: AcceptedReview) -> BundleAcceptance {
    BundleAcceptance {
        finding_id: accepted.finding_id,
        actor: accepted.actor,
        accepted_at: accepted.accepted_at,
        reason: accepted.reason,
    }
}

/// Map an engine refusal onto the shell's envelope.
///
/// The code and the remediation are the engine's own, so a UI shows the same words the CLI prints for the
/// same condition — and no path appears, because every name in a `BundleError` is bundle-relative or a
/// display name.
fn envelope_from_bundle(err: &BundleError, operation_id: &str) -> ErrorEnvelopeDto {
    ErrorEnvelopeDto {
        code: err.code().to_owned(),
        message: err.to_string(),
        operation_id: operation_id.to_owned(),
        details: None,
        remediation: Some(err.remediation().to_owned()),
    }
}

/// A bundle needs a policy: §11 resolves the release version from it, and §44 records its fingerprint.
fn no_project_context(operation_id: &str) -> ErrorEnvelopeDto {
    ErrorEnvelopeDto {
        code: "ERR-BUNDLE-6112".to_owned(),
        message: "No project policy is loaded, so there is nothing to release under.".to_owned(),
        operation_id: operation_id.to_owned(),
        details: Some(
            "a bundle states a release version, a policy fingerprint and Release Notes, all of which \
             come from the project's firmwaresight.toml"
                .to_owned(),
        ),
        remediation: Some(
            "Choose the project's firmwaresight.toml on the Release page, then run the Gate again."
                .to_owned(),
        ),
    }
}

/// The build was never analyzed in this session, so its sealed snapshot is not here to package.
///
/// The id is shortened, because a person cannot act on a 64-character digest in a dialog; the full one is in
/// the evidence inspector beside the build they can already see.
fn analysis_unavailable(snapshot_id: &str, operation_id: &str) -> ErrorEnvelopeDto {
    ErrorEnvelopeDto {
        code: "ERR-BUNDLE-6114".to_owned(),
        message: "That build was not analyzed in this session, so it cannot be packaged."
            .to_owned(),
        operation_id: operation_id.to_owned(),
        details: Some(format!(
            "a bundle is composed from the analysis of build {} and re-reads its source files, both of \
             which this shell holds for the current run only",
            short_id(snapshot_id)
        )),
        remediation: Some(
            "Analyze that artifact again, run the Gate over it, then prepare the bundle."
                .to_owned(),
        ),
    }
}

/// A snapshot id shortened for a sentence.
fn short_id(snapshot_id: &str) -> String {
    snapshot_id.chars().take(12).collect()
}

/// Open the native directory chooser. `None` is Cancel (§30).
///
/// A folder rather than a file, and chosen by a person: the WebView never names a directory, which is what
/// lets §31 promise that no bundle is ever written somewhere nobody pointed at.
fn choose_destination_folder(app: &AppHandle) -> Option<PathBuf> {
    app.dialog()
        .file()
        .set_title("Choose the folder to create the release bundle in")
        .blocking_pick_folder()
        .and_then(|picked| picked.into_path().ok())
}

/// Prepare one bundle plan and return the bounded preview a release owner authorizes (§26, §27).
#[tauri::command]
pub(crate) async fn prepare_release_bundle(
    request: BundlePlanRequestDto,
    state: State<'_, Arc<Session>>,
) -> Result<BundlePreviewDto, ErrorEnvelopeDto> {
    let session = Arc::clone(&state);
    let operation_id = next_operation_id();

    tauri::async_runtime::spawn_blocking(move || session.prepare_bundle(&request, &operation_id))
        .await
        .map_err(task_join_error)?
}

/// Choose where the bundle goes. Resolves to `None` when the release owner cancels, which leaves the preview
/// on screen and the plan without a destination (§29, §30).
#[tauri::command]
pub(crate) async fn choose_bundle_destination(
    app: AppHandle,
    plan_id: String,
    state: State<'_, Arc<Session>>,
) -> Result<Option<BundleDestinationDto>, ErrorEnvelopeDto> {
    let session = Arc::clone(&state);
    let operation_id = next_operation_id();

    let picked = tauri::async_runtime::spawn_blocking(move || choose_destination_folder(&app))
        .await
        .map_err(task_join_error)?;

    session.choose_bundle_destination(&plan_id, picked, &operation_id)
}

/// Write the bundle the release owner previewed.
///
/// `overwrite` arrives only from an explicit confirmation on the screen (§30): the default is no, and no
/// flag changes what the engine will replace (§35).
#[tauri::command]
pub(crate) async fn export_release_bundle(
    plan_id: String,
    destination_token: String,
    overwrite: bool,
    state: State<'_, Arc<Session>>,
) -> Result<BundleExportDto, ErrorEnvelopeDto> {
    let session = Arc::clone(&state);
    let operation_id = next_operation_id();

    tauri::async_runtime::spawn_blocking(move || {
        session.export_bundle(&plan_id, &destination_token, overwrite, &operation_id)
    })
    .await
    .map_err(task_join_error)?
}
