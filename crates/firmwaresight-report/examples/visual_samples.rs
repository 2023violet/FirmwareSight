//! Write six deterministic, synthetic visual cohorts to a caller-selected directory.
//!
//! Usage: cargo run -p firmwaresight-report --example visual_samples -- <output-directory>
//! No firmware, project configuration, or owner data is read by this example.

use std::path::PathBuf;

use firmwaresight_core::domain::artifact::{Artifact, ParserId};
use firmwaresight_core::domain::build_snapshot::SnapshotBuilder;
use firmwaresight_core::domain::capability::{
    Availability, Capabilities, FormatSupport, Provision,
};
use firmwaresight_core::domain::diff::{
    BudgetState, DiffArtifact, DiffBudget, DiffMemory, DiffSnapshotInput, DiffSymbol, SideEvidence,
    compare,
};
use firmwaresight_core::domain::gate::{
    GateArtifactFact, GateContext, GateGitFacts, GateMemoryFacts, GatePolicy, GateUnknownEvidence,
};
use firmwaresight_core::domain::identity::{
    Architecture, ArtifactKind, ArtifactTimes, Bitness, BuildIdentity, Endianness, Fact, Sha256,
};
use firmwaresight_core::domain::memory::{
    LayoutSource, MemoryFootprint, MemoryLayout, MemoryRegion,
};
use firmwaresight_core::domain::release::{ReleaseArtifact, ReleaseModel};
use firmwaresight_core::domain::section::{Section, SectionFlags, SectionRole};
use firmwaresight_report::analysis::AnalysisDocumentDto;
use firmwaresight_report::diff::DiffResultDto;
use firmwaresight_report::dto::AnalyzeResultDto;
use firmwaresight_report::gate::GateResultsDto;
use firmwaresight_report::visual::{self, VisualFormat, VisualReport};

fn digest(marker: char) -> Sha256 {
    Sha256::parse(&marker.to_string().repeat(64)).expect("synthetic digest")
}

fn build(
    marker: char,
    leaf: &str,
    bytes: u64,
    region_name: &str,
) -> firmwaresight_core::domain::build_snapshot::BuildSnapshot {
    let artifact = Artifact {
        id: format!("synthetic-{marker}"),
        path: leaf.to_owned(),
        kind: ArtifactKind::Elf,
        sha256: digest(marker),
        byte_size: 4096,
        parser_id: ParserId::elf_object("visual-sample"),
        architecture: Architecture::Thumb,
        bitness: Bitness::Bits32,
        endianness: Endianness::Little,
        entry_point: Fact::known(0x0800_0041),
        build_id: Fact::unknown("synthetic sample has no build id"),
        times: ArtifactTimes::default(),
        identity: BuildIdentity::default(),
        evidence: Vec::new(),
    };
    let section = Section {
        index: 1,
        name: Fact::known(".text".to_owned()),
        role: SectionRole::Code,
        flags: SectionFlags {
            alloc: true,
            execute: true,
            ..SectionFlags::default()
        },
        virtual_address: Fact::known(0x0800_0000),
        load_address: Fact::known(0x0800_0000),
        file_offset: Fact::known(0x100),
        file_size: bytes,
        memory_size: Fact::known(bytes),
        region: Fact::known(region_name.to_owned()),
    };
    let sections = vec![section];
    SnapshotBuilder::new("0.6.0-visual-sample")
        .project_id("synthetic-visual-sample")
        .artifact(artifact)
        .sections(sections.clone())
        .memory(MemoryFootprint::compute(
            &sections,
            &MemoryLayout {
                regions: vec![MemoryRegion::new(
                    region_name,
                    0x0800_0000,
                    262_144,
                    "rx",
                    "synthetic-region-1",
                )],
                source: LayoutSource::ProjectConfig,
            },
        ))
        .capabilities(Capabilities {
            elf: FormatSupport::Supported,
            sections: Availability::Available,
            symbols: Availability::Partial,
            debug_info: Availability::Unavailable,
            map: Provision::NotProvided,
            object_attribution: Availability::Unavailable,
            git: Availability::Unknown,
        })
        .seal()
        .expect("synthetic snapshot")
}

fn diff_input(id: &str, marker: char, bytes: Option<u64>, state: BudgetState) -> DiffSnapshotInput {
    DiffSnapshotInput {
        snapshot_id: id.to_owned(),
        artifact: DiffArtifact {
            file_name: format!("{id}.elf"),
            sha256: marker.to_string().repeat(64),
            byte_size: 4096,
        },
        memory: Some(DiffMemory {
            nonvolatile: DiffBudget { state, bytes },
            runtime_ram: DiffBudget {
                state: BudgetState::Unknown,
                bytes: None,
            },
            excluded_metadata_bytes: 0,
            evidence: SideEvidence {
                map_backed: false,
                layout_source: "none".to_owned(),
                weakest_basis: None,
            },
        }),
        sections: Vec::new(),
        symbols: Vec::new(),
    }
}

fn write_cohort(root: &std::path::Path, name: &str, report: &VisualReport) -> Result<(), String> {
    let directory = root.join(name);
    std::fs::create_dir_all(&directory).map_err(|e| format!("{}: {e}", directory.display()))?;
    for (suffix, format) in [
        ("svg", VisualFormat::Svg),
        ("png", VisualFormat::Png),
        ("html", VisualFormat::Html),
    ] {
        let output = directory.join(format!("{name}.{suffix}"));
        let bytes = visual::render(report, format).map_err(|e| e.to_string())?;
        visual::write_new(&output, &bytes).map_err(|e| format!("{}: {e}", output.display()))?;
        println!("{}", output.display());
    }
    Ok(())
}

fn run() -> Result<(), String> {
    let output = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or_else(|| "usage: visual_samples <output-directory>".to_owned())?;
    std::fs::create_dir_all(&output).map_err(|e| format!("{}: {e}", output.display()))?;

    let known = build('a', "固件-controller.elf", 1536, "FLASH");
    let analyze = AnalyzeResultDto::from_snapshot(&known, "固件-controller.elf");
    let regions = known
        .memory()
        .expect("Core memory facts")
        .region_usage
        .clone();
    write_cohort(
        &output,
        "analyze-known",
        &VisualReport::analyze(&analyze, &regions),
    )?;

    let mut canary = analyze.clone();
    canary.artifact.file_name = format!(
        "C:\\synthetic-private-canary\\<script onload=alert(1)> {}",
        "固件控制器".repeat(80),
    );
    write_cohort(
        &output,
        "privacy-cjk-long-label",
        &VisualReport::analyze(&canary, &regions),
    )?;

    let region_name = format!("闪存区域-{}", "配置空间".repeat(80));
    let large = build('f', "large-u64.elf", u64::MAX, &region_name);
    let large_dto = AnalyzeResultDto::from_snapshot(&large, "large-u64.elf");
    let large_regions = large
        .memory()
        .expect("Core memory facts")
        .region_usage
        .clone();
    write_cohort(
        &output,
        "large-u64-cjk-region",
        &VisualReport::analyze(&large_dto, &large_regions),
    )?;

    let base = diff_input("base", 'b', Some(1200), BudgetState::Partial);
    let target = diff_input("target", 'c', None, BudgetState::Unknown);
    let core_diff = compare(&base, &target).map_err(|e| format!("core compare failed: {e:?}"))?;
    let compare_report = VisualReport::compare(&DiffResultDto::from_diff(&core_diff), &core_diff);
    write_cohort(&output, "compare-unknown", &compare_report)?;

    let mut ranked_base = diff_input("ranked-base", '6', Some(10_000), BudgetState::Exact);
    let mut ranked_target = diff_input("ranked-target", '7', Some(10_800), BudgetState::Exact);
    ranked_base.symbols = ranked_symbols(&[("startup", 40), ("scheduler", 120), ("protocol", 64)]);
    ranked_target.symbols =
        ranked_symbols(&[("startup", 72), ("scheduler", 168), ("protocol", 80)]);
    let ranked_core = compare(&ranked_base, &ranked_target)
        .map_err(|error| format!("core ranked compare failed: {error:?}"))?;
    let ranked_report =
        VisualReport::compare(&DiffResultDto::from_diff(&ranked_core), &ranked_core);
    write_cohort(&output, "compare-ranked-growth", &ranked_report)?;

    let analysis = AnalysisDocumentDto::from_snapshot(
        &known,
        &[ReleaseArtifact {
            kind: ArtifactKind::Elf,
            file_name: "固件-controller.elf".to_owned(),
            sha256: digest('a'),
            byte_size: 4096,
        }],
    );
    let context = GateContext {
        snapshot_id: known.id().as_str().to_owned(),
        artifacts: vec![GateArtifactFact {
            kind: ArtifactKind::Elf,
            sha256: Fact::known(digest('a').hex().to_owned()),
            byte_size: 4096,
        }],
        attachments: Vec::new(),
        memory: Some(GateMemoryFacts::default()),
        git: GateGitFacts {
            available: true,
            head_commit: Fact::known("0123456789abcdef0123456789abcdef01234567".to_owned()),
            exact_tag: Fact::known("v1.7.0".to_owned()),
            dirty: Fact::known(false),
        },
        version: None,
        release_notes: None,
        growth: firmwaresight_core::domain::gate::GateGrowthFacts::without_baseline(),
        unknown_evidence: GateUnknownEvidence {
            count: 1,
            sample_refs: vec!["evidence:synthetic-unknown".to_owned()],
        },
        policy: GatePolicy {
            required_artifact_kinds: vec!["elf".to_owned(), "map".to_owned()],
            unknown_evidence_review_count: Some(0),
            ..GatePolicy::default()
        },
    };
    let evaluation = context.evaluate("gate-synthetic-review-block");
    let gate = GateResultsDto::from_evaluation(&context, &evaluation, &"d".repeat(64), 1);
    let release_id = ReleaseModel::release_id_from(&"e".repeat(64));
    let release_preview = VisualReport::gate(&analysis, &gate, Some((&release_id, "1.7.0")));
    write_cohort(&output, "release-review-block-unknown", &release_preview)?;
    Ok(())
}

fn ranked_symbols(rows: &[(&str, u64)]) -> Vec<DiffSymbol> {
    rows.iter()
        .enumerate()
        .map(|(ordinal, (name, size))| DiffSymbol {
            ordinal: ordinal as i64,
            name: Fact::known((*name).to_owned()),
            address: Some(0x0800_0100 + ordinal as u64 * 0x100),
            size: Fact::known(*size),
            kind: "Function".to_owned(),
            binding: "Global".to_owned(),
            section_ref: "Index(1)".to_owned(),
        })
        .collect()
}

fn main() {
    if let Err(error) = run() {
        eprintln!("visual sample generation failed: {error}");
        std::process::exit(1);
    }
}
