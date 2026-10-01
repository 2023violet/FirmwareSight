//! The contract tests prompt §53 requires for a Release Bundle's portable content: the
//! `release-manifest:1` document, the `SHA256SUMS` text and `release-report.html`.
//!
//! The manifest is validated against the *published* schema (`schemas/release-manifest.schema.json`), which
//! P4 reuses at major 1 rather than re-issues (§19), through `firmwaresight_report::schema_check` — the
//! dependency-free subset those schemas actually use. The rejecting cases are part of the test: a contract
//! test that can only ever pass proves nothing (`05_ENGINEERING/02_TEST_STRATEGY.md`), and the refusals in
//! `ReleaseManifestDto::from_parts` are the reason a bundle cannot index bytes its own fingerprint never
//! named.
//!
//! The HTML is checked as a file a stranger opens, because that is US-004: self-contained, byte-stable,
//! free of host paths, free of any signing claim (§62), and with the four identities spelled apart (§43).

use std::path::{Path, PathBuf};

use firmwaresight_core::domain::diff::{ChangeKind, Contributor};
use firmwaresight_core::domain::gate::{EffectiveSeverity, GateStateCounts};
use firmwaresight_core::domain::identity::{ArtifactKind, Sha256};
use firmwaresight_core::domain::release::{
    ACCEPTED_REVIEWS_DOC_NAME, ANALYSIS_DOC_NAME, ARTIFACTS_DIR, DIFF_DOC_NAME,
    GATE_RESULTS_DOC_NAME, MANIFEST_DOC_NAME, RELEASE_NOTES_NAME, REPORT_DOC_NAME, ReleaseArtifact,
    ReleaseId, ReleaseModel, ReleaseNotesDigest, ReleaseVersion, ReleaseVersionSource,
    SHA256SUMS_NAME, SchemaMajors, WorkspaceGit,
};
use firmwaresight_report::analysis::{AnalysisDocumentDto, PortableArtifactDto, SnapshotDto};
use firmwaresight_report::diff::{
    ByteChangeDto, ChangeCountsDto, DIFF_SCHEMA_ID, DiffArtifactDto, DiffResultDto, DiffSideDto,
    DiffSideMemoryDto, DiffWarningDto, MemoryDiffDto, ObjectChangesDto, SideBudgetDto,
    SideEvidenceDto, UnchangedCountsDto,
};
use firmwaresight_report::dto::{
    BudgetDto, CapabilitiesDto, ContributionDto, EvidenceDto, MemoryDto,
};
use firmwaresight_report::gate::{
    AcceptanceDto, AcceptedReviewsDto, AcceptedReviewsExtensionsDto, GateExtensionsDto,
    GateFindingDto, GateResultsDto,
};
use firmwaresight_report::release::{ManifestError, ManifestFileDto, ReleaseManifestDto};
use firmwaresight_report::release_render::{
    CompareSummary, ReleaseBundleReport, render_html, render_json, render_sums,
};
use firmwaresight_report::render::to_json;
use firmwaresight_report::schema_check::{self, Failure};

const ELF_HEX: &str = "1fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
const MAP_HEX: &str = "2fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
const NOTES_HEX: &str = "3fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
const POLICY_HEX: &str = "4fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
const RUN_HEX: &str = "6fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
const RELEASE_HEX: &str = "7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
const HEAD_COMMIT: &str = "8d2f1c4a5b6e7f8091a2b3c4d5e6f708192a3b4c";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|parent| parent.parent())
        .expect("crates/firmwaresight-report lives at <root>/crates/<name>")
        .to_path_buf()
}

fn manifest_schema() -> serde_json::Value {
    let path = repo_root()
        .join("schemas")
        .join("release-manifest.schema.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} must be readable: {error}", path.display()));
    serde_json::from_str(&text)
        .unwrap_or_else(|error| panic!("release-manifest.schema.json must parse: {error}"))
}

fn validate(doc: &serde_json::Value) -> Vec<Failure> {
    schema_check::validate(&manifest_schema(), doc)
}

/// A distinct 64-character lowercase digest per fixture file, so a test can tell one entry from another
/// without carrying 64 characters of noise in every assertion.
fn digest(marker: char) -> String {
    let mut text = marker.to_string();
    text.push_str(&"f".repeat(63));
    text
}

// --------------------------------------------------------------------------- fixtures

fn artifact(kind: ArtifactKind, name: &str, hex: &str, size: u64) -> ReleaseArtifact {
    ReleaseArtifact {
        kind,
        file_name: name.to_owned(),
        sha256: Sha256::parse(hex).expect("fixture digest is lowercase hex"),
        byte_size: size,
    }
}

/// A validated model over one ELF and one MAP, with notes, a baseline and observed Git facts.
fn model() -> ReleaseModel {
    ReleaseModel {
        snapshot_id: "snap-0001".to_owned(),
        baseline_snapshot_id: Some("snap-0000".to_owned()),
        gate_run_id: format!("gate-{RUN_HEX}"),
        disposition: EffectiveSeverity::Pass,
        version: ReleaseVersion::parse("1.4.2").expect("fixture version"),
        version_source: ReleaseVersionSource::GitTagCapture,
        acceptances_sha256: Sha256::parse(&digest('5')).expect("fixture digest"),
        artifacts: vec![
            artifact(ArtifactKind::Elf, "firmware.elf", ELF_HEX, 148_240),
            artifact(ArtifactKind::Map, "firmware.map", MAP_HEX, 51_208),
        ],
        release_notes: Some(ReleaseNotesDigest {
            bundle_name: RELEASE_NOTES_NAME.to_owned(),
            sha256: Sha256::parse(NOTES_HEX).expect("fixture digest"),
        }),
        workspace_git: WorkspaceGit {
            head_commit: Some(HEAD_COMMIT.to_owned()),
            exact_tag: Some("v1.4.2".to_owned()),
            dirty: Some(false),
        },
        schema_majors: SchemaMajors::CURRENT,
        fwsight_version: "0.6.0".to_owned(),
    }
    .validated()
    .expect("the fixture is a release")
}

/// The complete file list a manifest indexes: the fingerprint's artifacts plus every generated document,
/// each with the digest of the bytes actually written, including `release-report.html` and `SHA256SUMS`.
fn staged(notes: bool, baseline: bool) -> Vec<ManifestFileDto> {
    let mut files = vec![
        ManifestFileDto::new(ANALYSIS_DOC_NAME, digest('a'), 40_112),
        ManifestFileDto::new(GATE_RESULTS_DOC_NAME, digest('b'), 9_804),
        ManifestFileDto::new(ACCEPTED_REVIEWS_DOC_NAME, digest('c'), 512),
        ManifestFileDto::new(REPORT_DOC_NAME, digest('d'), 21_032),
        ManifestFileDto::new(SHA256SUMS_NAME, digest('e'), 1_024),
    ];
    if baseline {
        files.push(ManifestFileDto::new(DIFF_DOC_NAME, digest('9'), 33_408));
    }
    if notes {
        files.push(ManifestFileDto::new(RELEASE_NOTES_NAME, NOTES_HEX, 1_840));
    }
    files.extend(
        model()
            .artifacts
            .iter()
            .map(ManifestFileDto::shipped_artifact),
    );
    files
}

fn release_id() -> String {
    ReleaseModel::release_id_from(RELEASE_HEX)
}

/// The files that already exist while `release-report.html` is being composed (§41): everything except the
/// report itself and the two index files written after it. The report may print a digest for these and
/// only a name for the others.
fn staged_before_report(notes: bool, baseline: bool) -> Vec<ManifestFileDto> {
    let mut files = staged(notes, baseline);
    files.retain(|entry| entry.path != REPORT_DOC_NAME && entry.path != SHA256SUMS_NAME);
    files
}

fn manifest_for(model: &ReleaseModel, notes: bool, baseline: bool) -> ReleaseManifestDto {
    ReleaseManifestDto::from_parts(model, &release_id(), &staged(notes, baseline), POLICY_HEX)
        .expect("the fixture bundle assembles a manifest")
}

fn manifest() -> ReleaseManifestDto {
    manifest_for(&model(), true, true)
}

fn analysis() -> AnalysisDocumentDto {
    AnalysisDocumentDto {
        schema: firmwaresight_report::ANALYSIS_SCHEMA_ID,
        schema_version: 1,
        snapshot: SnapshotDto {
            snapshot_id: "snap-0001".to_owned(),
            fwsight_version: "0.6.0".to_owned(),
            normalization_version: "normalized-1",
        },
        artifacts: vec![PortableArtifactDto {
            file_name: "firmware.elf".to_owned(),
            kind: "elf",
            sha256: Sha256::parse(ELF_HEX).expect("fixture digest"),
            byte_size: 148_240,
            parser_id: "elf-legacy-1".to_owned(),
            architecture: "ARMv7-M".to_owned(),
            bitness: "32",
            endianness: "little",
            entry_point: Some("0x8000100".to_owned()),
            entry_point_unknown_reason: None,
            build_id: None,
            build_id_unknown_reason: Some("no NT_GNU_BUILD_ID note is present".to_owned()),
        }],
        memory: MemoryDto {
            accounting_rule: "load-image-and-nobits/1",
            layout_source: "elf-program-headers",
            weakest_evidence_basis: Some("section-name-heuristic"),
            admissible_for_hard_block: true,
            nonvolatile_image_footprint: BudgetDto {
                state: "partial",
                classification: "derived",
                bytes: Some(146_000),
                unattributed: vec![".itcm".to_owned()],
                reason: Some("one section could not be classified".to_owned()),
            },
            runtime_ram_footprint: BudgetDto {
                state: "exact",
                classification: "observed",
                bytes: Some(24_576),
                unattributed: Vec::new(),
                reason: None,
            },
            contributions: vec![ContributionDto {
                section_locator: "index 3".to_owned(),
                section_name: Some(".text".to_owned()),
                nonvolatile_bytes: Some(120_000),
                runtime_ram_bytes: Some(0),
                evidence_basis: "elf-program-headers",
                classification: "observed",
                accounting_rule: "load-image-and-nobits/1",
                dual_accounted: false,
            }],
            excluded_metadata_bytes: 1_024,
            dual_accounted_sections: Vec::new(),
        },
        sections: Vec::new(),
        symbols: Vec::new(),
        capabilities: CapabilitiesDto {
            elf: "supported",
            sections: "available",
            symbols: "available",
            debug_info: "unavailable",
            map: "provided",
            object_attribution: "unavailable",
            git: "available",
        },
        evidence: vec![EvidenceDto {
            id: "ev-3".to_owned(),
            field: "artifact.build_id".to_owned(),
            classification: "unknown",
            source_type: "elf.note",
            source_locator: "elf.note".to_owned(),
            value: "<script>alert(\"x\")</script>".to_owned(),
            rule: "no note found & nowhere".to_owned(),
            confidence: None,
        }],
    }
}

fn side(snapshot: &str, name: &str, sha: &str, bytes: u64) -> DiffSideDto {
    let budget = |bytes: u64| SideBudgetDto {
        state: "exact",
        classification: "observed",
        bytes: Some(bytes),
    };
    DiffSideDto {
        snapshot_id: snapshot.to_owned(),
        artifact: DiffArtifactDto {
            file_name: name.to_owned(),
            sha256: sha.to_owned(),
            byte_size: bytes,
        },
        memory: DiffSideMemoryDto {
            footprint_row_present: true,
            nonvolatile: budget(bytes),
            runtime_ram: budget(bytes / 6),
            excluded_metadata_bytes: Some(512),
            evidence: SideEvidenceDto {
                map_backed: true,
                layout_source: "map+elf-load".to_owned(),
                weakest_evidence_basis: None,
            },
        },
    }
}

fn diff() -> DiffResultDto {
    DiffResultDto {
        schema: DIFF_SCHEMA_ID,
        schema_version: 1,
        base: side("snap-0000", "firmware-old.elf", MAP_HEX, 140_000),
        target: side("snap-0001", "firmware.elf", ELF_HEX, 148_240),
        memory: MemoryDiffDto {
            nonvolatile: ByteChangeDto {
                base: Some(140_000),
                target: Some(148_240),
                delta: Some(8_240),
                comparability: "exact",
                reason: None,
            },
            runtime_ram: ByteChangeDto {
                base: None,
                target: Some(24_576),
                delta: None,
                comparability: "unknown",
                reason: Some("the base build has no stored footprint row".to_owned()),
            },
            comparability: "exact",
        },
        counts: ChangeCountsDto {
            sections_added: 1,
            sections_removed: 0,
            sections_changed: 4,
            sections_ambiguous: 0,
            symbols_added: 12,
            symbols_removed: 3,
            symbols_changed: 40,
            symbols_ambiguous: 1,
        },
        unchanged: UnchangedCountsDto {
            sections: 9,
            symbols: 400,
        },
        section_changes: Vec::new(),
        symbol_changes: Vec::new(),
        object_changes: ObjectChangesDto {
            available: false,
            reason: "no object-file attribution without debug info".to_owned(),
        },
        warnings: vec![DiffWarningDto {
            code: "DIFF-0007".to_owned(),
            message: "one row could not be paired & was left out".to_owned(),
        }],
    }
}

fn finding(
    rule: &str,
    state: &'static str,
    severity: &'static str,
    summary: &str,
) -> GateFindingDto {
    GateFindingDto {
        id: format!("gate-{RUN_HEX}#{rule}"),
        rule_id: rule.to_owned(),
        state,
        effective_severity: severity,
        summary: summary.to_owned(),
        evidence_refs: vec!["ev-1".to_owned(), "section .text".to_owned()],
        remediation: Some("raise the budget or shrink .text".to_owned()),
    }
}

fn gate() -> GateResultsDto {
    GateResultsDto {
        schema_version: 1,
        run_id: format!("gate-{RUN_HEX}"),
        snapshot_id: "snap-0001".to_owned(),
        findings: vec![
            finding(
                "MEMORY_GROWTH",
                "REVIEW",
                "REVIEW",
                "nonvolatile image grew by 8240 bytes over the budget & rule",
            ),
            finding(
                "EVIDENCE_COMPLETE",
                "UNKNOWN",
                "REVIEW",
                "one fact rests on no recorded evidence basis",
            ),
            finding(
                "VERSION_DECLARED",
                "PASS",
                "PASS",
                "the version the Gate judged matches the tag the release resolved from",
            ),
        ],
        extensions: GateExtensionsDto {
            policy_sha256: POLICY_HEX.to_owned(),
            project_config_schema_version: 1,
            baseline_snapshot_id: Some("snap-0000".to_owned()),
            overall_effective_severity: "REVIEW",
        },
    }
}

fn reviews(with_acceptance: bool) -> AcceptedReviewsDto {
    let acceptances = if with_acceptance {
        vec![
            AcceptanceDto {
                finding_id: format!("gate-{RUN_HEX}#MEMORY_GROWTH"),
                actor: "release-owner".to_owned(),
                accepted_at: "2026-09-30T08:12:44Z".to_owned(),
                reason: "the growth is the new <b>modem</b> driver, reviewed by hand".to_owned(),
                original_state: "REVIEW",
            },
            AcceptanceDto {
                finding_id: format!("gate-{RUN_HEX}#EVIDENCE_COMPLETE"),
                actor: "release-owner".to_owned(),
                accepted_at: "2026-09-30T08:13:02Z".to_owned(),
                reason: "the gap is a toolchain note, not a build fact".to_owned(),
                original_state: "REVIEW",
            },
        ]
    } else {
        Vec::new()
    };
    AcceptedReviewsDto {
        schema_version: 1,
        run_id: format!("gate-{RUN_HEX}"),
        acceptances,
        extensions: AcceptedReviewsExtensionsDto {
            overall_effective_severity: "PASS",
        },
    }
}

/// Growth rows, from Core's own selectors in the real builder. The keys carry markup on purpose: a
/// section name is bytes from a file, and a report that did not escape it would be an XSS in a bundle.
fn growth() -> Vec<Contributor> {
    vec![
        Contributor {
            key: ".text".to_owned(),
            change: ChangeKind::Changed,
            delta: Some(6_000),
            bytes: Some(120_000),
        },
        Contributor {
            key: "<img onerror=x>".to_owned(),
            change: ChangeKind::Added,
            delta: None,
            bytes: Some(24),
        },
        Contributor {
            key: ".data".to_owned(),
            change: ChangeKind::Changed,
            delta: Some(-16),
            bytes: Some(4_096),
        },
    ]
}

/// The whole bundle's renderable content, owned so the report can borrow all of it at once.
///
/// Two file lists, because §41 gives a bundle two moments: the report is written before `SHA256SUMS` and
/// `release-manifest.json`, so it can print a digest for `pre_report` and only a *name* for the rest. The
/// manifest is derived rather than stored, for the same reason — a fixture that carried it as a field would
/// let the report borrow a hash of bytes it is supposed to precede.
struct Bundle {
    model: ReleaseModel,
    release_id: String,
    files: Vec<ManifestFileDto>,
    pre_report: Vec<ManifestFileDto>,
    analysis: AnalysisDocumentDto,
    diff: Option<DiffResultDto>,
    sections: Vec<Contributor>,
    symbols: Vec<Contributor>,
    gate: GateResultsDto,
    reviews: AcceptedReviewsDto,
}

impl Bundle {
    fn full() -> Self {
        Self::build(model(), true, true, true)
    }

    /// The same release with no baseline: no `diff.json`, and nothing to rank.
    fn without_baseline() -> Self {
        let model = ReleaseModel {
            baseline_snapshot_id: None,
            ..model()
        }
        .validated()
        .expect("a release with no comparison is still a release");
        Self::build(model, true, false, true)
    }

    /// The same release with nobody having accepted a review.
    fn without_reviews() -> Self {
        Self::build(model(), true, true, false)
    }

    fn build(model: ReleaseModel, notes: bool, baseline: bool, accepted: bool) -> Self {
        let rows = growth();
        Self {
            model,
            release_id: release_id(),
            files: staged(notes, baseline),
            pre_report: staged_before_report(notes, baseline),
            analysis: analysis(),
            diff: baseline.then(diff),
            sections: rows.clone(),
            symbols: rows,
            gate: gate(),
            reviews: reviews(accepted),
        }
    }

    /// The manifest these staged files produce: written last, so it is derived here rather than held.
    fn manifest(&self) -> ReleaseManifestDto {
        ReleaseManifestDto::from_parts(&self.model, &self.release_id, &self.files, POLICY_HEX)
            .expect("the fixture bundle assembles a manifest")
    }

    fn report(&self) -> ReleaseBundleReport<'_> {
        ReleaseBundleReport {
            model: &self.model,
            release_id: &self.release_id,
            policy_sha256: POLICY_HEX,
            staged: &self.pre_report,
            analysis: &self.analysis,
            compare: self.diff.as_ref().map(|result| CompareSummary {
                result,
                top_sections: &self.sections,
                top_symbols: &self.symbols,
            }),
            gate_counts: GateStateCounts {
                pass: 1,
                review: 1,
                block: 0,
                unknown: 1,
                not_applicable: 6,
            },
            gate: &self.gate,
            reviews: &self.reviews,
        }
    }

    fn html(&self) -> String {
        render_html(&self.report())
    }
}

// --------------------------------------------------------------------------- manifest: validates

#[test]
fn the_manifest_validates_against_the_published_v1_schema() {
    let doc = serde_json::to_value(manifest()).expect("a typed document serializes");
    let errors = validate(&doc);
    assert!(errors.is_empty(), "{errors:?}");
    assert_eq!(doc["schema_version"], 1);
    assert_eq!(doc["generated_by"]["product"], "FirmwareSight");
    assert!(doc["files"].as_array().expect("an array").len() >= 7);
}

#[test]
fn the_schema_declares_only_patterns_the_subset_evaluates() {
    // `pattern_holds` panics on anything outside its three cases, so a schema edit that outgrows the
    // validator must be caught here rather than by a panic in an unrelated test.
    let schema = manifest_schema();
    let mut found = 0;
    collect_patterns(&schema, &mut |pattern| {
        assert!(
            matches!(
                pattern,
                "^[0-9a-f]{64}$" | "^0x[0-9a-f]+$" | "^([0-9a-f]{40}|[0-9a-f]{64})$"
            ),
            "unimplemented pattern {pattern}"
        );
        found += 1;
    });
    assert_eq!(found, 2, "the two digest patterns v1 declares");
}

fn collect_patterns(node: &serde_json::Value, visit: &mut impl FnMut(&str)) {
    match node {
        serde_json::Value::Object(map) => {
            if let Some(pattern) = map.get("pattern").and_then(serde_json::Value::as_str) {
                visit(pattern);
            }
            for value in map.values() {
                collect_patterns(value, visit);
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                collect_patterns(item, visit);
            }
        }
        _ => {}
    }
}

// --------------------------------------------------------------------------- manifest: identities

#[test]
fn the_manifest_names_app_version_project_version_and_release_id_apart() {
    // §43: three different facts, three different fields. A test that only asked "is a version present"
    // would pass if the two were swapped.
    let manifest = manifest();
    assert_eq!(manifest.generated_by.product, "FirmwareSight");
    assert_eq!(manifest.generated_by.version, "0.6.0");
    assert_eq!(manifest.release.version, "1.4.2");
    assert_eq!(manifest.release.id, release_id());
    assert_eq!(
        ReleaseId::parse(&manifest.release.id)
            .expect("shaped")
            .short_hex(),
        "7fffffffffff"
    );
    assert_eq!(manifest.release.notes.as_deref(), Some(RELEASE_NOTES_NAME));
    assert_eq!(
        manifest.extensions.release_version_source,
        "git_tag.capture"
    );

    let text = to_json(&manifest);
    assert!(text.contains("\"version\":\"0.6.0\""), "{text}");
    assert!(text.contains("\"version\":\"1.4.2\""), "{text}");
    // v1's property lists are closed, so a field P4 invented would fail the schema rather than ride along.
    assert!(!text.contains("app_version"), "{text}");
}

#[test]
fn the_build_block_carries_the_primary_hash_and_the_observed_git_facts() {
    let manifest = manifest();
    assert_eq!(manifest.build.snapshot_id, "snap-0001");
    assert_eq!(manifest.build.artifact_sha256.as_deref(), Some(ELF_HEX));
    assert_eq!(manifest.build.git_commit.as_deref(), Some(HEAD_COMMIT));
    assert_eq!(manifest.build.git_tag.as_deref(), Some("v1.4.2"));
    assert_eq!(manifest.build.git_dirty, Some(false));
}

#[test]
fn an_unobserved_fact_is_null_in_the_manifest_and_never_an_empty_string() {
    let bare = ReleaseModel {
        workspace_git: WorkspaceGit::default(),
        release_notes: None,
        baseline_snapshot_id: None,
        ..model()
    }
    .validated()
    .expect("a release with nothing observed is still a release");
    let manifest = manifest_for(&bare, false, false);
    let doc = serde_json::to_value(&manifest).expect("serializes");
    for field in ["git_commit", "git_tag", "git_dirty"] {
        assert!(
            doc["build"][field].is_null(),
            "{field} must be null, not absent or empty"
        );
    }
    assert!(doc["release"]["notes"].is_null());
    assert!(doc["extensions"]["baseline_snapshot_id"].is_null());
    assert!(doc["extensions"]["diff_schema_version"].is_null());
}

// --------------------------------------------------------------------------- manifest: files[]

#[test]
fn every_bundle_file_except_the_manifest_itself_is_indexed() {
    let manifest = manifest();
    let paths: Vec<&str> = manifest
        .files
        .iter()
        .map(|entry| entry.path.as_str())
        .collect();
    for expected in [
        ANALYSIS_DOC_NAME,
        GATE_RESULTS_DOC_NAME,
        ACCEPTED_REVIEWS_DOC_NAME,
        REPORT_DOC_NAME,
        SHA256SUMS_NAME,
        RELEASE_NOTES_NAME,
        DIFF_DOC_NAME,
        &format!("{ARTIFACTS_DIR}/firmware.elf"),
        &format!("{ARTIFACTS_DIR}/firmware.map"),
    ] {
        assert!(paths.contains(&expected), "{expected} is not indexed");
    }
    assert!(!paths.contains(&MANIFEST_DOC_NAME));
    assert_eq!(
        manifest
            .files
            .iter()
            .filter(|entry| entry.path == SHA256SUMS_NAME)
            .count(),
        1,
        "the manifest covers SHA256SUMS, exactly once"
    );
}

#[test]
fn the_index_is_ordered_and_the_artifact_entries_match_the_fingerprint() {
    let model = model();
    let manifest = manifest();
    for pair in manifest.files.windows(2) {
        let left = pair[0].path.to_lowercase();
        let right = pair[1].path.to_lowercase();
        assert!(left <= right, "{left} sorts after {right}");
    }
    assert!(
        manifest
            .files
            .iter()
            .any(|entry| entry.path == SHA256SUMS_NAME && entry.sha256 == digest('e')),
        "the sums entry is indexed with the digest of the bytes written"
    );

    let elf = model
        .artifacts
        .iter()
        .find(|row| row.kind == ArtifactKind::Elf)
        .expect("the fixture ships an ELF");
    let entry = manifest
        .files
        .iter()
        .find(|file| file.path == format!("{ARTIFACTS_DIR}/{}", elf.file_name))
        .expect("the ELF is indexed");
    assert_eq!(entry.sha256, elf.sha256.hex());
    assert_eq!(entry.size, elf.byte_size);
}

#[test]
fn the_integrity_model_states_the_rule_the_writer_follows() {
    let manifest = manifest();
    let model = &manifest.extensions.integrity_model;
    assert_eq!(model.scheme, "sha256-two-layer/1");
    assert_eq!(model.sums_file, SHA256SUMS_NAME);
    assert_eq!(model.manifest_file, MANIFEST_DOC_NAME);
    assert!(!model.self_digest_written);
    assert_eq!(model.sums_excludes, [SHA256SUMS_NAME, MANIFEST_DOC_NAME]);
    assert_eq!(model.manifest_excludes, [MANIFEST_DOC_NAME]);
    assert!(model.rule.contains("own digest"), "{}", model.rule);
    assert_eq!(
        manifest.extensions.analysis_schema_version,
        firmwaresight_report::ANALYSIS_SCHEMA_VERSION
    );
    assert_eq!(manifest.extensions.gate_schema_version, 1);
    assert_eq!(manifest.extensions.accepted_reviews_schema_version, 1);
    assert_eq!(manifest.extensions.diff_schema_version, Some(1));
}

#[test]
fn a_manifest_is_byte_identical_when_rebuilt_and_one_line_on_the_wire() {
    let first = to_json(&manifest());
    assert_eq!(
        first,
        to_json(&manifest()),
        "the manifest is a hash target, not a rendering"
    );
    assert!(!first.contains('\n'), "{first}");
    let written = render_json(&manifest());
    assert!(written.ends_with('\n'));
    assert_eq!(written.matches('\n').count(), 1);
    assert!(written.contains("\"integrity_model\""));
}

// --------------------------------------------------------------------------- manifest: refusals

#[test]
fn a_release_id_that_is_not_one_is_refused() {
    for bad in [
        "",
        "release-",
        "not-a-release-0000000000000000000000000000000000000000000000000000000000000000",
        "release-7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
    ] {
        let error = ReleaseManifestDto::from_parts(&model(), bad, &staged(true, true), POLICY_HEX)
            .expect_err("an id that is not a release id cannot be published");
        assert!(
            matches!(error, ManifestError::Release(_)),
            "{bad:?} produced {error}"
        );
    }
}

#[test]
fn a_path_that_could_escape_or_collide_is_refused() {
    for escape in [
        "../outside.json",
        "/absolute/analysis.json",
        "artifacts\\firmware.elf",
        "C:/work/analysis.json",
        "artifacts/../escape.elf",
        "artifacts//firmware.elf",
        "",
    ] {
        let mut files = staged(true, true);
        files.retain(|entry| entry.path != ANALYSIS_DOC_NAME);
        files.push(ManifestFileDto::new(escape, digest('a'), 1));
        let error = ReleaseManifestDto::from_parts(&model(), &release_id(), &files, POLICY_HEX)
            .expect_err("`{escape}` is not a bundle-relative path");
        assert!(
            matches!(error, ManifestError::UnsafePath { .. }),
            "{escape:?} produced {error}"
        );
    }

    // `release-manifest.json` inside its own `files[]` is the faked self-hash §20 forbids.
    let mut self_hash = staged(true, true);
    self_hash.push(ManifestFileDto::new(MANIFEST_DOC_NAME, digest('0'), 4096));
    let error = ReleaseManifestDto::from_parts(&model(), &release_id(), &self_hash, POLICY_HEX)
        .expect_err("a manifest cannot index itself");
    assert!(
        matches!(error, ManifestError::HashesItself { .. }),
        "{error}"
    );

    // Two names that are one file to a Windows reader. The report names whichever of the pair sorts second,
    // so the assertion is on the refusal and on its text rather than on which half was picked.
    let mut clash = staged(true, true);
    clash.push(ManifestFileDto::new("Analysis.json", digest('b'), 1));
    let error = ReleaseManifestDto::from_parts(&model(), &release_id(), &clash, POLICY_HEX)
        .expect_err("a case-only duplicate collides on one of the two platforms");
    assert!(
        matches!(error, ManifestError::DuplicatePath { .. }),
        "{error}"
    );
    assert!(
        error.to_string().contains("analysis.json"),
        "the collision must be named: {error}"
    );

    let error = ReleaseManifestDto::from_parts(&model(), &release_id(), &[], POLICY_HEX)
        .expect_err("an empty index is not a manifest");
    assert_eq!(error, ManifestError::NoFiles);
}

#[test]
fn a_digest_that_is_not_lowercase_hex_is_refused_with_the_file_that_carried_it() {
    let mut files = staged(true, true);
    for entry in &mut files {
        if entry.path == format!("{ARTIFACTS_DIR}/firmware.elf") {
            entry.sha256 = ELF_HEX.to_uppercase();
        }
    }
    let error = ReleaseManifestDto::from_parts(&model(), &release_id(), &files, POLICY_HEX)
        .expect_err("an uppercase digest matches nothing a lowercase reader computes");
    assert!(
        matches!(&error, ManifestError::MalformedDigest { value } if value == "artifacts/firmware.elf"),
        "{error}"
    );

    // The policy digest enters the release id, so a malformed one is a corrupted Gate record.
    let error =
        ReleaseManifestDto::from_parts(&model(), &release_id(), &staged(true, true), "nope")
            .expect_err("a policy digest that is not one is refused");
    assert!(
        matches!(&error, ManifestError::MalformedDigest { value } if value == "policy_sha256"),
        "{error}"
    );
}

#[test]
fn a_missing_required_document_is_named_rather_than_omitted() {
    for missing in [
        ANALYSIS_DOC_NAME,
        GATE_RESULTS_DOC_NAME,
        ACCEPTED_REVIEWS_DOC_NAME,
        REPORT_DOC_NAME,
        SHA256SUMS_NAME,
    ] {
        let mut files = staged(true, true);
        files.retain(|entry| entry.path != missing);
        let error = ReleaseManifestDto::from_parts(&model(), &release_id(), &files, POLICY_HEX)
            .expect_err("a bundle cannot drop a document it always carries");
        assert!(
            matches!(&error, ManifestError::MissingRequiredEntry { required } if *required == missing),
            "{missing} produced {error}"
        );
    }
}

#[test]
fn an_index_that_disagrees_with_the_release_fingerprint_is_refused() {
    // A note in the bundle the Gate never observed.
    let model_without_notes = ReleaseModel {
        release_notes: None,
        ..model()
    };
    let error = ReleaseManifestDto::from_parts(
        &model_without_notes,
        &release_id(),
        &staged(true, true),
        POLICY_HEX,
    )
    .expect_err("an unhashed note cannot ship");
    assert!(
        matches!(error, ManifestError::NotesPresenceDisagrees),
        "{error}"
    );

    // A note the Gate observed that nobody copied.
    let mut without_notes_file = staged(true, true);
    without_notes_file.retain(|entry| entry.path != RELEASE_NOTES_NAME);
    let error =
        ReleaseManifestDto::from_parts(&model(), &release_id(), &without_notes_file, POLICY_HEX)
            .expect_err("a promised note that is absent is a broken bundle");
    assert!(
        matches!(error, ManifestError::NotesPresenceDisagrees),
        "{error}"
    );

    // A `diff.json` with no baseline behind it.
    let model_without_baseline = ReleaseModel {
        baseline_snapshot_id: None,
        release_notes: None,
        ..model()
    };
    let error = ReleaseManifestDto::from_parts(
        &model_without_baseline,
        &release_id(),
        &staged(false, true),
        POLICY_HEX,
    )
    .expect_err("a release without a baseline has no diff to carry");
    assert!(
        matches!(error, ManifestError::DiffPresenceDisagrees),
        "{error}"
    );

    // Bytes whose size changed after the fingerprint was taken.
    let mut retouched = staged(true, true);
    for entry in &mut retouched {
        if entry.path == format!("{ARTIFACTS_DIR}/firmware.map") {
            entry.size += 1;
        }
    }
    let error = ReleaseManifestDto::from_parts(&model(), &release_id(), &retouched, POLICY_HEX)
        .expect_err("a size that no longer matches must not be indexed under the old digest");
    assert!(
        matches!(&error, ManifestError::ShippedEntryMismatch { value } if value == "artifacts/firmware.map"),
        "{error}"
    );

    // A shipped file the model does not name.
    let mut extra = staged(true, true);
    extra.push(ManifestFileDto::new(
        format!("{ARTIFACTS_DIR}/debug-only.elf"),
        digest('8'),
        4_096,
    ));
    let error = ReleaseManifestDto::from_parts(&model(), &release_id(), &extra, POLICY_HEX)
        .expect_err("bytes nobody fingerprinted cannot ride along");
    assert!(
        matches!(&error, ManifestError::UnfingerprintedEntry { value } if value == "artifacts/debug-only.elf"),
        "{error}"
    );

    // A fingerprinted artifact nobody staged.
    let mut missing = staged(true, true);
    missing.retain(|entry| entry.path != format!("{ARTIFACTS_DIR}/firmware.map"));
    let error = ReleaseManifestDto::from_parts(&model(), &release_id(), &missing, POLICY_HEX)
        .expect_err("the MAP the release names must be in the bundle");
    assert!(
        matches!(&error, ManifestError::ArtifactNotShipped { value } if value == "artifacts/firmware.map"),
        "{error}"
    );
}

// --------------------------------------------------------------------------- SHA256SUMS

#[test]
fn the_sums_text_is_the_standard_form_lf_terminated_and_sorted() {
    let manifest = manifest();
    let text = render_sums(&manifest.files);
    assert!(text.ends_with('\n'));
    assert!(!text.contains('\r'), "the text is LF-only");
    assert!(!text.contains('\\'), "no host separator in a portable list");

    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(
        lines.len(),
        manifest.files.len() - 1,
        "every file except the manifest"
    );
    let mut paths: Vec<&str> = Vec::new();
    for line in &lines {
        let (hex, path) = line
            .split_once("  ")
            .unwrap_or_else(|| panic!("`{line}` is not `<digest>  <path>`"));
        assert_eq!(hex.len(), 64);
        assert!(Sha256::parse(hex).is_ok(), "{hex} is not lowercase hex");
        assert!(!path.starts_with('/'), "{path} is absolute");
        assert!(!path.contains(".."), "{path} traverses");
        paths.push(path);
    }

    let mut sorted = paths.clone();
    sorted.sort_unstable_by_key(|path| path.to_lowercase());
    assert_eq!(paths, sorted, "case-insensitive lexicographic order (§21)");
    assert!(!paths.contains(&SHA256SUMS_NAME));
    assert!(!paths.contains(&MANIFEST_DOC_NAME));
    assert!(paths.contains(&REPORT_DOC_NAME));
    assert!(paths.contains(&"artifacts/firmware.elf"));
}

#[test]
fn the_sums_text_excludes_itself_and_the_manifest_even_when_handed_over() {
    let mut files = staged(true, true);
    files.push(ManifestFileDto::new(MANIFEST_DOC_NAME, digest('0'), 1));
    let text = render_sums(&files);
    assert!(!text.contains(SHA256SUMS_NAME), "{text}");
    assert!(!text.contains(MANIFEST_DOC_NAME), "{text}");
    assert_eq!(
        text,
        render_sums(&manifest().files),
        "the list order is not the caller's"
    );
}

#[test]
fn the_sums_lines_and_the_manifest_agree_on_every_entry_they_share() {
    let manifest = manifest();
    for line in render_sums(&manifest.files).lines() {
        let (hex, path) = line.split_once("  ").expect("pair");
        let entry = manifest
            .files
            .iter()
            .find(|file| file.path == path)
            .unwrap_or_else(|| panic!("`{path}` is in the sums but not in the manifest"));
        assert_eq!(entry.sha256, hex);
    }
}

// --------------------------------------------------------------------------- release-report.html

#[test]
fn the_report_is_deterministic_self_contained_and_free_of_host_paths() {
    let bundle = Bundle::full();
    let html = bundle.html();
    assert_eq!(
        html,
        bundle.html(),
        "byte-stable, so a golden is a real check"
    );
    assert!(html.starts_with("<!doctype html>"));
    assert!(html.ends_with("</html>\n"));

    for forbidden in [
        "http://",
        "https://",
        "<script",
        "link rel",
        "@import",
        "\\",
        "/home/",
        "repo_root",
        "target/",
    ] {
        assert!(
            !html.contains(forbidden),
            "the export must not contain {forbidden:?}"
        );
    }
}

#[test]
fn the_report_shows_the_four_identities_apart_and_words_git_as_workspace_facts() {
    let html = Bundle::full().html();
    assert!(html.contains("FirmwareSight 0.6.0"), "app version visible");
    assert!(html.contains(">1.4.2<"), "project release version visible");
    assert!(html.contains(&release_id()), "release id visible");
    assert!(html.contains("FirmwareSight app version"));
    assert!(html.contains("Project release version"));
    assert!(html.contains("Workspace HEAD observed by the Release Gate"));
    assert!(html.contains(HEAD_COMMIT));
    assert!(
        html.contains("git_tag.capture"),
        "the version's source is stated"
    );

    // §43 and §62: the wording that would overstate what two hash lists prove.
    for forbidden in [
        "built from commit",
        "trusted",
        "authentic",
        "signed",
        "tamper",
        "notarized",
    ] {
        assert!(
            !html.contains(forbidden),
            "the report must not say {forbidden:?}"
        );
    }
}

#[test]
fn the_report_shows_the_gate_disposition_the_findings_and_the_reviews() {
    let html = Bundle::full().html();
    assert!(html.contains("Release Gate"));
    assert!(html.contains("Findings by state"));
    assert!(html.contains("All findings"));
    for state in ["PASS", "REVIEW", "UNKNOWN", "N/A"] {
        assert!(
            html.contains(state),
            "{state} is part of the five-state vocabulary"
        );
    }
    assert!(html.contains("MEMORY_GROWTH"));
    assert!(html.contains("raise the budget or shrink .text"));
    assert!(html.contains("ev-1"));
    assert!(html.contains("Severity before reviews"));
    assert!(html.contains("Severity after accepted reviews"));
    assert!(html.contains("Accepted reviews"));
    assert!(html.contains("release-owner"));
    assert!(
        html.contains("2026-09-30T08:12:44Z"),
        "the audit time already in the record"
    );
}

#[test]
fn an_empty_review_list_is_reported_as_a_fact_not_an_omission() {
    let html = Bundle::without_reviews().html();
    assert!(
        html.contains("No review was accepted for this run"),
        "{html}"
    );
    assert!(!html.contains("release-owner"));
    assert!(!html.contains("2026-09-30T08:12:44Z"));
}

#[test]
fn every_value_that_arrived_from_a_build_or_a_person_is_escaped() {
    let html = Bundle::full().html();
    assert!(!html.contains("<script>alert"));
    assert!(html.contains("&lt;script&gt;alert(&quot;x&quot;)&lt;/script&gt;"));
    assert!(!html.contains("<img onerror"));
    assert!(html.contains("&lt;img onerror=x&gt;"));
    assert!(
        html.contains("&lt;b&gt;modem&lt;/b&gt;"),
        "a review reason is escaped"
    );
    assert!(html.contains("no note found &amp; nowhere"));
    assert!(html.contains("over the budget &amp; rule"));
    assert!(
        html.contains("&amp; was left out"),
        "a warning message is escaped"
    );
}

#[test]
fn a_release_without_a_baseline_says_so_instead_of_printing_an_empty_table() {
    let bundle = Bundle::without_baseline();
    let html = bundle.html();
    assert!(html.contains("No baseline build was compared"), "{html}");
    assert!(!html.contains("Top growth, as Core ranked it"));
    assert!(!html.contains("Base (old) snapshot"));
    assert!(!html.contains("DIFF-0007"));
    assert!(!html.contains(&digest('9')));
    // The section that does not exist still explains itself.
    assert!(html.contains("statement about the release"));
}

#[test]
fn the_report_carries_the_integrity_tables_and_the_verification_steps() {
    let bundle = Bundle::full();
    let html = bundle.html();
    assert!(html.contains("Bundle integrity"));
    assert!(html.contains("The self-reference rule"));
    assert!(html.contains("Verifying this bundle without FirmwareSight"));
    assert!(html.contains("sha256sum -c SHA256SUMS"));
    assert!(
        html.contains("Get-FileHash"),
        "the Windows reader is not left out"
    );
    assert!(html.contains("self_digest_written</span> is false"));
    let manifest = bundle.manifest();
    for entry in &manifest.files {
        assert!(
            html.contains(&entry.path),
            "{} missing from the report's own index",
            entry.path
        );
    }
    for entry in &bundle.pre_report {
        assert!(
            html.contains(&entry.sha256),
            "a file written before the report is printed without its digest"
        );
    }
    // The cycle this shape exists to break: the report is payload, so the bytes of these two files do not
    // exist yet while it is composed. Printing either digest would make SHA256SUMS hash a file that
    // already hashes SHA256SUMS.
    assert!(
        !html.contains(&digest('d')),
        "the report printed a digest of itself"
    );
    assert!(
        !html.contains(&digest('e')),
        "the report printed SHA256SUMS's digest"
    );
}

#[test]
fn the_report_names_the_limits_this_release_carried() {
    let html = Bundle::full().html();
    assert!(html.contains("Known capability limits"));
    assert!(html.contains("Debug info"));
    assert!(html.contains("Object attribution"));
    assert!(html.contains("no object-file attribution without debug info"));
    assert!(
        html.contains("DIFF-0007"),
        "a comparison warning travels with the report"
    );
    assert!(
        html.contains("artifact.build_id"),
        "an unknown evidence row is listed"
    );
    assert!(html.contains("no NT_GNU_BUILD_ID note is present"));
}

#[test]
fn the_release_report_colours_come_from_the_frozen_token_set() {
    // AGENTS.md 11: a standalone export has no tokens.css to read, so every hex it prints must already be a
    // value in assets/design-tokens.json — including status.review, which the diff export never used.
    let tokens = std::fs::read_to_string(repo_root().join("assets").join("design-tokens.json"))
        .expect("the frozen token file is readable");
    let mut known: Vec<String> = tokens
        .split('"')
        .filter(|piece| {
            piece.starts_with('#')
                && matches!(piece.len(), 7 | 9)
                && piece[1..].bytes().all(|byte| byte.is_ascii_hexdigit())
        })
        .map(|piece| piece.to_uppercase())
        .collect();
    known.sort();
    known.dedup();

    let html = Bundle::full().html();
    let used: Vec<String> = html
        .split([' ', ';', ',', '\n', ':'])
        .filter(|piece| piece.starts_with('#') && matches!(piece.len(), 7 | 9))
        .map(|piece| piece.to_uppercase())
        .collect();
    assert!(
        !used.is_empty(),
        "the report is expected to carry token colours"
    );
    for colour in &used {
        assert!(
            known.contains(colour),
            "{colour} is not a value in assets/design-tokens.json"
        );
    }
    assert!(
        used.iter().any(|colour| colour == "#9A6700"),
        "the REVIEW chip must be the frozen status.review value"
    );
}

#[test]
fn the_analysis_document_and_the_manifest_describe_the_same_shipped_bytes() {
    // The bundle is read as one object: analysis.json names a file by the leaf it has inside the bundle and
    // the manifest names it under artifacts/. If those two ever disagreed, section 2 would print a kind for
    // a file that is not there.
    let bundle = Bundle::full();
    let manifest = bundle.manifest();
    let entry = manifest
        .files
        .iter()
        .find(|file| file.path == format!("{ARTIFACTS_DIR}/firmware.elf"))
        .expect("the ELF is indexed");
    let row = bundle
        .analysis
        .artifacts
        .iter()
        .find(|artifact| artifact.file_name == "firmware.elf")
        .expect("the analysis names the same leaf");
    assert_eq!(row.sha256.hex(), entry.sha256);
    assert_eq!(row.byte_size, entry.size);

    let html = bundle.html();
    let section2 = html
        .split("2. Shipped artifacts")
        .nth(1)
        .and_then(|rest| rest.split("3. Analysis summary").next())
        .expect("section 2 is present");
    assert!(section2.contains("artifacts/firmware.elf"));
    assert!(section2.contains("artifacts/firmware.map"));
    assert!(section2.contains(">elf<"), "the kind join resolved");
    assert!(
        !section2.contains(ANALYSIS_DOC_NAME),
        "generated docs are not shipped artifacts"
    );
}

#[test]
fn the_notes_section_names_the_file_it_shipped_and_refuses_to_embed_it() {
    let bundle = Bundle::full();
    let html = bundle.html();
    assert!(html.contains("yes, as <span class=\"mono\">release-notes.md</span>"));
    assert!(
        html.contains(NOTES_HEX),
        "the shipped copy is identified by digest"
    );
    let embedded = format!("<p>{NOTES_HEX}");
    assert!(!html.contains(&embedded));
    assert!(html.contains("never parsed, rewritten or embedded as raw HTML"));

    let without = Bundle::build(
        ReleaseModel {
            release_notes: None,
            ..model()
        },
        false,
        true,
        true,
    );
    let html = without.html();
    assert!(html.contains("observed no release notes"), "{html}");
    assert!(!html.contains(NOTES_HEX));
}
