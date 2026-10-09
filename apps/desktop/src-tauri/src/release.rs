//! The P3 Release Gate surface: load a project policy, judge a stored build against it, store the run,
//! and let a named person accept a review.
//!
//! Four rules shape this module.
//!
//! 1. **Core decides every state.** Nothing here classifies, thresholds or aggregates. The findings come
//!    from `firmwaresight_core::domain::gate`; this layer assembles facts, stores the answer, and reads
//!    states back out of the findings it stored (`AGENTS.md` 3).
//! 2. **The project root never leaves Rust.** A `firmwaresight.toml` is chosen in a dialog this side
//!    opened, and the WebView gets the project name, the file's name, the policy fields and the warnings —
//!    never the directory, never the raw text (prompt §40, §46).
//! 3. **A run is judged against a policy that is named.** `policy_source` says whether the run used a
//!    project's config or FirmwareSight's stated default, so a default-policy run can never be read as
//!    somebody's policy (prompt §40).
//! 4. **Git describes the workspace, not the artifact.** Every sentence here is about the workspace HEAD,
//!    its tags and its dirtiness. Nothing says an artifact was built from a commit, because the Gate holds
//!    no evidence of a build (prompt §50).
//!
//! The numbers under the findings (`tables`) are sent only for a run this session computed. A stored run
//! keeps its findings, its evidence locators and its acceptances, but not the policy text it was judged
//! against, so re-projecting a budget or a threshold from a different policy would state a verdict the
//! original run never made.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use firmwaresight_core::domain::diff::{BudgetState, compare};
use firmwaresight_core::domain::gate::{
    EffectiveSeverity, FindingState, GateArtifactFact, GateBudgetFact, GateFileFact,
    GateFileStatus, GateFinding, GateGitFacts, GateGrowthFacts, GateMemoryFacts, GatePolicy,
    GateRuleId, GateStateCounts, UnknownDisposition, UnknownPolicy, VersionPolicy, aggregate,
};
use firmwaresight_core::domain::identity::{ArtifactKind, Fact};
use firmwaresight_project::LoadedProject;
use firmwaresight_project::config::{new_config, validate, write_config};
use firmwaresight_project::evidence::{
    SnapshotFacts, UNKNOWN_EVIDENCE_SAMPLE, budget_fact, growth_facts, unknown_evidence_from_ids,
};
use firmwaresight_project::{
    CONFIG_FILE_NAME, FOOTPRINT_EVIDENCE_FIELDS, GateRunRequest, GitObservation, GitProbe,
    build_context, observe_release_notes, policy_sha256,
};
use firmwaresight_storage::{
    AcceptReviewError, AcceptedReview, GateRunDraft, GateSnapshotFacts, StoredBudget,
    StoredGateFinding, StoredGateRun,
};
use tauri::{AppHandle, State};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};

use crate::compare::envelope_from_diff;
use crate::intake::pick_path;
use crate::ipc::{
    AcceptReviewOutcomeDto, AcceptReviewRequestDto, AcceptedReviewDto, ErrorEnvelopeDto,
    GateArtifactRowDto, GateBudgetRowDto, GateCountsDto, GateFindingRowDto, GateGitDto,
    GateGrowthRowDto, GateNotesDto, GateRunDto, GateRunRequestDto, GateTablesDto,
    ProjectContextDto, ProjectPolicyDto, UnknownDispositionDto, UnknownPolicyDto,
};
use crate::{
    Session, envelope_from_project, envelope_from_storage, next_operation_id, service,
    task_join_error,
};

/// What a run was judged against, as the payload has to say it.
///
/// `project_name` is `None` for a run read back from history whose policy is not loaded in this session:
/// the stored record keeps the policy *fingerprint*, and a name attached to a different policy would be a
/// claim about a run this session did not make.
#[derive(Debug)]
struct PolicyLabel {
    project_name: Option<String>,
    source: String,
    warnings: Vec<String>,
}

const DEFAULT_POLICY_SOURCE: &str = "FirmwareSight default policy (no project config loaded)";

/// What a historical record does and does not show — stated by Rust rather than inferred by the screen,
/// because the difference is a fact about the payload.
const RECORD_NOTE: &str = "This run was read back from history. Its findings, evidence and accepted \
                           reviews are the stored record; the policy it was judged against is not part \
                           of that record, so its numbers stay inside each finding.";

impl Session {
    /// Load a project config the dialog produced and make it the session's policy.
    ///
    /// `None` is the Cancel path and is not an error: the Release page keeps the project it already had
    /// (prompt §40). A config that cannot be read is a typed failure, and the previously loaded project
    /// stays loaded — a refused open never leaves the page without a policy.
    ///
    /// Public because the integration tests drive the same path the command does; the command adds
    /// nothing but the dialog and the thread hop.
    ///
    /// # Errors
    ///
    /// The typed `ERR-CONFIG-*` envelope for an unreadable, malformed or unsupported config.
    pub fn load_project_from(
        &self,
        picked: Option<PathBuf>,
        operation_id: &str,
    ) -> Result<Option<ProjectContextDto>, ErrorEnvelopeDto> {
        let Some(path) = picked else {
            return Ok(None);
        };
        self.store_project(LoadedProject::load_file(&path), &path, operation_id)
            .map(Some)
    }

    /// Write the edited policy back to the loaded config.
    ///
    /// `None` means no project is loaded, so there is no file to write to; the command answers that with a
    /// native save dialog rather than inventing a location.
    ///
    /// # Errors
    ///
    /// `ERR-CONFIG-7007` when the file holds keys this build does not understand — the save is refused so
    /// those keys survive — and the typed envelope for any other policy or write failure.
    pub fn save_policy_in_project(
        &self,
        request: &ProjectPolicyDto,
        operation_id: &str,
    ) -> Result<Option<ProjectContextDto>, ErrorEnvelopeDto> {
        let Some(loaded) = self.lock_project(operation_id)?.clone() else {
            return Ok(None);
        };
        let path = loaded.path.clone();
        loaded
            .save_policy(&gate_policy(request))
            .map_err(|err| envelope_from_project(&err, operation_id, &hidden_of(&path)))?;
        self.store_project(LoadedProject::load_file(&path), &path, operation_id)
            .map(Some)
    }

    /// Create a first config at the path the save dialog produced, then load it.
    ///
    /// # Errors
    ///
    /// The typed envelope when the policy is invalid or the write fails. A failed write leaves whatever was
    /// at that path untouched, because `write_config` stages the bytes beside the target and swaps them in
    /// only after reading them back (prompt §42).
    pub fn save_policy_as_new(
        &self,
        picked: Option<PathBuf>,
        request: &ProjectPolicyDto,
        operation_id: &str,
    ) -> Result<Option<ProjectContextDto>, ErrorEnvelopeDto> {
        let Some(path) = picked else {
            return Ok(None);
        };
        let hidden = hidden_of(&path);
        let config = new_config(&request.project_name, &gate_policy(request));
        validate(&config).map_err(|err| envelope_from_project(&err, operation_id, &hidden))?;
        write_config(&path, &config)
            .map_err(|err| envelope_from_project(&err, operation_id, &hidden))?;
        self.store_project(LoadedProject::load_file(&path), &path, operation_id)
            .map(Some)
    }

    /// Judge one stored build against the policy in force, store the run, and return it.
    ///
    /// The build's facts are read from SQLite rather than re-analyzed, so a Gate run works on a build whose
    /// files have moved, and the run a reviewer accepted last week is the same run today.
    ///
    /// # Errors
    ///
    /// `ERR-STORAGE-4005` when a snapshot id names no stored build, `ERR-DIFF-5001` when both ids name the
    /// same build, and the storage envelope for a failed read or write.
    pub fn run_release_gate(
        &self,
        request: &GateRunRequestDto,
        operation_id: &str,
    ) -> Result<GateRunDto, ErrorEnvelopeDto> {
        let (stored, footprint_evidence) = {
            let db = self.lock_db(operation_id)?;
            let stored = db
                .load_gate_facts(&request.snapshot_id)
                .map_err(|err| envelope_from_storage(&err, operation_id))?;
            // The two pointers a fresh analysis would have quoted for these totals, read from the same
            // build. Without them the desktop and the CLI fingerprint the same judgement differently,
            // and a run id is meant to name one answer, not one answer per surface.
            let evidence = FOOTPRINT_EVIDENCE_FIELDS.map(|field| {
                db.evidence_id_for_field(&stored.build_id, field)
                    .map_err(|err| envelope_from_storage(&err, operation_id))
            });
            match evidence {
                [Ok(nonvolatile), Ok(runtime_ram)] => (stored, [nonvolatile, runtime_ram]),
                [Err(err), _] | [Ok(_), Err(err)] => return Err(err),
            }
        };
        let baseline_build = match &request.baseline_snapshot_id {
            None => None,
            Some(id) => Some({
                let db = self.lock_db(operation_id)?;
                db.build_id_for_snapshot(id)
                    .map_err(|err| envelope_from_storage(&err, operation_id))?
                    .ok_or_else(|| snapshot_missing(id, operation_id))?
            }),
        };
        // The baseline is measured with the same Core diff Compare shows, so the growth a Gate asks a
        // person to review and the growth they read in the change tables are one calculation (§23).
        let growth = self.growth_for(request, operation_id)?;
        let target = snapshot_facts(&stored, &footprint_evidence);

        let project = self.lock_project(operation_id)?.clone();
        let (policy, label) = match &project {
            Some(loaded) => (
                loaded.policy.clone(),
                PolicyLabel {
                    project_name: Some(loaded.project_name().to_owned()),
                    source: CONFIG_FILE_NAME.to_owned(),
                    warnings: loaded.warnings.clone(),
                },
            ),
            None => (
                GatePolicy::default(),
                PolicyLabel {
                    project_name: None,
                    source: DEFAULT_POLICY_SOURCE.to_owned(),
                    warnings: Vec::new(),
                },
            ),
        };

        // Git is read here and only when there is a workspace to read. With no project loaded the honest
        // answer is that nothing was observed, not that the tree is clean.
        let git = match &project {
            Some(loaded) => GitProbe::system().observe(&loaded.root),
            None => GitObservation::unavailable(
                "no project configuration is loaded, so there is no workspace to read",
            ),
        };
        // Notes are observed only when the policy asks for them and there is a project folder to look in.
        // With no folder the fact is simply absent, and `release.notes` says so as `UNKNOWN` rather than
        // claiming a file is missing where nothing was read (prompt §53).
        let notes = match (&project, policy.require_release_notes) {
            (Some(loaded), true) => Some(observe_release_notes(
                &loaded.root,
                &policy.release_notes_path,
            )),
            _ => None,
        };

        let context = build_context(&GateRunRequest {
            target: &target,
            growth: growth.clone(),
            git: &git,
            release_notes: notes.clone(),
            policy: policy.clone(),
        });
        let generated = firmwaresight_project::run_id(&context);
        let evaluation = context.evaluate(&generated);
        let digest = policy_sha256(&policy);

        {
            let mut db = self.lock_db(operation_id)?;
            db.persist_gate_run(&GateRunDraft {
                run_id: &generated,
                build_id: &stored.build_id,
                baseline_build_id: baseline_build.as_deref(),
                policy_sha256: &digest,
                evaluation: &evaluation,
                // Nothing attaches a file yet: no surface offers a choice, so the run that is judged
                // and stored here binds an empty set (`04_TECH/28` §3, C1-U1's boundary).
                attachments: &[],
            })
            .map_err(|err| envelope_from_storage(&err, operation_id))?;
        }

        let run = self.stored_run(&generated, operation_id)?;
        let acceptances = self.acceptances_for(&generated, operation_id)?;
        let findings = stored_findings(&run, operation_id)?;
        Ok(gate_run_dto(
            &run,
            &findings,
            &acceptances,
            &label,
            &target.snapshot_id,
            request.baseline_snapshot_id.clone(),
            git_dto(&git.facts),
            Some(&policy),
            Some(tables(&findings, &policy, &target, &growth, notes.as_ref())),
            None,
        ))
    }

    /// Record that a named person accepted one `REVIEW` finding, and report what that did to the run.
    ///
    /// The finding is not touched. Two places change on the screen — the row gains its acceptance and the
    /// aggregate may drop one review — which is why this returns a small outcome rather than a whole run
    /// (prompt §48).
    ///
    /// # Errors
    ///
    /// `ERR-STORAGE-4007`..`4010` for an unknown finding, a finding that is not a review, a second
    /// acceptance, and an empty actor or reason; `ERR-STORAGE-4005` when the run is not stored.
    pub fn accept_gate_review(
        &self,
        request: &AcceptReviewRequestDto,
        operation_id: &str,
    ) -> Result<AcceptReviewOutcomeDto, ErrorEnvelopeDto> {
        let acceptance = {
            let mut db = self.lock_db(operation_id)?;
            db.accept_review(
                &request.run_id,
                &request.finding_id,
                &request.actor,
                &request.reason,
            )
        };
        let acceptance = acceptance.map_err(|err| envelope_from_acceptance(&err, operation_id))?;

        let run = self.stored_run(&request.run_id, operation_id)?;
        let finding = run
            .findings
            .iter()
            .find(|row| row.id == acceptance.finding_id)
            .ok_or_else(|| acceptance_finding_missing(&acceptance.finding_id, operation_id))?;
        let acceptances = self.acceptances_for(&request.run_id, operation_id)?;
        let findings = stored_findings(&run, operation_id)?;

        Ok(AcceptReviewOutcomeDto {
            run_id: run.run_id.clone(),
            finding_id: finding.id.clone(),
            // The accepted row still reads REVIEW. Nothing in this payload lets the screen turn an
            // accepted review into a pass.
            state: finding.state.as_str().to_owned(),
            disposition_effective_severity: aggregate(&findings, &accepted_ids(&acceptances))
                .as_str()
                .to_owned(),
            acceptance: acceptance_dto(&acceptance),
        })
    }

    /// Read a stored run back, findings and acceptances included.
    ///
    /// `None` is the honest answer to "what does history say about this id" when history says nothing: the
    /// id came from another database or was typed. It is not a failure of this shell, so it travels as an
    /// empty result the screen labels rather than as an envelope (prompt §49).
    ///
    /// # Errors
    ///
    /// The storage envelope for a failed read, and `ERR-INTERNAL-9003` if a stored finding names a rule this
    /// build does not know.
    pub fn gate_run(
        &self,
        id: &str,
        operation_id: &str,
    ) -> Result<Option<GateRunDto>, ErrorEnvelopeDto> {
        let run = {
            let db = self.lock_db(operation_id)?;
            db.gate_run_by_id(id)
                .map_err(|err| envelope_from_storage(&err, operation_id))?
        };
        let Some(run) = run else {
            return Ok(None);
        };
        let acceptances = self.acceptances_for(id, operation_id)?;
        let findings = stored_findings(&run, operation_id)?;

        let (snapshot_id, baseline_snapshot_id) = {
            let db = self.lock_db(operation_id)?;
            let snapshot_id = db
                .summary(&run.build_id)
                .map_err(|err| envelope_from_storage(&err, operation_id))?
                .snapshot_id;
            let baseline = match &run.baseline_build_id {
                Some(build) => Some(
                    db.summary(build)
                        .map_err(|err| envelope_from_storage(&err, operation_id))?
                        .snapshot_id,
                ),
                None => None,
            };
            (snapshot_id, baseline)
        };

        // The policy a historical run used is identified by its fingerprint only. When this session holds
        // that same policy, its name and its warnings are the run's own and saying them costs nothing;
        // otherwise both stay unstated.
        let project = self.lock_project(operation_id)?.clone();
        let label = match project.filter(|loaded| loaded.policy_sha256 == run.policy_sha256) {
            Some(loaded) => PolicyLabel {
                project_name: Some(loaded.project_name().to_owned()),
                source: CONFIG_FILE_NAME.to_owned(),
                warnings: loaded.warnings.clone(),
            },
            None => PolicyLabel {
                project_name: None,
                source: format!(
                    "policy {} (read back from history)",
                    short_hash(&run.policy_sha256)
                ),
                warnings: Vec::new(),
            },
        };

        Ok(Some(gate_run_dto(
            &run,
            &findings,
            &acceptances,
            &label,
            &snapshot_id,
            baseline_snapshot_id,
            GateGitDto {
                available: false,
                head_commit: None,
                exact_tag: None,
                dirty: None,
                summary: "Workspace not re-observed".to_owned(),
                reason: Some(
                    "a stored record keeps the Git findings it made; the workspace is read again only \
                     by a new run"
                        .to_owned(),
                ),
            },
            None,
            None,
            Some(RECORD_NOTE.to_owned()),
        )))
    }

    /// The growth facts for one request, from P2's Core diff over the two stored builds.
    fn growth_for(
        &self,
        request: &GateRunRequestDto,
        operation_id: &str,
    ) -> Result<GateGrowthFacts, ErrorEnvelopeDto> {
        let Some(baseline_id) = &request.baseline_snapshot_id else {
            return Ok(GateGrowthFacts::without_baseline());
        };
        let (base, target) = {
            let db = self.lock_db(operation_id)?;
            let base = db
                .load_diff_input(baseline_id)
                .map_err(|err| envelope_from_storage(&err, operation_id))?;
            let target = db
                .load_diff_input(&request.snapshot_id)
                .map_err(|err| envelope_from_storage(&err, operation_id))?;
            (base, target)
        };
        let diff = compare(&base, &target).map_err(|err| envelope_from_diff(&err, operation_id))?;
        Ok(growth_facts(&diff))
    }

    fn store_project(
        &self,
        loaded: Result<LoadedProject, firmwaresight_project::ProjectError>,
        path: &Path,
        operation_id: &str,
    ) -> Result<ProjectContextDto, ErrorEnvelopeDto> {
        let loaded =
            loaded.map_err(|err| envelope_from_project(&err, operation_id, &hidden_of(path)))?;
        let dto = project_dto(&loaded);
        *self.lock_project(operation_id)? = Some(loaded);
        Ok(dto)
    }

    fn stored_run(&self, id: &str, operation_id: &str) -> Result<StoredGateRun, ErrorEnvelopeDto> {
        let db = self.lock_db(operation_id)?;
        db.gate_run_by_id(id)
            .map_err(|err| envelope_from_storage(&err, operation_id))?
            .ok_or_else(|| stored_run_missing(id, operation_id))
    }

    fn acceptances_for(
        &self,
        id: &str,
        operation_id: &str,
    ) -> Result<Vec<AcceptedReview>, ErrorEnvelopeDto> {
        let db = self.lock_db(operation_id)?;
        db.accepted_reviews_for_run(id)
            .map_err(|err| envelope_from_storage(&err, operation_id))
    }
}

/// The whole payload for one run, shared by the fresh and the historical path so the two cannot drift.
#[allow(clippy::too_many_arguments)]
fn gate_run_dto(
    run: &StoredGateRun,
    findings: &[GateFinding],
    acceptances: &[AcceptedReview],
    label: &PolicyLabel,
    snapshot_id: &str,
    baseline_snapshot_id: Option<String>,
    git: GateGitDto,
    policy: Option<&GatePolicy>,
    tables: Option<GateTablesDto>,
    record_note: Option<String>,
) -> GateRunDto {
    let counts = GateStateCounts::of(findings);
    let accepted = accepted_ids(acceptances);
    GateRunDto {
        run_id: run.run_id.clone(),
        snapshot_id: snapshot_id.to_owned(),
        baseline_snapshot_id,
        project_name: label.project_name.clone(),
        policy_source: label.source.clone(),
        policy_sha256: run.policy_sha256.clone(),
        created_at: run.created_at.clone(),
        overall_effective_severity: run.overall_effective_severity.as_str().to_owned(),
        disposition_effective_severity: aggregate(findings, &accepted).as_str().to_owned(),
        counts: GateCountsDto {
            pass: counts.pass,
            review: counts.review,
            block: counts.block,
            unknown: counts.unknown,
            not_applicable: counts.not_applicable,
        },
        findings: run
            .findings
            .iter()
            .map(|finding| finding_dto(finding, acceptances))
            .collect(),
        warnings: label.warnings.clone(),
        git,
        policy: policy
            .map(|used| policy_dto(used, label.project_name.as_deref().unwrap_or_default())),
        tables,
        record_note,
    }
}

fn accepted_ids(acceptances: &[AcceptedReview]) -> Vec<String> {
    acceptances
        .iter()
        .map(|row| row.finding_id.clone())
        .collect()
}

/// The stored rows, back in Core's shape so the aggregate is Core's precedence rather than a
/// reimplementation of it here.
fn stored_findings(
    run: &StoredGateRun,
    operation_id: &str,
) -> Result<Vec<GateFinding>, ErrorEnvelopeDto> {
    run.findings
        .iter()
        .map(|finding| {
            let Some(rule_id) = finding.rule() else {
                return Err(unknown_rule(&finding.rule_id, operation_id));
            };
            Ok(GateFinding {
                id: finding.id.clone(),
                rule_id,
                state: finding.state,
                effective_severity: finding.effective_severity,
                summary: finding.summary.clone(),
                evidence_refs: finding.evidence_refs.clone(),
                remediation: finding.remediation.clone(),
            })
        })
        .collect()
}

/// One finding, with the acceptance beside it when a person recorded one.
fn finding_dto(finding: &StoredGateFinding, acceptances: &[AcceptedReview]) -> GateFindingRowDto {
    let acceptance = acceptances.iter().find(|row| row.finding_id == finding.id);
    GateFindingRowDto {
        id: finding.id.clone(),
        rule_id: finding.rule_id.clone(),
        state: finding.state.as_str().to_owned(),
        effective_severity: finding.effective_severity.as_str().to_owned(),
        summary: finding.summary.clone(),
        evidence_refs: finding.evidence_refs.clone(),
        remediation: finding.remediation.clone(),
        acceptable: finding.state == FindingState::Review && acceptance.is_none(),
        acceptance: acceptance.map(acceptance_dto),
    }
}

fn acceptance_dto(acceptance: &AcceptedReview) -> AcceptedReviewDto {
    AcceptedReviewDto {
        finding_id: acceptance.finding_id.clone(),
        actor: acceptance.actor.clone(),
        accepted_at: acceptance.accepted_at.clone(),
        reason: acceptance.reason.clone(),
        original_state: acceptance.original_state.as_str().to_owned(),
    }
}

/// Everything the findings do not carry: the artifact rows, the two budgets, the growth against the
/// baseline and the notes file.
///
/// Each row's `state` and `effective_severity` are read out of the findings rather than worked out again,
/// so a table cannot disagree with the rule that produced it (prompt §51–§52).
fn tables(
    findings: &[GateFinding],
    policy: &GatePolicy,
    target: &SnapshotFacts,
    growth: &GateGrowthFacts,
    notes: Option<&GateFileFact>,
) -> GateTablesDto {
    GateTablesDto {
        artifacts: artifact_rows(policy, target),
        budgets: budget_rows(findings, policy, target),
        growth: growth_rows(findings, policy, growth),
        notes: notes_dto(findings, policy, notes),
    }
}

/// Required first, then whatever else the build holds, so a `Required · Missing` row is never hidden by
/// the list's length (prompt §51).
fn artifact_rows(policy: &GatePolicy, target: &SnapshotFacts) -> Vec<GateArtifactRowDto> {
    let mut rows: Vec<GateArtifactRowDto> = policy
        .required_artifact_kinds
        .iter()
        .map(|kind| {
            let found = target
                .artifacts
                .iter()
                .find(|artifact| kind_word(artifact.kind) == *kind);
            GateArtifactRowDto {
                kind: kind.clone(),
                required: true,
                present: found.is_some(),
                sha256: found.and_then(|artifact| artifact.sha256.value().cloned()),
                byte_size: found.map(|artifact| artifact.byte_size),
            }
        })
        .collect();
    for artifact in &target.artifacts {
        if !rows.iter().any(|row| row.kind == kind_word(artifact.kind)) {
            rows.push(GateArtifactRowDto {
                kind: kind_word(artifact.kind).to_owned(),
                required: false,
                present: true,
                sha256: artifact.sha256.value().cloned(),
                byte_size: Some(artifact.byte_size),
            });
        }
    }
    rows
}

fn budget_rows(
    findings: &[GateFinding],
    policy: &GatePolicy,
    target: &SnapshotFacts,
) -> Vec<GateBudgetRowDto> {
    let memory = target.memory.clone().unwrap_or_default();
    [
        (
            "flash",
            "FLASH",
            GateRuleId::FlashBudget,
            policy.flash_budget,
            memory.nonvolatile,
        ),
        (
            "ram",
            "RAM",
            GateRuleId::RamBudget,
            policy.ram_budget,
            memory.runtime_ram,
        ),
    ]
    .into_iter()
    .map(|(side, label, rule, budget, fact)| {
        let (state, severity) = verdict(findings, rule);
        let fact = fact.unwrap_or_else(|| {
            GateBudgetFact::unknown("the snapshot carries no footprint for this side")
        });
        GateBudgetRowDto {
            side: side.to_owned(),
            label: label.to_owned(),
            rule_id: rule.as_str().to_owned(),
            state: state.as_str().to_owned(),
            effective_severity: severity.as_str().to_owned(),
            budget_bytes: budget,
            actual_bytes: fact.bytes,
            // Only a deterministic verdict has a margin, and which of the two it is follows the state Core
            // reached. The subtraction is arithmetic over two numbers Core already published; it never
            // decides which state the row shows, and an UNKNOWN row gets no invented number (§52).
            headroom_bytes: match state {
                FindingState::Pass => {
                    budget.and_then(|limit| fact.bytes.map(|bytes| limit - bytes))
                }
                _ => None,
            },
            over_bytes: match state {
                FindingState::Block => {
                    budget.and_then(|limit| fact.bytes.map(|bytes| bytes - limit))
                }
                _ => None,
            },
            exact: fact.exact,
            admissible: fact.admissible,
            basis: fact.basis.clone(),
            reason: fact.reason.clone(),
        }
    })
    .collect()
}

/// One row per thresholded side. A side the policy never set a threshold for has no row, because there
/// would be no rule to attribute a state to.
fn growth_rows(
    findings: &[GateFinding],
    policy: &GatePolicy,
    growth: &GateGrowthFacts,
) -> Vec<GateGrowthRowDto> {
    let (state, severity) = verdict(findings, GateRuleId::BaselineGrowth);
    [
        (
            "flash",
            "FLASH",
            policy.flash_growth_review_bytes,
            growth.nonvolatile.as_ref(),
        ),
        (
            "ram",
            "RAM",
            policy.ram_growth_review_bytes,
            growth.runtime_ram.as_ref(),
        ),
    ]
    .into_iter()
    .filter_map(|(side, label, threshold, change)| {
        let threshold = threshold?;
        let no_baseline = growth.baseline_snapshot_id.is_none();
        Some(GateGrowthRowDto {
            side: side.to_owned(),
            label: label.to_owned(),
            rule_id: GateRuleId::BaselineGrowth.as_str().to_owned(),
            // Both sides answer the single `diff.growth` rule, so both carry its state.
            state: state.as_str().to_owned(),
            effective_severity: severity.as_str().to_owned(),
            old_bytes: change.and_then(|row| row.base),
            new_bytes: change.and_then(|row| row.target),
            delta_bytes: change.and_then(|row| row.delta),
            threshold_bytes: Some(threshold),
            comparability: match change {
                Some(row) => row.comparability.label().to_owned(),
                None => if no_baseline {
                    "no baseline"
                } else {
                    "no delta"
                }
                .to_owned(),
            },
            reason: match change {
                Some(row) => row.reason.clone(),
                None => Some("no baseline snapshot was supplied for this run".to_owned()),
            },
        })
    })
    .collect()
}

fn notes_dto(
    findings: &[GateFinding],
    policy: &GatePolicy,
    notes: Option<&GateFileFact>,
) -> GateNotesDto {
    let (state, severity) = verdict(findings, GateRuleId::ReleaseNotes);
    let path = policy
        .require_release_notes
        .then(|| policy.release_notes_path.clone());
    let (present, sha256, reason) = match notes.map(|fact| &fact.status) {
        None => (
            None,
            None,
            Some(
                "no observation was recorded: the Gate ran without a project folder to look in"
                    .to_owned(),
            ),
        ),
        Some(GateFileStatus::Present { sha256 }) => (
            Some(true),
            sha256.clone(),
            sha256.is_none().then(|| {
                "the file is there but was not read, so no digest was recorded".to_owned()
            }),
        ),
        Some(GateFileStatus::Missing) => (
            Some(false),
            None,
            Some("the configured path holds nothing".to_owned()),
        ),
        Some(GateFileStatus::Unreadable { reason }) => (Some(true), None, Some(reason.clone())),
    };
    GateNotesDto {
        required: policy.require_release_notes,
        state: state.as_str().to_owned(),
        effective_severity: severity.as_str().to_owned(),
        relative_path: path,
        present,
        sha256,
        reason,
    }
}

/// The finding Core reached for one rule. A run missing that rule's finding is a construction fault, not
/// an empty answer, so the row degrades to `UNKNOWN` rather than to nothing.
fn verdict(findings: &[GateFinding], rule: GateRuleId) -> (FindingState, EffectiveSeverity) {
    findings
        .iter()
        .find(|finding| finding.rule_id == rule)
        .map_or(
            (FindingState::Unknown, EffectiveSeverity::Review),
            |finding| (finding.state, finding.effective_severity),
        )
}

/// The three phrasings prompt §50 allows, and nothing beyond them.
fn git_dto(git: &GateGitFacts) -> GateGitDto {
    let summary = if !git.available {
        "No workspace facts were observed"
    } else if git.dirty.value() == Some(&true) {
        "Workspace dirty"
    } else {
        "Workspace HEAD"
    };
    GateGitDto {
        available: git.available,
        head_commit: git.head_commit.value().cloned(),
        exact_tag: git.exact_tag.value().cloned(),
        // `None` means the probe did not learn this, which is not the same as clean.
        dirty: git.dirty.value().copied(),
        summary: summary.to_owned(),
        reason: [
            git.head_commit.reason_if_unknown(),
            git.exact_tag.reason_if_unknown(),
            git.dirty.reason_if_unknown(),
        ]
        .into_iter()
        .flatten()
        .map(str::to_owned)
        .next(),
    }
}

/// A stored build, turned into the facts the Gate reads.
///
/// The stored footprint keeps no reason text for a partial or unknown total, so `reason` stays `None`:
/// Core's rules carry their own words for those cases, which name what is missing without inventing a
/// reason the database never recorded. The evidence pointer is different — it names a row the build
/// really holds, so the caller reads it out of the database and hands it in beside the totals.
///
/// Public so the integration tests can hold it against [`SnapshotFacts::from_snapshot`], the CLI's
/// path: a Gate run's identity is a hash of these facts, so the two ways of assembling them have to
/// agree.
///
/// [`SnapshotFacts::from_snapshot`]: firmwaresight_project::evidence::SnapshotFacts::from_snapshot
#[must_use]
pub fn snapshot_facts(
    stored: &GateSnapshotFacts,
    footprint_evidence: &[Option<String>; 2],
) -> SnapshotFacts {
    SnapshotFacts::from_stored(
        stored.snapshot_id.clone(),
        stored
            .artifacts
            .iter()
            .map(|row| GateArtifactFact {
                kind: kind_of(&row.kind),
                sha256: Fact::known(row.sha256.clone()),
                byte_size: row.byte_size,
            })
            .collect(),
        Some(GateMemoryFacts {
            nonvolatile: Some(budget_of(
                &stored.footprint.nonvolatile,
                stored.footprint.admissible_hard_block,
                stored.footprint.weakest_basis.clone(),
                footprint_evidence[0].as_deref(),
            )),
            runtime_ram: Some(budget_of(
                &stored.footprint.runtime_ram,
                stored.footprint.admissible_hard_block,
                stored.footprint.weakest_basis.clone(),
                footprint_evidence[1].as_deref(),
            )),
        }),
        unknown_evidence_from_ids(
            &stored.gaps.sample_ids,
            stored.gaps.count,
            UNKNOWN_EVIDENCE_SAMPLE,
        ),
    )
}

fn budget_of(
    row: &StoredBudget,
    admissible: bool,
    basis: Option<String>,
    evidence_id: Option<&str>,
) -> GateBudgetFact {
    let state = match row.state.as_str() {
        "exact" => BudgetState::Exact,
        "partial" => BudgetState::Partial,
        _ => BudgetState::Unknown,
    };
    // An unknown total has no number, whatever a row holds in the bytes column: the state is the fact.
    let bytes = if state == BudgetState::Unknown {
        None
    } else {
        row.bytes
    };
    budget_fact(
        state,
        bytes,
        admissible,
        basis,
        None,
        evidence_id.map(|id| format!("evidence:{id}")),
    )
}

fn kind_of(stored: &str) -> ArtifactKind {
    match stored {
        "Elf" => ArtifactKind::Elf,
        "Map" => ArtifactKind::Map,
        "Bin" => ArtifactKind::Bin,
        "IntelHex" => ArtifactKind::IntelHex,
        _ => ArtifactKind::Unknown,
    }
}

/// The v1 requirement vocabulary the config speaks, from the kind Core names.
fn kind_word(kind: ArtifactKind) -> &'static str {
    match kind {
        ArtifactKind::Elf => "elf",
        ArtifactKind::Map => "map",
        ArtifactKind::Bin => "bin",
        ArtifactKind::IntelHex => "hex",
        ArtifactKind::Unknown => "unknown",
    }
}

/// The loaded project, reduced to the facts a UI may hold. The config path appears here as a *name*.
fn project_dto(loaded: &LoadedProject) -> ProjectContextDto {
    ProjectContextDto {
        project_name: loaded.project_name().to_owned(),
        config_file_name: loaded
            .path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| CONFIG_FILE_NAME.to_owned()),
        config_schema_version: loaded.config.schema_version,
        policy: policy_dto(&loaded.policy, loaded.project_name()),
        policy_sha256: loaded.policy_sha256.clone(),
        warnings: loaded.warnings.clone(),
        unknown_keys: loaded.unknown_keys.clone(),
    }
}

fn policy_dto(policy: &GatePolicy, project_name: &str) -> ProjectPolicyDto {
    ProjectPolicyDto {
        project_name: project_name.to_owned(),
        required_artifact_kinds: Some(policy.required_artifact_kinds.clone()),
        flash_budget: policy.flash_budget,
        ram_budget: policy.ram_budget,
        // A policy read from a config, or the default a run used, is resolved: what the reader sees is
        // what the rules were judged with, so nothing in it is left unset.
        require_clean_git: Some(policy.require_clean_git),
        require_release_notes: Some(policy.require_release_notes),
        release_notes_path: Some(policy.release_notes_path.clone()),
        // `git_tag` is the only version source major 1 has, so its presence says whether a `[version]`
        // section applies at all. An empty pattern means "any tag" and is sent as `null`.
        version_source: policy.version.as_ref().map(|_| "git_tag".to_owned()),
        version_pattern: policy
            .version
            .as_ref()
            .map(|version| version.pattern.clone())
            .filter(|pattern| !pattern.is_empty()),
        expected_version: policy
            .version
            .as_ref()
            .and_then(|version| version.expected_version.clone()),
        expected_commit: policy.expected_commit.clone(),
        flash_growth_review_bytes: policy.flash_growth_review_bytes,
        ram_growth_review_bytes: policy.ram_growth_review_bytes,
        unknown_evidence_review_count: policy.unknown_evidence_review_count,
        on_unknown: UnknownPolicyDto {
            git_clean: Some(disposition(policy.on_unknown.git_clean)),
            commit_matches_release: Some(disposition(policy.on_unknown.commit_matches_release)),
            version_match: Some(disposition(policy.on_unknown.version_match)),
            flash_budget: Some(disposition(policy.on_unknown.flash_budget)),
            ram_budget: Some(disposition(policy.on_unknown.ram_budget)),
            baseline_growth: Some(disposition(policy.on_unknown.baseline_growth)),
            release_notes: Some(disposition(policy.on_unknown.release_notes)),
        },
    }
}

const fn disposition(value: UnknownDisposition) -> UnknownDispositionDto {
    match value {
        UnknownDisposition::Review => UnknownDispositionDto::Review,
        UnknownDisposition::Block => UnknownDispositionDto::Block,
    }
}

const fn unknown_disposition(value: UnknownDispositionDto) -> UnknownDisposition {
    match value {
        UnknownDispositionDto::Review => UnknownDisposition::Review,
        UnknownDispositionDto::Block => UnknownDisposition::Block,
    }
}

/// The edited form back into the policy Core evaluates.
///
/// Every unset field takes the value `GatePolicy::default()` carries, which is the default
/// `04_TECH/08:60-62` documents and that `firmwaresight.toml` itself does not restate. That is why the
/// front end holds no copy of a default: an empty box on the screen and an absent key in the config mean
/// the same thing because this function resolves both, and resolves them only here (prompt §41, §42).
///
/// Validation happens on the way in, in the adapter that owns the rules: an absolute notes path or an
/// unknown `on_unknown` word is refused there rather than reaching a Gate run.
fn gate_policy(dto: &ProjectPolicyDto) -> GatePolicy {
    let base = GatePolicy::default();
    GatePolicy {
        require_clean_git: dto.require_clean_git.unwrap_or(base.require_clean_git),
        expected_commit: dto
            .expected_commit
            .clone()
            .filter(|value| !value.trim().is_empty()),
        version: dto.version_source.as_ref().map(|_| VersionPolicy {
            pattern: dto.version_pattern.clone().unwrap_or_default(),
            expected_version: dto
                .expected_version
                .clone()
                .filter(|value| !value.trim().is_empty()),
        }),
        required_artifact_kinds: dto
            .required_artifact_kinds
            .clone()
            .filter(|kinds| !kinds.is_empty())
            .unwrap_or(base.required_artifact_kinds),
        flash_budget: dto.flash_budget.or(base.flash_budget),
        ram_budget: dto.ram_budget.or(base.ram_budget),
        flash_growth_review_bytes: dto.flash_growth_review_bytes,
        ram_growth_review_bytes: dto.ram_growth_review_bytes,
        require_release_notes: dto
            .require_release_notes
            .unwrap_or(base.require_release_notes),
        release_notes_path: dto
            .release_notes_path
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(base.release_notes_path),
        unknown_evidence_review_count: dto.unknown_evidence_review_count,
        on_unknown: UnknownPolicy {
            git_clean: disposition_or(dto.on_unknown.git_clean, base.on_unknown.git_clean),
            commit_matches_release: disposition_or(
                dto.on_unknown.commit_matches_release,
                base.on_unknown.commit_matches_release,
            ),
            version_match: disposition_or(
                dto.on_unknown.version_match,
                base.on_unknown.version_match,
            ),
            flash_budget: disposition_or(dto.on_unknown.flash_budget, base.on_unknown.flash_budget),
            ram_budget: disposition_or(dto.on_unknown.ram_budget, base.on_unknown.ram_budget),
            baseline_growth: disposition_or(
                dto.on_unknown.baseline_growth,
                base.on_unknown.baseline_growth,
            ),
            release_notes: disposition_or(
                dto.on_unknown.release_notes,
                base.on_unknown.release_notes,
            ),
        },
    }
}

const fn disposition_or(
    value: Option<UnknownDispositionDto>,
    default: UnknownDisposition,
) -> UnknownDisposition {
    match value {
        Some(inner) => unknown_disposition(inner),
        None => default,
    }
}

/// The paths a project error can name, so the envelope hands the WebView a file name instead of a
/// directory listing (`AGENTS.md` 7).
fn hidden_of(path: &Path) -> Vec<&Path> {
    vec![path, path.parent().unwrap_or(path)]
}

fn short_hash(value: &str) -> String {
    value.chars().take(8).collect()
}

fn snapshot_missing(_snapshot_id: &str, operation_id: &str) -> ErrorEnvelopeDto {
    ErrorEnvelopeDto {
        code: "ERR-STORAGE-4005".to_owned(),
        message: "No stored build matches that baseline.".to_owned(),
        operation_id: operation_id.to_owned(),
        details: None,
        remediation: Some(
            "Pick the baseline again from the builds this application analyzed.".to_owned(),
        ),
    }
}

pub(crate) fn stored_run_missing(_run_id: &str, operation_id: &str) -> ErrorEnvelopeDto {
    ErrorEnvelopeDto {
        code: "ERR-STORAGE-4005".to_owned(),
        message: "No Gate run with that id is stored.".to_owned(),
        operation_id: operation_id.to_owned(),
        details: Some("gate_runs holds every run this application has computed".to_owned()),
        remediation: Some("Run the Gate again, or reload the Release page.".to_owned()),
    }
}

fn acceptance_finding_missing(_finding_id: &str, operation_id: &str) -> ErrorEnvelopeDto {
    ErrorEnvelopeDto {
        code: "ERR-STORAGE-4007".to_owned(),
        message: "That finding is not part of the stored run.".to_owned(),
        operation_id: operation_id.to_owned(),
        details: None,
        remediation: Some("Reload the run and accept one of the findings it lists.".to_owned()),
    }
}

fn unknown_rule(rule_id: &str, operation_id: &str) -> ErrorEnvelopeDto {
    ErrorEnvelopeDto {
        code: "ERR-INTERNAL-9003".to_owned(),
        message: "A stored Gate finding names a rule this build does not know.".to_owned(),
        operation_id: operation_id.to_owned(),
        details: Some(rule_id.to_owned()),
        remediation: Some(
            "This database was written by a newer FirmwareSight. Read it with the build that wrote it."
                .to_owned(),
        ),
    }
}

/// Each acceptance refusal is a different answer for the release owner, so each keeps its own code and
/// states what can still be done.
fn envelope_from_acceptance(err: &AcceptReviewError, operation_id: &str) -> ErrorEnvelopeDto {
    let (message, remediation) = match err {
        AcceptReviewError::FindingNotFound { .. } => (
            "That finding is not part of the stored run.".to_owned(),
            "Reload the run and accept one of the findings it lists.".to_owned(),
        ),
        AcceptReviewError::NotReview { state, .. } => (
            format!("Only a REVIEW finding can be accepted; this finding is {state}."),
            "A BLOCK needs the condition fixed, an UNKNOWN needs evidence, and a PASS needs nothing. \
             Accepting is not how any of those is disposed of."
                .to_owned(),
        ),
        AcceptReviewError::AlreadyAccepted { actor, .. } => (
            format!("This review was already accepted by {actor}."),
            "The first acceptance stands. It cannot be edited or deleted.".to_owned(),
        ),
        AcceptReviewError::EmptyActor => (
            "An acceptance has to name who accepted it.".to_owned(),
            "Enter the person responsible for this release decision.".to_owned(),
        ),
        AcceptReviewError::EmptyReason => (
            "An acceptance has to state why it was accepted.".to_owned(),
            "Say what made the review acceptable; the reason is part of the record.".to_owned(),
        ),
        AcceptReviewError::Storage(source) => {
            return envelope_from_storage(source, operation_id);
        }
    };
    ErrorEnvelopeDto {
        code: err.stable_code().to_owned(),
        message,
        operation_id: operation_id.to_owned(),
        details: None,
        remediation: Some(remediation),
    }
}

/// Open the native save dialog for a project that has no config yet, confirming before an existing file
/// would be replaced. `None` is Cancel, which is a decision rather than a failure (prompt §41).
fn choose_config_path(app: &AppHandle) -> Option<PathBuf> {
    let picked = app
        .dialog()
        .file()
        .set_title("Save the project policy as firmwaresight.toml")
        .add_filter("TOML", &["toml"])
        .set_file_name(CONFIG_FILE_NAME)
        .blocking_save_file()
        .and_then(|picked| picked.into_path().ok())?;
    if !picked.exists() {
        return Some(picked);
    }
    let replace = app
        .dialog()
        .message(format!(
            "{} already exists. Replacing it overwrites the policy file you have.",
            service::display_name(&picked)
        ))
        .title("Replace existing file?")
        .buttons(MessageDialogButtons::OkCancelCustom(
            "Replace it".to_owned(),
            "Keep mine".to_owned(),
        ))
        .blocking_show();
    if replace { Some(picked) } else { None }
}

/// Choose the project's `firmwaresight.toml`. Resolves to `None` when the user cancels, which keeps the
/// page on the project it already had (prompt §40).
#[tauri::command]
pub(crate) async fn open_project_config(
    app: AppHandle,
    state: State<'_, Arc<Session>>,
) -> Result<Option<ProjectContextDto>, ErrorEnvelopeDto> {
    let session = Arc::clone(&state);
    let operation_id = next_operation_id();

    let picked = tauri::async_runtime::spawn_blocking(move || {
        pick_path(&app, "Choose the project's firmwaresight.toml")
    })
    .await
    .map_err(task_join_error)?;

    session.load_project_from(picked, &operation_id)
}

/// Save the edited policy. With a project loaded it goes back to that project's own file; without one the
/// shell opens a save dialog and writes it there.
///
/// The payload is a structured policy — never a path, never a document (prompt §41, §42).
#[tauri::command]
pub(crate) async fn save_project_policy(
    app: AppHandle,
    policy: ProjectPolicyDto,
    state: State<'_, Arc<Session>>,
) -> Result<Option<ProjectContextDto>, ErrorEnvelopeDto> {
    let session = Arc::clone(&state);
    let operation_id = next_operation_id();

    if session.has_project() {
        return tauri::async_runtime::spawn_blocking(move || {
            session.save_policy_in_project(&policy, &operation_id)
        })
        .await
        .map_err(task_join_error)?;
    }

    let picked = tauri::async_runtime::spawn_blocking(move || choose_config_path(&app))
        .await
        .map_err(task_join_error)?;
    tauri::async_runtime::spawn_blocking(move || {
        session.save_policy_as_new(picked, &policy, &operation_id)
    })
    .await
    .map_err(task_join_error)?
}

/// Judge one stored build, or two with a baseline, and store the answer.
#[tauri::command]
pub(crate) async fn run_release_gate(
    request: GateRunRequestDto,
    state: State<'_, Arc<Session>>,
) -> Result<GateRunDto, ErrorEnvelopeDto> {
    let session = Arc::clone(&state);
    let operation_id = next_operation_id();

    tauri::async_runtime::spawn_blocking(move || session.run_release_gate(&request, &operation_id))
        .await
        .map_err(task_join_error)?
}

/// Accept one review, naming the person and the reason.
#[tauri::command]
pub(crate) async fn accept_review(
    request: AcceptReviewRequestDto,
    state: State<'_, Arc<Session>>,
) -> Result<AcceptReviewOutcomeDto, ErrorEnvelopeDto> {
    let session = Arc::clone(&state);
    let operation_id = next_operation_id();

    tauri::async_runtime::spawn_blocking(move || {
        session.accept_gate_review(&request, &operation_id)
    })
    .await
    .map_err(task_join_error)?
}

/// Read a run back from history, so a page switch or a restart does not lose the record a person just
/// made (prompt §45, §49).
#[tauri::command]
pub(crate) async fn get_gate_run(
    run_id: String,
    state: State<'_, Arc<Session>>,
) -> Result<Option<GateRunDto>, ErrorEnvelopeDto> {
    let session = Arc::clone(&state);
    let operation_id = next_operation_id();

    tauri::async_runtime::spawn_blocking(move || session.gate_run(&run_id, &operation_id))
        .await
        .map_err(task_join_error)?
}
