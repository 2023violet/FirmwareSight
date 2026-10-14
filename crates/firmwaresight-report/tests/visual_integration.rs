use firmwaresight_core::domain::artifact::{Artifact, ParserId};
use firmwaresight_core::domain::build_snapshot::{BuildSnapshot, SnapshotBuilder};
use firmwaresight_core::domain::capability::{
    Availability, Capabilities, FormatSupport, Provision,
};
use firmwaresight_core::domain::diff::{
    BudgetState, DiffArtifact, DiffBudget, DiffMemory, DiffSnapshotInput, SideEvidence, compare,
};
use firmwaresight_core::domain::gate::{
    GateArtifactFact, GateContext, GateGitFacts, GateMemoryFacts, GatePolicy, GateUnknownEvidence,
};
use firmwaresight_core::domain::identity::{
    Architecture, ArtifactKind, ArtifactTimes, Bitness, BuildIdentity, Endianness, Fact, Sha256,
};
use firmwaresight_core::domain::memory::{
    LayoutSource, MemoryFootprint, MemoryLayout, MemoryRegion, MemoryRegionUsage,
    RegionAddressKind, RegionKind, RegionUsageRisk, RegionUsageState,
};
use firmwaresight_core::domain::release::{ReleaseArtifact, ReleaseModel};
use firmwaresight_core::domain::section::{Section, SectionFlags, SectionRole};
use firmwaresight_core::domain::symbol::{Symbol, SymbolBinding, SymbolKind, SymbolSectionRef};
use firmwaresight_report::analysis::AnalysisDocumentDto;
use firmwaresight_report::diff::DiffResultDto;
use firmwaresight_report::dto::AnalyzeResultDto;
use firmwaresight_report::gate::GateResultsDto;
use firmwaresight_report::visual::{self, HEIGHT, VisualFormat, VisualReport, WIDTH};

fn sha(marker: char) -> Sha256 {
    Sha256::parse(&marker.to_string().repeat(64)).expect("fixture digest")
}

fn section(name: &str, index: usize, file_size: u64) -> Section {
    Section {
        index,
        name: Fact::known(name.to_owned()),
        role: SectionRole::Code,
        flags: SectionFlags {
            alloc: true,
            execute: true,
            ..SectionFlags::default()
        },
        virtual_address: Fact::known(0x0800_0000 + index as u64 * 0x100),
        load_address: Fact::known(0x0800_0000 + index as u64 * 0x100),
        file_offset: Fact::known(0x40 + index as u64 * 0x100),
        file_size,
        memory_size: Fact::known(file_size),
        region: Fact::known("FLASH".to_owned()),
    }
}

fn artifact(marker: char, file_name: &str) -> Artifact {
    Artifact {
        id: format!("artifact-{marker}"),
        path: file_name.to_owned(),
        kind: ArtifactKind::Elf,
        sha256: sha(marker),
        byte_size: 4096,
        parser_id: ParserId::elf_object("visual-fixture"),
        architecture: Architecture::Thumb,
        bitness: Bitness::Bits32,
        endianness: Endianness::Little,
        entry_point: Fact::known(0x0800_0041),
        build_id: Fact::unknown("fixture has no build id"),
        times: ArtifactTimes::default(),
        identity: BuildIdentity::default(),
        evidence: Vec::new(),
    }
}

fn snapshot(
    marker: char,
    file_name: &str,
    text_bytes: u64,
    symbol_size: Option<u64>,
) -> BuildSnapshot {
    let text = section(".text", 1, text_bytes);
    let memory = MemoryFootprint::compute(
        std::slice::from_ref(&text),
        &MemoryLayout {
            regions: vec![MemoryRegion::new(
                "FLASH",
                0x0800_0000,
                8192,
                "rx",
                "ev-visual-region",
            )],
            source: LayoutSource::ProjectConfig,
        },
    );
    let mut builder = SnapshotBuilder::new("0.6.0-visual-test")
        .project_id("synthetic-visual-fixture")
        .artifact(artifact(marker, file_name))
        .sections(vec![text])
        .memory(memory)
        .capabilities(Capabilities {
            elf: FormatSupport::Supported,
            sections: Availability::Available,
            symbols: Availability::Partial,
            debug_info: Availability::Unavailable,
            map: Provision::NotProvided,
            object_attribution: Availability::Unavailable,
            git: Availability::Unknown,
        });
    if let Some(size) = symbol_size {
        builder = builder.symbols(vec![Symbol {
            name: Fact::known("firmware_start".to_owned()),
            address: Fact::known(0x0800_0041),
            size: Fact::known(size),
            kind: SymbolKind::Function,
            binding: SymbolBinding::Global,
            section: SymbolSectionRef::Index(1),
        }]);
    }
    builder.seal().expect("synthetic snapshot")
}

fn analysis(snapshot: &BuildSnapshot, leaf: &str) -> AnalyzeResultDto {
    AnalyzeResultDto::from_snapshot(snapshot, leaf)
}

fn portable_analysis(snapshot: &BuildSnapshot) -> AnalysisDocumentDto {
    AnalysisDocumentDto::from_snapshot(
        snapshot,
        &[ReleaseArtifact {
            kind: ArtifactKind::Elf,
            file_name: "firmware.elf".to_owned(),
            sha256: sha('a'),
            byte_size: 4096,
        }],
    )
}

fn region(
    name: &str,
    used: Option<u64>,
    capacity: Option<u64>,
    state: RegionUsageState,
) -> MemoryRegionUsage {
    MemoryRegionUsage {
        name: name.to_owned(),
        kind: RegionKind::Nonvolatile,
        origin: 0x0800_0000,
        capacity_bytes: capacity,
        used_bytes: used,
        remaining_bytes: None,
        remaining_reason: "remaining space is not allocator headroom",
        state,
        risk: if state == RegionUsageState::Exact {
            RegionUsageRisk::WithinCapacity
        } else {
            RegionUsageRisk::Unknown
        },
        evidence_source: "synthetic fixture",
        evidence_id: "ev-region-1".to_owned(),
        address_kind: RegionAddressKind::Load,
        sections: Vec::new(),
        reasons: if state == RegionUsageState::Exact {
            Vec::new()
        } else {
            vec!["capacity evidence unavailable".to_owned()]
        },
    }
}

fn diff_input(
    id: &str,
    marker: char,
    bytes: Option<u64>,
    state: BudgetState,
    size: Option<u64>,
) -> DiffSnapshotInput {
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
        symbols: size
            .map(|size| {
                vec![firmwaresight_core::domain::diff::DiffSymbol {
                    ordinal: 0,
                    name: Fact::known("growth_contributor".to_owned()),
                    address: Some(0x0800_0100),
                    size: Fact::known(size),
                    kind: "Function".to_owned(),
                    binding: "Global".to_owned(),
                    section_ref: "Index(1)".to_owned(),
                }]
            })
            .unwrap_or_default(),
    }
}

fn gate_context(snapshot: &BuildSnapshot) -> GateContext {
    GateContext {
        snapshot_id: snapshot.id().as_str().to_owned(),
        artifacts: vec![GateArtifactFact {
            kind: ArtifactKind::Elf,
            sha256: Fact::known(sha('a').hex().to_owned()),
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
    }
}

#[test]
fn analyze_card_carries_core_bytes_regions_and_unknown_without_inventing_capacity() {
    let snapshot = snapshot('a', "C:\\private-canary\\firmware.elf", 1536, Some(48));
    let dto = analysis(&snapshot, "firmware.elf");
    let mut regions = snapshot
        .memory()
        .expect("Core memory facts")
        .region_usage
        .clone();
    regions.push(region("SRAM", None, None, RegionUsageState::Unknown));
    let report = VisualReport::analyze(&dto, &regions);
    let svg = String::from_utf8(visual::render(&report, VisualFormat::Svg).expect("SVG"))
        .expect("UTF-8 SVG");
    let html = String::from_utf8(visual::render(&report, VisualFormat::Html).expect("HTML"))
        .expect("UTF-8 HTML");

    assert!(svg.contains("1536 bytes"));
    assert!(svg.contains("Unknown"));
    assert!(svg.contains("Mapped 1536 bytes / capacity 8192 bytes"));
    assert!(html.contains("Capacity unknown for footprint budgets"));
    assert!(html.contains("FLASH / nonvolatile image"));
    assert!(html.contains("capacity evidence unavailable"));
    assert!(!svg.contains("private-canary"));
    assert!(!html.contains("private-canary"));
    assert!(
        !svg.contains("Usage 18%"),
        "footprint utilisation is not established by a region row"
    );
}

#[test]
fn compare_card_keeps_signed_core_delta_partial_unknown_and_ranked_growth() {
    let base = diff_input("snap-base", 'b', Some(1000), BudgetState::Partial, Some(64));
    let target = diff_input("snap-target", 'c', Some(1260), BudgetState::Exact, Some(96));
    let core = compare(&base, &target).expect("distinct synthetic snapshots");
    let dto = DiffResultDto::from_diff(&core);
    let report = VisualReport::compare(&dto, &core);
    let svg = String::from_utf8(visual::render(&report, VisualFormat::Svg).expect("SVG"))
        .expect("UTF-8 SVG");
    let html = String::from_utf8(visual::render(&report, VisualFormat::Html).expect("HTML"))
        .expect("UTF-8 HTML");

    assert!(svg.contains("A snap-base.elf -&gt; B snap-target.elf"));
    assert!(svg.contains("A 1000 bytes"));
    assert!(svg.contains("B 1260 bytes"));
    assert!(svg.contains("Delta +260 bytes"));
    assert!(svg.contains("A Unknown"));
    assert!(svg.contains("B Unknown"));
    assert!(svg.contains("Delta Unknown"));
    assert!(svg.contains("Unknown is not zero"));
    assert!(html.contains("Core Top 5 changed-symbol size growth"));
    assert!(html.contains("+32 bytes"));
    assert!(
        !svg.contains("A 0 |"),
        "an unavailable side remains absent, never zero"
    );

    let exact_base = diff_input(
        "snap-exact-base",
        'd',
        Some(1000),
        BudgetState::Exact,
        Some(64),
    );
    let exact_target = diff_input(
        "snap-exact-target",
        'e',
        Some(1260),
        BudgetState::Exact,
        Some(96),
    );
    let exact_core = compare(&exact_base, &exact_target).expect("exact fixture pair");
    let exact_report = VisualReport::compare(&DiffResultDto::from_diff(&exact_core), &exact_core);
    let exact_svg = String::from_utf8(
        visual::render(&exact_report, VisualFormat::Svg).expect("exact compare SVG"),
    )
    .expect("UTF-8 SVG");
    assert!(exact_svg.contains("Delta +260 bytes"));

    let shrink_base = diff_input(
        "snap-shrink-base",
        '8',
        Some(1260),
        BudgetState::Exact,
        None,
    );
    let shrink_target = diff_input(
        "snap-shrink-target",
        '9',
        Some(1000),
        BudgetState::Exact,
        None,
    );
    let shrink_core = compare(&shrink_base, &shrink_target).expect("shrink fixture pair");
    let shrink_report =
        VisualReport::compare(&DiffResultDto::from_diff(&shrink_core), &shrink_core);
    let shrink_svg = String::from_utf8(
        visual::render(&shrink_report, VisualFormat::Svg).expect("shrinking compare SVG"),
    )
    .expect("UTF-8 SVG");
    assert!(shrink_svg.contains("Delta -260 bytes"));
}

#[test]
fn gate_card_preserves_core_review_block_unknown_and_explicit_release_identity() {
    let snapshot = snapshot('a', "firmware.elf", 2048, None);
    let context = gate_context(&snapshot);
    let evaluation = context.evaluate("gate-synthetic-visual-run");
    let gate = GateResultsDto::from_evaluation(&context, &evaluation, &"d".repeat(64), 1);
    let doc = portable_analysis(&snapshot);
    let release_id = ReleaseModel::release_id_from(&"e".repeat(64));
    let report = VisualReport::gate(&doc, &gate, Some((&release_id, "1.7.0")));
    let svg = String::from_utf8(visual::render(&report, VisualFormat::Svg).expect("SVG"))
        .expect("UTF-8 SVG");
    let html = String::from_utf8(visual::render(&report, VisualFormat::Html).expect("HTML"))
        .expect("UTF-8 HTML");

    for state in ["REVIEW", "BLOCK", "UNKNOWN"] {
        assert!(html.contains(state), "missing Core Gate state {state}");
    }
    for state_with_icon in ["! REVIEW", "x BLOCK", "o UNKNOWN", "+ PASS"] {
        assert!(
            html.contains(state_with_icon),
            "Gate state must be readable with its glyph: {state_with_icon}"
        );
    }
    assert!(svg.contains("gate-synthetic-visual-run"));
    assert!(html.contains(&release_id));
    assert!(html.contains("Project release version"));
    assert!(html.contains("PASS is a policy result, not shipping permission"));
    assert!(html.contains("SHA-256 checks bytes, not publisher identity"));
}

#[test]
fn svg_html_png_are_deterministic_standalone_and_png_decodes_to_meaningful_pixels() {
    let snapshot = snapshot('a', "固件.elf", 4096, Some(192));
    let report = VisualReport::analyze(&analysis(&snapshot, "固件.elf"), &[]);
    for format in [VisualFormat::Svg, VisualFormat::Html, VisualFormat::Png] {
        let first = visual::render(&report, format).expect("synthetic report renders");
        let second = visual::render(&report, format).expect("same report renders twice");
        assert_eq!(first, second, "{format:?} must be deterministic");
        match format {
            VisualFormat::Svg => {
                let svg = std::str::from_utf8(&first).expect("SVG UTF-8");
                assert!(svg.starts_with("<svg "));
                assert!(svg.contains(&format!("viewBox=\"0 0 {WIDTH} {HEIGHT}\"")));
                assert!(!svg.contains("<script"));
                assert!(!svg.contains("foreignObject"));
                assert!(!svg.contains("href=\"http"));
            }
            VisualFormat::Html => {
                let html = std::str::from_utf8(&first).expect("HTML UTF-8");
                assert!(html.starts_with("<!doctype html>"));
                assert!(html.contains("@page { size: A4;"));
                assert!(html.contains("@media print"));
                assert!(html.contains("native PDF export is unavailable"));
                assert!(!html.contains("<script"));
                assert!(!html.contains("https://"));
            }
            VisualFormat::Png => {
                assert!(first.starts_with(b"\x89PNG\r\n\x1a\n"));
                let decoded =
                    resvg::tiny_skia::Pixmap::decode_png(&first).expect("independent PNG decode");
                assert_eq!((decoded.width(), decoded.height()), (WIDTH, HEIGHT));
                let nonwhite = decoded
                    .pixels()
                    .iter()
                    .filter(|p| p.red() < 245 || p.green() < 245 || p.blue() < 245)
                    .count();
                assert!(
                    nonwhite > 10_000,
                    "PNG should contain substantial report pixels, got {nonwhite}"
                );
            }
        }
    }
}

#[test]
fn untrusted_markup_host_paths_unicode_and_huge_region_inputs_are_bounded() {
    let mut dto = analysis(&snapshot('f', "firmware.elf", 1024, None), "firmware.elf");
    dto.artifact.file_name = "../../private-canary/<script onload=alert(1)> 固件".to_owned();
    let regions = (0..100_000)
        .map(|i| {
            region(
                &format!("region-{i}-<script>C:\\private-canary"),
                Some(i),
                Some(8192),
                RegionUsageState::Exact,
            )
        })
        .collect::<Vec<_>>();
    let report = VisualReport::analyze(&dto, &regions);
    let svg = visual::render(&report, VisualFormat::Svg).expect("bounded SVG");
    let html = visual::render(&report, VisualFormat::Html).expect("bounded HTML");
    let svg = String::from_utf8(svg).expect("UTF-8 SVG");
    let html = String::from_utf8(html).expect("UTF-8 HTML");

    assert!(svg.len() < 16_000, "card row count must remain bounded");
    assert!(
        html.len() < 100_000,
        "full details must not duplicate 100k presentation rows"
    );
    for output in [&svg, &html] {
        assert!(!output.contains("private-canary"));
        assert!(!output.contains("<script"));
        assert!(!output.contains("onload="));
    }
    assert!(html.contains("label withheld"));
}

#[test]
fn zero_and_subpixel_occupancy_omit_only_the_decorative_foreground() {
    let dto = analysis(&snapshot('a', "firmware.elf", 1024, None), "firmware.elf");
    for (used, capacity, state, foreground) in [
        (Some(0), Some(8192), RegionUsageState::Exact, false),
        (Some(1), Some(u64::MAX), RegionUsageState::Exact, false),
        (Some(8192), Some(8192), RegionUsageState::Exact, true),
        (Some(1), None, RegionUsageState::Unknown, false),
        (Some(1), Some(0), RegionUsageState::Exact, false),
    ] {
        let report = VisualReport::analyze(&dto, &[region("FLASH", used, capacity, state)]);
        let svg = String::from_utf8(visual::render(&report, VisualFormat::Svg).unwrap()).unwrap();
        assert!(
            !svg.contains("width=\"0\""),
            "zero-width decoration is invalid geometry"
        );
        assert!(!svg.contains("NaN") && !svg.contains("Infinity"));
        assert_eq!(
            svg.contains("width=\"144\" height=\"16\" fill=\"#2563EB\""),
            foreground
        );
        let html = String::from_utf8(visual::render(&report, VisualFormat::Html).unwrap()).unwrap();
        assert!(html.contains(&format!("{} bytes", used.unwrap())));
        assert!(!html.contains("100% occupancy") && !html.contains("0% occupancy"));
        let png = visual::render(&report, VisualFormat::Png).unwrap();
        let decoded = resvg::tiny_skia::Pixmap::decode_png(&png).unwrap();
        assert_eq!((decoded.width(), decoded.height()), (WIDTH, HEIGHT));
    }
}
