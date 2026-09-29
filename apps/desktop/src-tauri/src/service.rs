//! The desktop application service: selection, analysis, projection and persistence.
//!
//! Nothing in this module knows about Tauri. That is what makes the parity claim testable — the
//! desktop's own tests call the same functions its commands do, and the `#[tauri::command]`
//! bodies stay thin wrappers that move work onto a blocking thread.
//!
//! Two ways to arrive at an analysis exist, and they differ only in who chose the file: the P0
//! fixture path resolves a closed key inside the application's own fixture root, while the P1-A0
//! artifact path takes a path the Rust-side native dialog returned.
//!
//! Either way the UI never supplies a path (`05_ENGINEERING/03`, Tauri security rules: no
//! `read_file`/`get_any_path` surface), and both paths run the same pipeline and the same projection.

use std::path::{Path, PathBuf};

use firmwaresight_artifact::ArtifactError;
use firmwaresight_artifact::pipeline::{self, Analysis, AnalysisRequest};
use firmwaresight_report::AnalyzeResultDto;

use crate::ipc::{
    AnalysisSummaryDto, ArtifactDto, BudgetDto, CapabilitiesDto, EvidenceSummaryDto, FixtureKey,
    IdentityDto, MemorySummaryDto,
};

/// The summary came from a committed fixture the shell chose.
pub const SOURCE_FIXTURE: &str = "fixture";
/// The summary came from a file the user selected in a native dialog.
pub const SOURCE_ARTIFACT: &str = "artifact";

/// Analyze a path the Rust side obtained itself, with an optional MAP.
///
/// This is the whole P1-A0 addition: the same `pipeline::analyze` the CLI and the fixture path
/// call, reached with a user-chosen file instead of a catalog key. Nothing here re-implements a
/// fact - format detection, size guarding, parsing and accounting all stay in the pipeline.
///
/// # Errors
///
/// Any [`ArtifactError`] from intake or parsing, unchanged.
pub fn analyze_paths(artifact: &Path, map: Option<&Path>) -> Result<Analysis, ArtifactError> {
    let mut request = AnalysisRequest::new(artifact);
    if let Some(map_path) = map {
        request = request.with_map(map_path);
    }
    pipeline::analyze(&request)
}

/// The name the user sees for a path: the file name, never a parent directory.
#[must_use]
pub fn display_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}

/// Resolves closed fixture keys onto real paths inside the application's fixture root.
#[derive(Debug, Clone)]
pub struct FixtureCatalog {
    root: PathBuf,
}

impl FixtureCatalog {
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Development and test layout: `<repo>/fixtures`. `FIRMWARESIGHT_FIXTURE_DIR` overrides it
    /// for packaged runs and for the parity harness; the value never comes from the WebView.
    #[must_use]
    pub fn from_environment() -> Self {
        if let Some(dir) = std::env::var_os("FIRMWARESIGHT_FIXTURE_DIR") {
            return Self::new(dir);
        }
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        Self::new(
            manifest
                .parent() // apps/desktop/src-tauri -> apps/desktop
                .and_then(Path::parent) // -> apps
                .and_then(Path::parent) // -> <repo root>
                .unwrap_or(manifest)
                .join("fixtures"),
        )
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The directory of a fixture key, relative to the catalog root.
    const fn dir_of(key: FixtureKey) -> (&'static str, Option<&'static str>) {
        match key {
            FixtureKey::P0Basic => ("elf/p0-basic", None),
            FixtureKey::P0DualRegion => ("elf/p0-dual-region", Some("firmware.map")),
        }
    }

    #[must_use]
    pub fn artifact_path(&self, key: FixtureKey) -> PathBuf {
        let (dir, _) = Self::dir_of(key);
        self.root.join(dir).join("firmware.elf")
    }

    /// The file name the UI shows. Derived here, never accepted from the caller.
    #[must_use]
    pub fn file_name(&self, key: FixtureKey) -> String {
        self.artifact_path(key)
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "firmware.elf".to_owned())
    }

    /// The artifact and optional MAP for a fixture key. Both are resolved here, never supplied.
    fn request(&self, key: FixtureKey) -> AnalysisRequest {
        let (dir, map) = Self::dir_of(key);
        let mut request = AnalysisRequest::new(self.artifact_path(key));
        if let Some(map_name) = map {
            request = request.with_map(self.root.join(dir).join(map_name));
        }
        request
    }

    pub fn analyze(&self, key: FixtureKey) -> Result<Analysis, ArtifactError> {
        let request = self.request(key);
        pipeline::analyze(&request)
    }
}

/// Project an analysis into the bounded IPC summary.
///
/// The projection reads only from `AnalyzeResultDto`, which is the very same type the CLI
/// renders. No arithmetic is repeated here, so the two surfaces cannot drift by construction.
#[must_use]
pub fn summarize(dto: &AnalyzeResultDto, source: &str) -> AnalysisSummaryDto {
    let mut counts = EvidenceSummaryDto {
        total: dto.evidence.len(),
        observed: 0,
        derived: 0,
        declared: 0,
        unknown: 0,
    };
    for item in &dto.evidence {
        match item.classification {
            "observed" => counts.observed += 1,
            "derived" => counts.derived += 1,
            "declared" => counts.declared += 1,
            // Any other label is unknown-provenance by definition; counting it elsewhere would
            // overstate what was actually observed.
            _ => counts.unknown += 1,
        }
    }

    AnalysisSummaryDto {
        source: source.to_owned(),
        artifact: ArtifactDto {
            file_name: dto.artifact.file_name.clone(),
            kind: dto.artifact.kind.clone(),
            sha256: dto.artifact.sha256.clone(),
            byte_size: dto.artifact.byte_size,
            parser_id: dto.artifact.parser_id.clone(),
            architecture: dto.artifact.architecture.clone(),
            bitness: dto.artifact.bitness.clone(),
            endianness: dto.artifact.endianness.clone(),
            entry_point: dto.artifact.entry_point.clone(),
            entry_point_unknown_reason: dto.artifact.entry_point_unknown_reason.clone(),
            build_id: dto.artifact.build_id.clone(),
            build_id_unknown_reason: dto.artifact.build_id_unknown_reason.clone(),
        },
        identity: IdentityDto {
            schema: dto.schema.to_owned(),
            schema_stability: dto.schema_stability.to_owned(),
            snapshot_id: dto.snapshot_id.clone(),
            normalization_version: dto.normalization_version.to_owned(),
            created_by_fwsight_version: dto.created_by_fwsight_version.clone(),
        },
        memory: MemorySummaryDto {
            accounting_rule: dto.memory.accounting_rule.to_owned(),
            layout_source: dto.memory.layout_source.to_owned(),
            weakest_evidence_basis: dto.memory.weakest_evidence_basis.map(str::to_owned),
            admissible_for_hard_block: dto.memory.admissible_for_hard_block,
            nonvolatile_image_footprint: budget(&dto.memory.nonvolatile_image_footprint),
            runtime_ram_footprint: budget(&dto.memory.runtime_ram_footprint),
            dual_accounted_sections: dto.memory.dual_accounted_sections.clone(),
            excluded_metadata_bytes: dto.memory.excluded_metadata_bytes,
        },
        section_count: dto.counts.sections,
        symbol_count: dto.counts.symbols,
        capabilities: CapabilitiesDto {
            elf: dto.capabilities.elf.to_owned(),
            sections: dto.capabilities.sections.to_owned(),
            symbols: dto.capabilities.symbols.to_owned(),
            debug_info: dto.capabilities.debug_info.to_owned(),
            map: dto.capabilities.map.to_owned(),
            object_attribution: dto.capabilities.object_attribution.to_owned(),
            git: dto.capabilities.git.to_owned(),
        },
        evidence_summary: counts,
    }
}

fn budget(source: &firmwaresight_report::dto::BudgetDto) -> BudgetDto {
    BudgetDto {
        state: source.state.to_owned(),
        classification: source.classification.to_owned(),
        bytes: source.bytes,
        unattributed: source.unattributed.clone(),
        reason: source.reason.clone(),
    }
}

/// Project a user-selected artifact into the same bounded summary.
///
/// `file_name` is the name of the file the dialog returned, taken by the Rust side. The projection
/// is identical apart from the source label, which is the point: choosing your own file changes
/// where the bytes came from, not what FirmwareSight says about them.
#[must_use]
pub fn project_artifact(file_name: &str, analysis: &Analysis) -> AnalysisSummaryDto {
    let dto = AnalyzeResultDto::from_snapshot(&analysis.snapshot, file_name);
    summarize(&dto, SOURCE_ARTIFACT)
}
/// Project a completed fixture analysis into the bounded IPC summary.
///
/// Takes the analysis rather than the snapshot alone: the DTO is built by the same
/// `AnalyzeResultDto::from_snapshot` the CLI uses, so this is the single projection point for
/// both surfaces.
#[must_use]
pub fn project(
    catalog: &FixtureCatalog,
    key: FixtureKey,
    analysis: &Analysis,
) -> AnalysisSummaryDto {
    let dto = AnalyzeResultDto::from_snapshot(&analysis.snapshot, &catalog.file_name(key));
    summarize(&dto, SOURCE_FIXTURE)
}
